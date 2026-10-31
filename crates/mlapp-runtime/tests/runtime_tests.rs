use mlapp_lang::parse_app_classdef;
use mlapp_runtime::{call_octave_bridge, AppHandle, AppRuntime, RuntimeError, UiMutation};
use mlapp_ui::{Component, ComponentKind, PropertyValue, Rect};
use rustlab_math::Value;

#[test]
fn test_app_runtime_lifecycle_and_mutations() {
    let mut fig = Component::new(ComponentKind::UIFigure, "UIFigure");
    fig.position = Rect::new(100.0, 100.0, 640.0, 480.0);

    let btn = Component::new(ComponentKind::Button, "RunButton");
    fig.children.push(btn);

    let handle = AppHandle::from_ui_tree("TestApp", &fig);
    let mut runtime = AppRuntime::new(handle.clone());

    let source = r#"
classdef TestApp < matlab.apps.AppBase
    properties (Access = public)
        Count = 0
    end
    methods (Access = private)
        function startupFcn(app)
            x = 10 + 20;
        end
        function RunButtonPushed(app, event)
            uialert(app, "Calculation Finished", "Success");
        end
    end
end
"#;

    let class_def = parse_app_classdef(source).expect("parse classdef");
    runtime.load_class_def(class_def);

    // Run startup
    runtime.run_startup_fcn().expect("run startupFcn");

    // Run callback
    runtime
        .run_callback(
            "RunButtonPushed",
            Value::Double(rustlab_math::DenseArray::scalar(0.0)),
        )
        .expect("run callback");

    // Verify UI mutation drained
    let mutations = runtime.app_handle.drain_mutations();
    assert_eq!(mutations.len(), 1);
    match &mutations[0] {
        UiMutation::Alert { title, message } => {
            assert_eq!(title, "Success");
            assert_eq!(message, "Calculation Finished");
        }
        _ => panic!("Expected alert mutation"),
    }

    // Mutate component property through app handle
    runtime
        .app_handle
        .set_component_property(
            "RunButton",
            "Text",
            PropertyValue::String("Done".to_string()),
        )
        .expect("set property");

    let post_mutations = runtime.app_handle.drain_mutations();
    assert_eq!(post_mutations.len(), 1);
    match &post_mutations[0] {
        UiMutation::SetProperty {
            component,
            property,
            value,
        } => {
            assert_eq!(component, "RunButton");
            assert_eq!(property, "Text");
            assert_eq!(value, &PropertyValue::String("Done".to_string()));
        }
        _ => panic!("Expected set property mutation"),
    }
}

#[test]
fn test_unsupported_function_diagnostics() {
    let err = call_octave_bridge("besselk", &[]).expect_err("must fail");
    match err {
        RuntimeError::UnsupportedFunction {
            name,
            recommendation,
        } => {
            assert_eq!(name, "besselk");
            assert!(recommendation.contains("not currently implemented"));
        }
        other => panic!("expected UnsupportedFunction, got {other:?}"),
    }
}

#[test]
fn test_runtime_permission_enforcement() {
    let fig = Component::new(ComponentKind::UIFigure, "UIFigure");
    let handle = AppHandle::from_ui_tree("SecurityApp", &fig);
    let mut runtime = AppRuntime::new(handle);

    // Block file I/O and network
    runtime.enforce_permissions(false, false);

    let source = r#"
classdef SecurityApp < matlab.apps.AppBase
    methods (Access = private)
        function BlockedFileCall(app)
            save("data.mat");
        end
        function BlockedNetCall(app)
            web("http://example.com");
        end
    end
end
"#;

    let class_def = parse_app_classdef(source).expect("parse");
    runtime.load_class_def(class_def);

    let file_err = runtime
        .run_callback(
            "BlockedFileCall",
            Value::Double(rustlab_math::DenseArray::scalar(0.0)),
        )
        .expect_err("must block save");
    assert!(format!("{file_err}").contains("Security violation"));
    assert!(format!("{file_err}").contains("allow_file_io = false"));

    let net_err = runtime
        .run_callback(
            "BlockedNetCall",
            Value::Double(rustlab_math::DenseArray::scalar(0.0)),
        )
        .expect_err("must block web");
    assert!(format!("{net_err}").contains("Security violation"));
    assert!(format!("{net_err}").contains("allow_network = false"));
}

#[test]
fn test_cooperative_cancellation() {
    let fig = Component::new(ComponentKind::UIFigure, "UIFigure");
    let handle = AppHandle::from_ui_tree("CancelApp", &fig);
    let mut runtime = AppRuntime::new(handle.clone());

    let source = r#"
classdef CancelApp < matlab.apps.AppBase
    methods (Access = private)
        function LongTask(app)
            x = 1;
            y = 2;
        end
    end
end
"#;

    let class_def = parse_app_classdef(source).expect("parse");
    runtime.load_class_def(class_def);

    // Trigger cancellation token
    runtime.app_handle.cancel();

    let err = runtime
        .run_callback(
            "LongTask",
            Value::Double(rustlab_math::DenseArray::scalar(0.0)),
        )
        .expect_err("must be cancelled");
    assert!(format!("{err}").contains("cancelled by user request"));
}

#[test]
fn test_comprehensive_runtime_sandbox_policies() {
    let fig = Component::new(ComponentKind::UIFigure, "UIFigure");
    let handle = AppHandle::from_ui_tree("SandboxApp", &fig);
    let mut runtime = AppRuntime::new(handle);

    // Enforce default deny: file_io=false, network=false, system=false, eval=false, interop=false
    runtime.enforce_sandbox_policies(false, false, false, false, false);

    let source = r#"
classdef SandboxApp < matlab.apps.AppBase
    methods (Access = private)
        function CallSystem(app)
            system("dir");
        end
        function CallAtSystem(app)
            f = @system;
            f("dir");
        end
        function CallCellfunSystem(app)
            cellfun(@system, {"dir"});
        end
        function CallBuiltin(app)
            builtin("system", "dir");
        end
        function CallEval(app)
            eval("x = 1");
        end
        function CallEvalc(app)
            evalc("x = 1");
        end
        function CallEvalin(app)
            evalin("base", "x = 1");
        end
        function CallAssignin(app)
            assignin("base", "x", 1);
        end
        function CallRun(app)
            run("script.m");
        end
        function CallSource(app)
            source("script.m");
        end
        function CallFevalSystem(app)
            feval("system", "dir");
        end
        function CallStr2funcSystem(app)
            f = str2func("system");
            f("dir");
        end
        function CallLoad(app)
            load("data.mat");
        end
        function CallSave(app)
            save("data.mat");
        end
        function CallFopen(app)
            fid = fopen("test.txt");
        end
        function CallFwrite(app)
            fwrite(1, "data");
        end
        function CallDelete(app)
            delete("secret.txt");
        end
        function CallRmdir(app)
            rmdir("folder");
        end
        function CallCd(app)
            cd("..");
        end
        function CallCopyfile(app)
            copyfile("a", "b");
        end
        function CallMovefile(app)
            movefile("a", "b");
        end
        function CallGetenv(app)
            getenv("PATH");
        end
        function CallSetenv(app)
            setenv("FOO", "BAR");
        end
        function CallExit(app)
            exit();
        end
        function CallBang(app)
            feval("!", "dir");
        end
        function CallUrlread(app)
            urlread("http://example.com");
        end
        function CallUrlwrite(app)
            urlwrite("http://example.com", "out.html");
        end
        function CallWebread(app)
            webread("http://example.com");
        end
        function CallWebsave(app)
            websave("out.html", "http://example.com");
        end
        function CallJava(app)
            javaObject("java.lang.String");
        end
        function CallPython(app)
            python("script.py");
        end
        function CallLoadlibrary(app)
            loadlibrary("libm.so");
        end
        function CallCalllib(app)
            calllib("libm", "sin", 1.0);
        end
    end
end
"#;

    let class_def = parse_app_classdef(source).expect("parse");
    runtime.load_class_def(class_def);

    let dummy_val = Value::Double(rustlab_math::DenseArray::scalar(0.0));

    // 1. system & handles
    let err = runtime
        .run_callback("CallSystem", dummy_val.clone())
        .expect_err("system must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'system'"));

    let err = runtime
        .run_callback("CallAtSystem", dummy_val.clone())
        .expect_err("@system must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'system'"));

    let err = runtime
        .run_callback("CallCellfunSystem", dummy_val.clone())
        .expect_err("cellfun(@system) must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'system'"));

    let err = runtime
        .run_callback("CallBuiltin", dummy_val.clone())
        .expect_err("builtin must fail");
    assert!(format!("{err}").contains("Security violation: builtin invocation of 'system'"));

    let err = runtime
        .run_callback("CallGetenv", dummy_val.clone())
        .expect_err("getenv must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'getenv'"));

    let err = runtime
        .run_callback("CallSetenv", dummy_val.clone())
        .expect_err("setenv must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'setenv'"));

    let err = runtime
        .run_callback("CallExit", dummy_val.clone())
        .expect_err("exit must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'exit'"));

    let err = runtime
        .run_callback("CallBang", dummy_val.clone())
        .expect_err("! must fail");
    assert!(format!("{err}").contains("Security violation: shell command function '!'"));

    // 2. eval, evalc, evalin, assignin, run, source
    let err = runtime
        .run_callback("CallEval", dummy_val.clone())
        .expect_err("eval must fail");
    assert!(format!("{err}").contains("Security violation: dynamic evaluation function 'eval'"));

    let err = runtime
        .run_callback("CallEvalc", dummy_val.clone())
        .expect_err("evalc must fail");
    assert!(format!("{err}").contains("Security violation: dynamic evaluation function 'evalc'"));

    let err = runtime
        .run_callback("CallEvalin", dummy_val.clone())
        .expect_err("evalin must fail");
    assert!(format!("{err}").contains("Security violation: dynamic evaluation function 'evalin'"));

    let err = runtime
        .run_callback("CallAssignin", dummy_val.clone())
        .expect_err("assignin must fail");
    assert!(format!("{err}").contains("Security violation: dynamic evaluation function 'assignin'"));

    let err = runtime
        .run_callback("CallRun", dummy_val.clone())
        .expect_err("run must fail");
    assert!(format!("{err}").contains("Security violation: dynamic evaluation function 'run'"));

    let err = runtime
        .run_callback("CallSource", dummy_val.clone())
        .expect_err("source must fail");
    assert!(format!("{err}").contains("Security violation: dynamic evaluation function 'source'"));

    // 3. feval('system', ...) and str2func('system')
    let err = runtime
        .run_callback("CallFevalSystem", dummy_val.clone())
        .expect_err("feval system must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'system'"));

    let err = runtime
        .run_callback("CallStr2funcSystem", dummy_val.clone())
        .expect_err("str2func system must fail");
    assert!(format!("{err}").contains("Security violation: shell command function 'system'"));

    // 4. File I/O: load, save, fopen, fwrite, delete, rmdir, cd, copyfile, movefile
    let err = runtime
        .run_callback("CallLoad", dummy_val.clone())
        .expect_err("load must fail");
    assert!(format!("{err}").contains("Security violation: function 'load'"));

    let err = runtime
        .run_callback("CallSave", dummy_val.clone())
        .expect_err("save must fail");
    assert!(format!("{err}").contains("Security violation: function 'save'"));

    let err = runtime
        .run_callback("CallFopen", dummy_val.clone())
        .expect_err("fopen must fail");
    assert!(format!("{err}").contains("Security violation: function 'fopen'"));

    let err = runtime
        .run_callback("CallFwrite", dummy_val.clone())
        .expect_err("fwrite must fail");
    assert!(format!("{err}").contains("Security violation: function 'fwrite'"));

    let err = runtime
        .run_callback("CallDelete", dummy_val.clone())
        .expect_err("delete must fail");
    assert!(format!("{err}").contains("Security violation: function 'delete'"));

    let err = runtime
        .run_callback("CallRmdir", dummy_val.clone())
        .expect_err("rmdir must fail");
    assert!(format!("{err}").contains("Security violation: function 'rmdir'"));

    let err = runtime
        .run_callback("CallCd", dummy_val.clone())
        .expect_err("cd must fail");
    assert!(format!("{err}").contains("Security violation: function 'cd'"));

    let err = runtime
        .run_callback("CallCopyfile", dummy_val.clone())
        .expect_err("copyfile must fail");
    assert!(format!("{err}").contains("Security violation: function 'copyfile'"));

    let err = runtime
        .run_callback("CallMovefile", dummy_val.clone())
        .expect_err("movefile must fail");
    assert!(format!("{err}").contains("Security violation: function 'movefile'"));

    // 5. Network: urlread, urlwrite, webread, websave
    let err = runtime
        .run_callback("CallUrlread", dummy_val.clone())
        .expect_err("urlread must fail");
    assert!(format!("{err}").contains("Security violation: function 'urlread'"));

    let err = runtime
        .run_callback("CallUrlwrite", dummy_val.clone())
        .expect_err("urlwrite must fail");
    assert!(format!("{err}").contains("Security violation: function 'urlwrite'"));

    let err = runtime
        .run_callback("CallWebread", dummy_val.clone())
        .expect_err("webread must fail");
    assert!(format!("{err}").contains("Security violation: function 'webread'"));

    let err = runtime
        .run_callback("CallWebsave", dummy_val.clone())
        .expect_err("websave must fail");
    assert!(format!("{err}").contains("Security violation: function 'websave'"));

    // 6. External interop: java, python, loadlibrary, calllib
    let err = runtime
        .run_callback("CallJava", dummy_val.clone())
        .expect_err("java must fail");
    assert!(
        format!("{err}").contains("Security violation: external language integration 'javaObject'")
    );

    let err = runtime
        .run_callback("CallPython", dummy_val.clone())
        .expect_err("python must fail");
    assert!(format!("{err}").contains("Security violation: external language integration 'python'"));

    let err = runtime
        .run_callback("CallLoadlibrary", dummy_val.clone())
        .expect_err("loadlibrary must fail");
    assert!(format!("{err}")
        .contains("Security violation: external language integration 'loadlibrary'"));

    let err = runtime
        .run_callback("CallCalllib", dummy_val.clone())
        .expect_err("calllib must fail");
    assert!(
        format!("{err}").contains("Security violation: external language integration 'calllib'")
    );
}

#[test]
fn test_cancellation_while_loop_and_long_builtin() {
    let fig = Component::new(ComponentKind::UIFigure, "UIFigure");
    let handle = AppHandle::from_ui_tree("CancelLoopApp", &fig);
    let mut runtime = AppRuntime::new(handle.clone());

    let source = r#"
classdef CancelLoopApp < matlab.apps.AppBase
    methods (Access = private)
        function InfiniteWhile(app)
            while true
                x = 1;
            end
        end
        function LongPause(app)
            pause(10);
        end
    end
end
"#;

    let class_def = parse_app_classdef(source).expect("parse");
    runtime.load_class_def(class_def);

    let dummy_val = Value::Double(rustlab_math::DenseArray::scalar(0.0));

    // 1. Test while true, end interrupted by cancel token after 50ms
    let h1 = handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h1.cancel();
    });

    let start = std::time::Instant::now();
    let while_err = runtime
        .run_callback("InfiniteWhile", dummy_val.clone())
        .expect_err("while must be cancelled");
    assert!(
        start.elapsed().as_millis() < 500,
        "while loop must exit quickly upon cancellation"
    );
    assert!(format!("{while_err}").contains("cancelled"));

    // 2. Reset cancellation token for pause test
    handle
        .cancel_token
        .store(false, std::sync::atomic::Ordering::Relaxed);
    let h2 = handle.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        h2.cancel();
    });

    let pause_start = std::time::Instant::now();
    let pause_err = runtime
        .run_callback("LongPause", dummy_val.clone())
        .expect_err("pause must be cancelled");
    assert!(
        pause_start.elapsed().as_millis() < 500,
        "pause builtin must exit quickly upon cancellation"
    );
    assert!(format!("{pause_err}").contains("cancelled"));
}
