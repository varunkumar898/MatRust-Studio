use mlapp_host::{AppSession, HostConfig, HostNotification, UiEvent};
use mlapp_runtime::UiMutation;
use mlapp_ui::PropertyValue;
use std::io::{Cursor, Write};

#[test]
fn test_app_host_session_lifecycle() {
    let candidate_paths = [
        "fixtures/sample_apps/r2021a_sample.mlapp",
        "crates/mlapp-container/fixtures/sample_apps/r2021a_sample.mlapp",
        "../../fixtures/sample_apps/r2021a_sample.mlapp",
    ];

    let mut file = None;
    for path in candidate_paths {
        if let Ok(f) = std::fs::File::open(path) {
            file = Some(f);
            break;
        }
    }
    let file = file.expect("open sample app from candidate paths");

    let mut session = AppSession::launch(file, HostConfig::default()).expect("launch session");
    assert_eq!(session.manifest.app_name, "FixtureApp_R2021a");
    assert_eq!(session.ui_tree.name, "UIFigure");

    // Send ButtonClick event to StartBtn
    session
        .send_event(UiEvent::ButtonClick {
            component_name: "StartBtn".to_string(),
        })
        .expect("send event");

    // Wait slightly for worker thread processing
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Poll notification
    while let Some(notif) = session.poll_notification() {
        match notif {
            HostNotification::Mutations(m) => {
                println!("Received mutations: {:?}", m);
            }
            HostNotification::Alert { title, message } => {
                println!("Alert: {title}: {message}");
            }
            HostNotification::Error(e) => {
                println!("Worker error: {e}");
            }
            HostNotification::Terminated => {}
        }
    }

    session.shutdown();
}

#[test]
fn test_end_to_end_callback_with_advanced_matlab_features() {
    use mlapp_runtime::UiMutation;
    use mlapp_ui::PropertyValue;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // [Content_Types].xml
        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        // metadata/coreProperties.xml
        zip.start_file("metadata/coreProperties.xml", options)
            .unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>E2EApp</dc:title>
    <cp:version>1.0.0</cp:version>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        // appdesigner/app.xml
        zip.start_file("appdesigner/app.xml", options).unwrap();
        let app_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<appdata>
    <UIFigure Name="UIFigure" Position="[100, 100, 640, 480]">
        <Label Name="ResultLabel" Text="Initial" Position="20, 300, 200, 30" />
        <Button Name="ComputeBtn" Text="Compute" Position="20, 200, 100, 30" ButtonPushedFcn="ComputeBtnPushed" />
    </UIFigure>
</appdata>"#;
        zip.write_all(app_xml.as_bytes()).unwrap();

        // matlab/document.xml
        zip.start_file("matlab/document.xml", options).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef E2EApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            ResultLabel
            ComputeBtn
        end
        methods (Access = private)
            function ComputeBtnPushed(app, event)
                % 1. for loop
                acc = 0;
                for i = 1:3
                    acc = acc + i;
                end
                % acc = 6

                % 2. struct
                s = struct();
                s.val = acc;
                % s.val = 6

                % 3. cell array
                c = {10, 20};

                % 4. anonymous function
                f = @(x) x * 2;

                % 5. cellfun
                res_cell = cellfun(f, c);
                % res_cell = [20, 40]

                % 6. x(end+1) = 5
                x = [1, 2];
                x(end+1) = 5;
                % x = [1, 2, 5]

                % 7. A(2,:) = []
                A = [1, 2; 3, 4; 5, 6];
                A(2,:) = [];
                % A = [1, 2; 5, 6]

                % 8. Transpose
                At = A';
                % At = [1, 5; 2, 6]

                % Compute combined total:
                % s.val (6) + sum(res_cell) (60) + sum(x) (8) + At(1,1) (1) + At(2,2) (6) = 81
                total = s.val + sum(res_cell) + sum(x) + At(1,1) + At(2,2);
                app.ResultLabel.Text = sprintf('Result=%d', total);
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let mut session =
        AppSession::launch(Cursor::new(buf), HostConfig::default()).expect("launch e2e session");
    assert_eq!(session.manifest.app_name, "E2EApp");
    println!(
        "Methods found in class_def: {:?}",
        session.class_def.methods.keys()
    );

    // Send ButtonClick event
    session
        .send_event(UiEvent::ButtonClick {
            component_name: "ComputeBtn".to_string(),
        })
        .expect("send click");

    // Wait and poll notifications
    let mut collected_mutations = Vec::new();
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < 1000 {
        if let Some(notif) = session.poll_notification() {
            match notif {
                HostNotification::Mutations(mut m) => {
                    let has_result = m.iter().any(|item| {
                        if let UiMutation::SetProperty {
                            component,
                            property,
                            value,
                        } = item
                        {
                            component == "ResultLabel"
                                && property == "Text"
                                && *value == PropertyValue::String("Result=81".to_string())
                        } else {
                            false
                        }
                    });
                    collected_mutations.append(&mut m);
                    if has_result {
                        break;
                    }
                }
                HostNotification::Error(e) => {
                    panic!("Worker error in e2e callback: {e}");
                }
                _ => {}
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Assert mutation queue contains ResultLabel.Text = "Result=81"
    let text_mutation = collected_mutations.iter().find(|m| match m {
        UiMutation::SetProperty {
            component,
            property,
            value,
        } => {
            component == "ResultLabel"
                && property == "Text"
                && *value == PropertyValue::String("Result=81".to_string())
        }
        _ => false,
    });
    assert!(
        text_mutation.is_some(),
        "Mutation queue must contain SetProperty ResultLabel.Text = 'Result=81', got: {:?}",
        collected_mutations
    );

    // Assert internal component state in app_handle is also updated
    {
        let state = session
            .app_handle
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let label = state
            .components
            .get("ResultLabel")
            .expect("ResultLabel component");
        assert_eq!(
            label.get_property("Text"),
            Some(&PropertyValue::String("Result=81".to_string()))
        );
    }

    session.shutdown();
}

#[test]
fn test_host_automatic_timeout_cancelling_infinite_loop() {
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opt = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", opt).unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>InfiniteLoopApp</dc:title>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", opt).unwrap();
        let app_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<appdata>
    <UIFigure Name="UIFigure" Position="[100, 100, 640, 480]">
        <Button Name="StuckBtn" Text="Run Forever" Position="20, 200, 100, 30" ButtonPushedFcn="StuckBtnPushed" />
    </UIFigure>
</appdata>"#;
        zip.write_all(app_xml.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", opt).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef InfiniteLoopApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            StuckBtn
        end
        methods (Access = private)
            function StuckBtnPushed(app, event)
                while true
                    % Infinite loop - host watchdog must fire automatic timeout cancellation
                end
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    // Configure session with 100ms automatic timeout limit
    let config = HostConfig {
        callback_timeout_ms: 100,
        ..HostConfig::default()
    };

    let mut session =
        AppSession::launch(Cursor::new(buf), config).expect("launch stuck app session");

    // Click the button to trigger infinite loop
    session
        .send_event(UiEvent::ButtonClick {
            component_name: "StuckBtn".to_string(),
        })
        .expect("send click");

    // Poll for notifications - automatic timeout cancellation must produce HostNotification::Error
    let start = std::time::Instant::now();
    let mut timeout_occurred = false;
    let mut error_msg = String::new();

    while start.elapsed().as_millis() < 2000 {
        if let Some(HostNotification::Error(e)) = session.poll_notification() {
            timeout_occurred = true;
            error_msg = e;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    assert!(
        timeout_occurred,
        "Host watchdog must automatically cancel stuck callback without manual cancel, elapsed: {} ms",
        start.elapsed().as_millis()
    );
    assert!(
        error_msg.contains("cancelled") || error_msg.contains("Execution cancelled"),
        "Error message must indicate cancellation, got: {error_msg}"
    );

    session.shutdown();
}

#[test]
fn test_comprehensive_e2e_features_and_axes_model() {
    use mlapp_plot::{AxesModel, PlotSeries};
    use mlapp_runtime::UiMutation;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opt = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", opt).unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>ComprehensiveApp</dc:title>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", opt).unwrap();
        let app_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<appdata>
    <UIFigure Name="UIFigure" Position="[100, 100, 800, 600]">
        <NumericEditField Name="ValField" Value="7.0" Position="20, 500, 100, 30" />
        <UIAxes Name="PlotAxes" Position="200, 50, 550, 500" />
        <Button Name="PlotBtn" Text="Plot Now" Position="20, 450, 100, 30" ButtonPushedFcn="PlotBtnPushed" />
    </UIFigure>
</appdata>"#;
        zip.write_all(app_xml.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", opt).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef ComprehensiveApp < matlab.apps.AppBase
        properties (Access = private)
            BaseOffset = 10
        end
        properties (Access = public)
            UIFigure
            ValField
            PlotAxes
            PlotBtn
        end
        methods (Access = public)
            function startupFcn(app)
                % startupFcn execution updates private property
                app.BaseOffset = app.BaseOffset + 5; % 15
            end
        end
        methods (Access = private)
            function res = helperCompute(app, delta)
                % Helper method with negative numbers [1 -2] and end index
                arr = [1 -2];
                res = app.BaseOffset + delta + arr(end); % 15 + delta - 2 = 13 + delta
            end

            function PlotBtnPushed(app, event)
                % Read app.Field.Value
                fld = app.ValField.Value; % 7
                
                % Call helper method
                total_bias = app.helperCompute(fld); % 13 + 7 = 20

                % First plot line
                x1 = [1, 2, 3];
                y1 = [total_bias, total_bias + 10, total_bias + 20];
                plot(app.PlotAxes, x1, y1);

                % Hold on
                hold on;

                % Second plot line
                x2 = [4, 5];
                y2 = [100, 200];
                plot(app.PlotAxes, x2, y2);
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let mut session = AppSession::launch(Cursor::new(buf), HostConfig::default())
        .expect("launch comprehensive session");

    // Send ButtonClick event
    session
        .send_event(UiEvent::ButtonClick {
            component_name: "PlotBtn".to_string(),
        })
        .expect("send click");

    // Collect mutations and apply to AxesModel
    let mut axes_model = AxesModel::new("PlotAxes");
    let start = std::time::Instant::now();
    let mut plot_calls = 0;

    while start.elapsed().as_millis() < 2000 {
        if let Some(notif) = session.poll_notification() {
            match notif {
                HostNotification::Error(e) => {
                    println!("HOST NOTIF ERROR: {e}");
                }
                HostNotification::Mutations(mutations) => {
                    for m in mutations {
                        println!("RECEIVED MUTATION: {m:?}");
                        if let UiMutation::PlotCall {
                            axes,
                            command,
                            args,
                        } = m
                        {
                            if axes == "PlotAxes" || axes == "CurrentAxes" {
                                match command.as_str() {
                                    "hold" => {
                                        axes_model.hold_on = true;
                                    }
                                    "plot" => {
                                        plot_calls += 1;
                                        let to_vec = |v: &rustlab_math::Value| -> Vec<f64> {
                                            match v {
                                                rustlab_math::Value::Double(d) => {
                                                    d.as_slice().to_vec()
                                                }
                                                _ => Vec::new(),
                                            }
                                        };
                                        if args.len() >= 2 {
                                            let x = to_vec(&args[0]);
                                            let y = to_vec(&args[1]);
                                            axes_model.plot(x, y, None);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                other => {
                    println!("OTHER NOTIF: {other:?}");
                }
            }
        }
        if plot_calls >= 2 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    assert_eq!(plot_calls, 2, "Expected 2 plot calls, got {plot_calls}");
    assert!(
        axes_model.hold_on,
        "axes_model.hold_on must be true after hold on"
    );
    assert_eq!(
        axes_model.series.len(),
        2,
        "axes_model must contain 2 series"
    );

    // Verify first series: x=[1, 2, 3], y=[20, 30, 40]
    match &axes_model.series[0] {
        PlotSeries::Line(l) => {
            assert_eq!(l.x, vec![1.0, 2.0, 3.0]);
            assert_eq!(l.y, vec![20.0, 30.0, 40.0]);
        }
        other => panic!("expected LineSeries, got {other:?}"),
    }

    // Verify second series: x=[4, 5], y=[100, 200]
    match &axes_model.series[1] {
        PlotSeries::Line(l) => {
            assert_eq!(l.x, vec![4.0, 5.0]);
            assert_eq!(l.y, vec![100.0, 200.0]);
        }
        other => panic!("expected LineSeries, got {other:?}"),
    }

    session.shutdown();
}

#[test]
fn test_value_changed_callback_receives_event_value_and_app_field() {
    use mlapp_runtime::UiMutation;
    use mlapp_ui::PropertyValue;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let opt = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", opt).unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>EventValueApp</dc:title>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", opt).unwrap();
        let app_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<appdata>
    <UIFigure Name="UIFigure" Position="[100, 100, 640, 480]">
        <NumericEditField Name="FreqField" Value="5.0" Position="20, 400, 100, 22" ValueChangedFcn="FreqFieldValueChanged" />
        <Label Name="OutLabel" Text="Idle" Position="20, 300, 300, 30" />
    </UIFigure>
</appdata>"#;
        zip.write_all(app_xml.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", opt).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef EventValueApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            FreqField
            OutLabel
        end
        methods (Access = private)
            function FreqFieldValueChanged(app, event)
                app.OutLabel.Text = sprintf('App=%d Event=%d', app.FreqField.Value, event.Value);
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let mut session = AppSession::launch(Cursor::new(buf), HostConfig::default())
        .expect("launch event-value session");
    session
        .send_event(UiEvent::ValueChanged {
            component_name: "FreqField".to_string(),
            new_val: PropertyValue::Number(7.0),
        })
        .expect("send value changed");

    let mut saw_expected = false;
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < 2000 {
        while let Some(notif) = session.poll_notification() {
            match notif {
                HostNotification::Mutations(mutations) => {
                    for mutation in mutations {
                        if let UiMutation::SetProperty {
                            component,
                            property,
                            value,
                        } = mutation
                        {
                            if component == "OutLabel"
                                && property == "Text"
                                && value == PropertyValue::String("App=7 Event=7".to_string())
                            {
                                saw_expected = true;
                            }
                        }
                    }
                }
                HostNotification::Error(e) => panic!("worker error: {e}"),
                _ => {}
            }
        }
        if saw_expected {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert!(
        saw_expected,
        "ValueChanged callback must observe both app.FreqField.Value and event.Value"
    );
    session.shutdown();
}

#[test]
fn test_matlab_bridge_delegation_and_unsupported_hint() {
    use mlapp_runtime::UiMutation;
    use mlapp_ui::PropertyValue;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    // Helper to generate a test zip buffer calling svd([1 2; 3 4])
    let build_app_zip = || -> Vec<u8> {
        let mut buf = Vec::new();
        let mut zip = ZipWriter::new(Cursor::new(&mut buf));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", options).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", options)
            .unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties">
    <dc:title>BridgeApp</dc:title>
    <cp:version>1.0.0</cp:version>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", options).unwrap();
        let app_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<appdata>
    <UIFigure Name="UIFigure" Position="[100, 100, 640, 480]">
        <Label Name="ResultLabel" Text="Waiting" Position="20, 300, 200, 30" />
        <Button Name="BridgeBtn" Text="Run SVD" Position="20, 200, 100, 30" ButtonPushedFcn="BridgeBtnPushed" />
    </UIFigure>
</appdata>"#;
        zip.write_all(app_xml.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", options).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef BridgeApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            ResultLabel
            BridgeBtn
        end
        methods (Access = public)
            function BridgeBtnPushed(app, event)
                res = fminsearch([0, 0]);
                app.ResultLabel.Text = sprintf('x=%.1f,y=%.1f', res(1), res(2));
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
        buf
    };

    // Case 1: Without --allow-matlab, calling fminsearch fails with a clear hint
    {
        let zip_bytes = build_app_zip();
        let config = HostConfig {
            allow_matlab: false,
            ..Default::default()
        };
        let mut session =
            AppSession::launch(Cursor::new(zip_bytes), config).expect("launch session");

        session
            .send_event(UiEvent::ButtonClick {
                component_name: "BridgeBtn".to_string(),
            })
            .expect("send button click");

        let mut saw_hint_error = false;
        for _ in 0..100 {
            while let Some(notif) = session.poll_notification() {
                if let HostNotification::Error(e) = notif {
                    if e.contains("Unsupported MATLAB function 'fminsearch'")
                        && e.contains("run with --allow-matlab to delegate to real MATLAB")
                    {
                        saw_hint_error = true;
                    }
                }
            }
            if saw_hint_error {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            saw_hint_error,
            "Calling unsupported function without --allow-matlab must fail with delegation hint"
        );
        session.shutdown();
    }

    // Case 2: With --allow-matlab, calling fminsearch succeeds via the bridge
    {
        let zip_bytes = build_app_zip();
        let config = HostConfig {
            allow_matlab: true,
            ..Default::default()
        };
        let mut session =
            AppSession::launch(Cursor::new(zip_bytes), config).expect("launch session");

        session
            .send_event(UiEvent::ButtonClick {
                component_name: "BridgeBtn".to_string(),
            })
            .expect("send button click");

        let mut saw_success_mutation = false;
        for _ in 0..100 {
            while let Some(notif) = session.poll_notification() {
                if let HostNotification::Mutations(mutations) = notif {
                    for mutation in mutations {
                        if let UiMutation::SetProperty {
                            component,
                            property,
                            value,
                        } = mutation
                        {
                            if component == "ResultLabel"
                                && property == "Text"
                                && value == PropertyValue::String("x=1.0,y=2.0".to_string())
                            {
                                saw_success_mutation = true;
                            }
                        }
                    }
                }
            }
            if saw_success_mutation {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            saw_success_mutation,
            "Calling unsupported function with --allow-matlab must delegate to MATLAB bridge and update UI"
        );
        session.shutdown();
    }
}

#[test]
fn test_simulink_app_designer_integration() {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", opt).unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>SimulinkApp</dc:title>
    <dc:creator>TestRunner</dc:creator>
    <cp:version>1.0.0</cp:version>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", opt).unwrap();
        let app = r#"<?xml version="1.0" encoding="utf-8"?>
<appdata>
  <UIFigure Name="UIFigure" Position="[100, 100, 800, 600]">
    <Slider Name="GainSlider" Value="2.0" Position="50, 480, 200, 22" />
    <Button Name="RunBtn" Text="Run Simulation" Position="50, 420, 120, 30" ButtonPushedFcn="RunBtnPushed" />
    <Label Name="StatusLabel" Text="Ready" Position="50, 380, 200, 22" />
    <UIAxes Name="SimAxes" Position="300, 50, 450, 450" />
  </UIFigure>
</appdata>"#;
        zip.write_all(app.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", opt).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef SimulinkApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            GainSlider
            RunBtn
            StatusLabel
            SimAxes
        end
        methods (Access = private)
            function startupFcn(app)
                load_system('simple_feedback');
            end
            function RunBtnPushed(app, event)
                set_param('simple_feedback/Gain', 'Gain', num2str(app.GainSlider.Value));
                simOut = sim('simple_feedback', 'StopTime', '5.0', 'FixedStep', '0.01');
                plot(app.SimAxes, simOut.tout, simOut.yout);
                app.StatusLabel.Text = sprintf('Simulation Finished (Gain=%.1f)', app.GainSlider.Value);
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let host_cfg = HostConfig {
        allow_file_io: true,
        ..HostConfig::default()
    };
    let mut session = AppSession::launch(Cursor::new(buf), host_cfg).expect("launch session");

    // Test across 3 different gain settings: 1.0, 3.5, 7.0
    for gain_val in [1.0, 3.5, 7.0] {
        session
            .send_event(UiEvent::ValueChanged {
                component_name: "GainSlider".to_string(),
                new_val: PropertyValue::Number(gain_val),
            })
            .expect("update slider");

        session
            .send_event(UiEvent::ButtonClick {
                component_name: "RunBtn".to_string(),
            })
            .expect("click run");

        let mut saw_finished = false;
        let mut saw_plot = false;
        let mut plot_len = 0;

        let expected_status = format!("Simulation Finished (Gain={gain_val:.1})");
        for _ in 0..100 {
            while let Some(notif) = session.poll_notification() {
                if let HostNotification::Mutations(mutations) = notif {
                    for m in mutations {
                        match m {
                            UiMutation::SetProperty {
                                component,
                                property,
                                value,
                            } => {
                                if component == "StatusLabel"
                                    && property == "Text"
                                    && value == PropertyValue::String(expected_status.clone())
                                {
                                    saw_finished = true;
                                }
                            }
                            UiMutation::PlotCall { axes, args, .. }
                                if axes == "SimAxes" && args.len() == 2 =>
                            {
                                saw_plot = true;
                                if let rustlab_math::Value::Double(d) = &args[0] {
                                    plot_len = d.as_slice().len();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            if saw_finished && saw_plot {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(
            saw_finished,
            "Expected 'Simulation Finished' status update for gain {gain_val}"
        );
        assert!(
            saw_plot,
            "Expected plot command emitted for SimAxes for gain {gain_val}"
        );
        assert_eq!(
            plot_len, 501,
            "Expected 501 points logged across 5.0 seconds with step 0.01"
        );
    }

    session.shutdown();
}

#[test]
fn test_simulink_error_paths() {
    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(Cursor::new(&mut buf));
        let opt = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("[Content_Types].xml", opt).unwrap();
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><Types></Types>")
            .unwrap();

        zip.start_file("metadata/coreProperties.xml", opt).unwrap();
        let core = r#"<?xml version="1.0" encoding="UTF-8"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>SimErrorApp</dc:title>
    <dc:creator>TestRunner</dc:creator>
    <cp:version>1.0.0</cp:version>
</cp:coreProperties>"#;
        zip.write_all(core.as_bytes()).unwrap();

        zip.start_file("appdesigner/app.xml", opt).unwrap();
        let app = r#"<?xml version="1.0" encoding="utf-8"?>
<appdata>
  <UIFigure Name="UIFigure" Position="[100, 100, 800, 600]">
    <Button Name="MissingModelBtn" Text="Test Missing" Position="50, 420, 120, 30" ButtonPushedFcn="MissingModelBtnPushed" />
    <Button Name="InvalidParamBtn" Text="Test Invalid" Position="50, 350, 120, 30" ButtonPushedFcn="InvalidParamBtnPushed" />
  </UIFigure>
</appdata>"#;
        zip.write_all(app.as_bytes()).unwrap();

        zip.start_file("matlab/document.xml", opt).unwrap();
        let doc_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <![CDATA[
    classdef SimErrorApp < matlab.apps.AppBase
        properties (Access = public)
            UIFigure
            MissingModelBtn
            InvalidParamBtn
        end
        methods (Access = private)
            function MissingModelBtnPushed(app, event)
                sim('non_existent_system_model_12345');
            end
            function InvalidParamBtnPushed(app, event)
                set_param('non_existent_system_model_12345/Gain', 'Gain', '10');
            end
        end
    end
    ]]>
</w:document>"#;
        zip.write_all(doc_xml.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    let mut session =
        AppSession::launch(Cursor::new(buf), HostConfig::default()).expect("launch session");

    // Test 1: Missing model returns error notification
    session
        .send_event(UiEvent::ButtonClick {
            component_name: "MissingModelBtn".to_string(),
        })
        .expect("click missing model button");

    let mut saw_missing_err = false;
    for _ in 0..100 {
        while let Some(notif) = session.poll_notification() {
            if let HostNotification::Error(e) = notif {
                if e.contains("Simulink model 'non_existent_system_model_12345' not found") {
                    saw_missing_err = true;
                }
            }
        }
        if saw_missing_err {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        saw_missing_err,
        "Simulating missing model must emit clear error notification"
    );

    // Test 2: Invalid parameter on missing model returns error notification
    session
        .send_event(UiEvent::ButtonClick {
            component_name: "InvalidParamBtn".to_string(),
        })
        .expect("click invalid param button");

    let mut saw_param_err = false;
    for _ in 0..100 {
        while let Some(notif) = session.poll_notification() {
            if let HostNotification::Error(e) = notif {
                if e.contains("Simulink model 'non_existent_system_model_12345' not loaded") {
                    saw_param_err = true;
                }
            }
        }
        if saw_param_err {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        saw_param_err,
        "set_param on missing model must emit clear error notification"
    );

    session.shutdown();
}
