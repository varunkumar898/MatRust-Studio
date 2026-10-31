use mlapp_lang::{
    analyze_compatibility, extract_code_from_document_xml, parse_app_classdef, AccessModifier,
    CompatStatus, MethodRole,
};

#[test]
fn test_extract_code_from_xml() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<![CDATA[
classdef FilterApp < matlab.apps.AppBase
    properties (Access = public)
        UIFigure matlab.ui.Figure
        RunBtn   matlab.ui.control.Button
    end
end
]]>
</w:document>"#;

    let code = extract_code_from_document_xml(xml).expect("extract code");
    assert!(code.contains("classdef FilterApp"));
    assert!(code.contains("RunBtn"));
}

#[test]
fn test_parse_app_classdef_structure() {
    let source = r#"
classdef SineApp < matlab.apps.AppBase
    properties (Access = public)
        UIFigure  matlab.ui.Figure
        PlotAxes  matlab.ui.control.UIAxes
        StartBtn  matlab.ui.control.Button
    end

    properties (Access = private)
        SamplingRate = 1000
    end

    methods (Access = private)
        function startupFcn(app)
            app.SamplingRate = 2000;
        end

        function StartBtnPushed(app, event)
            t = 0:0.01:1;
            y = sin(2 * pi * 5 * t);
            plot(app.PlotAxes, t, y);
        end

        function res = calculateFft(app, signal)
            res = abs(fft(signal));
        end
    end
end
"#;

    let app = parse_app_classdef(source).expect("parse classdef");
    assert_eq!(app.name, "SineApp");
    assert_eq!(app.super_class.as_deref(), Some("matlab.apps.AppBase"));

    // Check properties
    assert_eq!(app.properties.len(), 4);
    let ui_fig = app
        .properties
        .iter()
        .find(|p| p.name == "UIFigure")
        .unwrap();
    assert_eq!(ui_fig.access, AccessModifier::Public);
    assert_eq!(ui_fig.type_annotation.as_deref(), Some("matlab.ui.Figure"));

    let samp = app
        .properties
        .iter()
        .find(|p| p.name == "SamplingRate")
        .unwrap();
    assert_eq!(samp.access, AccessModifier::Private);
    assert_eq!(samp.initial_value.as_deref(), Some("1000"));

    // Check methods
    assert_eq!(app.methods.len(), 3);

    let startup = app.methods.get("startupFcn").unwrap();
    assert!(matches!(startup.role, MethodRole::StartupFcn));

    let callback = app.methods.get("StartBtnPushed").unwrap();
    assert!(matches!(callback.role, MethodRole::Callback { .. }));

    let helper = app.methods.get("calculateFft").unwrap();
    assert!(matches!(helper.role, MethodRole::Helper));
    assert_eq!(helper.inputs, vec!["app", "signal"]);
    assert_eq!(helper.outputs, vec!["res"]);
}

#[test]
fn test_static_compatibility_scanner() {
    let source = r#"
classdef TestSecurityApp < matlab.apps.AppBase
    methods (Access = private)
        function Run(app)
            x = zeros(10, 10);
            eval("delete file.txt");
            system("dir");
        end
    end
end
"#;

    let app = parse_app_classdef(source).expect("parse");
    let report = analyze_compatibility(&app);

    assert_eq!(report.status, CompatStatus::Unsupported);
    assert!(report.hazardous_functions.contains(&"eval".to_string()));
    assert!(report.hazardous_functions.contains(&"system".to_string()));
    assert!(report.compatibility_score < 60);
}

#[test]
fn test_check_compat_deterministic_snapshot() {
    let source = r#"
classdef SnapshotApp < matlab.apps.AppBase
    methods (Access = private)
        function Calculate(app)
            a = linspace(0, 10, 50);
            b = sin(a);
            c = cos(b);
            plot(a, c);
            disp('Finished');
        end
    end
end
"#;

    let app = parse_app_classdef(source).expect("parse");
    let report = analyze_compatibility(&app);
    insta::assert_yaml_snapshot!(report);
}
