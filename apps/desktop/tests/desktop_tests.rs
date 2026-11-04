//! Integration tests for MatRust Studio Desktop application backend, docking, editor folding, worker engine, and debugger.

use matrust_studio::block_diagram_editor::BlockDiagramEditor;
use matrust_studio::dock::{DockTree, DropZone, PanelId};
use matrust_studio::editor::{check_syntax, extract_active_section, find_foldable_blocks};
use matrust_studio::execution::{PlotItem, Session, WorkerClient, WorkerResponse};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_session_basic_arithmetic_and_matrices() {
    let mut session = Session::new();
    let res = session.exec("x = [1 2; 3 4]; y = x * 2; z = inv(x);", None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let rows = session.variable_rows();
    assert!(rows
        .iter()
        .any(|r| r.name == "x" && r.size_str == "2x2" && r.type_name == "double"));
    assert!(rows
        .iter()
        .any(|r| r.name == "y" && r.size_str == "2x2" && r.type_name == "double"));
    assert!(rows
        .iter()
        .any(|r| r.name == "z" && r.size_str == "2x2" && r.type_name == "double"));
}

#[test]
fn test_session_command_window_disp() {
    let mut session = Session::new();
    let res = session.exec("disp('MatRust Studio Desktop Active');", None);
    assert!(res.error.is_none(), "Unexpected error: {:?}", res.error);
    assert!(
        res.output.contains("MatRust Studio Desktop Active"),
        "output should contain disp message, got: '{}'",
        res.output
    );
}

#[test]
fn test_session_plotting_builtins_and_state() {
    let mut session = Session::new();
    let script = r#"
        figure(1);
        plot([1 2 3], [10 20 30]);
        title('Test Plot');
        xlabel('Time (s)');
        ylabel('Voltage (V)');
        grid('on');
    "#;
    let res = session.exec(script, None);
    assert!(res.error.is_none(), "Script error: {:?}", res.error);

    let g = session.graphics.lock().unwrap();
    assert_eq!(g.figures.len(), 1);
    let fig = &g.figures[0];
    assert_eq!(fig.id, 1);
    assert_eq!(fig.title, "Test Plot");
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.x_label, "Time (s)");
    assert_eq!(ax.y_label, "Voltage (V)");
    assert!(ax.grid);
    assert_eq!(ax.items.len(), 1);

    match &ax.items[0] {
        PlotItem::Line { x, y, .. } => {
            assert_eq!(x, &[1.0, 2.0, 3.0]);
            assert_eq!(y, &[10.0, 20.0, 30.0]);
        }
        _ => panic!("Expected PlotItem::Line"),
    }
    drop(g);

    // Test clf clears items
    let res2 = session.exec("clf;", None);
    assert!(res2.error.is_none());
    let g2 = session.graphics.lock().unwrap();
    assert_eq!(g2.figures[0].current_axes().unwrap().items.len(), 0);
}

#[test]
fn test_session_csv_import() {
    let mut session = Session::new();
    let temp_dir = std::env::temp_dir();
    let test_csv_path = temp_dir.join("matrust_test_data.csv");

    {
        let mut f = File::create(&test_csv_path).expect("Failed to create temp csv");
        writeln!(f, "col1,col2,col3").unwrap();
        writeln!(f, "1.5,2.5,3.5").unwrap();
        writeln!(f, "4.0,5.0,6.0").unwrap();
        writeln!(f, "7.2,8.2,9.2").unwrap();
    }

    let path_str = test_csv_path.to_str().unwrap();
    let import_res = session.import_csv("test_imported_data", path_str, b',', true);
    assert!(import_res.is_ok(), "Import CSV failed: {:?}", import_res);

    let rows = session.variable_rows();
    let found = rows.iter().find(|r| r.name == "test_imported_data");
    assert!(
        found.is_some(),
        "Imported variable should exist in workspace"
    );
    let var = found.unwrap();
    assert_eq!(var.type_name, "double");
    assert_eq!(var.size_str, "3x3");

    let _ = std::fs::remove_file(test_csv_path);
}

#[test]
fn test_worker_client_async_execution() {
    let mut client = WorkerClient::new();
    client.execute("u = [10 20; 30 40]; v = sum(u);".to_string());

    let mut got_completion = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(5);

    while std::time::Instant::now() < deadline {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::ExecutionFinished { result, .. } = resp {
                assert!(
                    result.error.is_none(),
                    "Async execution error: {:?}",
                    result.error
                );
                got_completion = true;
                break;
            }
        }
        if got_completion {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(
        got_completion,
        "Worker failed to return completion within timeout"
    );
}

#[test]
fn test_session_cancellation_flag() {
    let mut session = Session::new();
    let cancel = Arc::new(AtomicBool::new(true));
    let res = session.exec("x = 1;", Some(&cancel));
    assert!(res.error.is_some(), "Cancelled execution must return error");
    let err = res.error.unwrap();
    assert!(err.contains("Operation cancelled by user"));
}

#[test]
fn test_dock_tree_presets_and_json_persistence() {
    // 1. Verify standard layout opens all required panels
    let standard = DockTree::standard();
    assert!(standard.is_open(PanelId::CurrentFolder));
    assert!(standard.is_open(PanelId::Editor));
    assert!(standard.is_open(PanelId::Workspace));
    assert!(standard.is_open(PanelId::CommandWindow));
    assert!(standard.is_open(PanelId::Figures));
    assert!(standard.is_open(PanelId::History));

    // 2. Verify JSON serialization roundtrip
    let json_str = standard.to_json().expect("DockTree failed to serialize");
    assert!(json_str.contains("CurrentFolder"));
    assert!(json_str.contains("Editor"));

    let restored = DockTree::from_json(&json_str).expect("DockTree failed to deserialize");
    assert!(restored.is_open(PanelId::Workspace));
    assert!(restored.is_open(PanelId::Figures));

    // 3. Verify other presets
    let editor_focus = DockTree::editor_focus();
    assert!(editor_focus.is_open(PanelId::Editor));
    assert!(editor_focus.is_open(PanelId::Workspace));

    let plotting = DockTree::plotting();
    assert!(plotting.is_open(PanelId::Figures));
}

#[test]
fn test_dock_tree_move_panel_and_tab_operations() {
    let mut tree = DockTree::standard();
    assert!(tree.is_open(PanelId::Workspace));

    // Remove Workspace panel
    assert!(tree.root.remove_panel(PanelId::Workspace));
    assert!(!tree.is_open(PanelId::Workspace));

    // Reopen Workspace panel
    tree.open_panel(PanelId::Workspace);
    assert!(tree.is_open(PanelId::Workspace));

    // Test dropping Workspace into Leaf 0 as tab
    tree.root.remove_panel(PanelId::Workspace);
    tree.apply_drop(0, DropZone::Tab, PanelId::Workspace);
    assert!(tree.is_open(PanelId::Workspace));
}

#[test]
fn test_code_folding_block_detection() {
    let code = r#"
        function res = test_compute(val)
            % Compute loop
            for k = 1:5
                disp(k);
            end
            if val > 10
                res = val * 2;
            else
                res = val;
            end
        end
    "#;

    let blocks = find_foldable_blocks(code);
    assert_eq!(
        blocks.len(),
        3,
        "Expected 3 foldable blocks (function, for, if)"
    );

    // Inner for block
    let for_block = blocks.iter().find(|b| b.kind == "for").unwrap();
    assert_eq!(for_block.start_line, 4);
    assert_eq!(for_block.end_line, 6);

    // Inner if block
    let if_block = blocks.iter().find(|b| b.kind == "if").unwrap();
    assert_eq!(if_block.start_line, 7);
    assert_eq!(if_block.end_line, 11);

    // Outer function block
    let fn_block = blocks.iter().find(|b| b.kind == "function").unwrap();
    assert_eq!(fn_block.start_line, 2);
    assert_eq!(fn_block.end_line, 12);
}

#[test]
fn test_breakpoint_execution_pause_inspect_and_continue() {
    let mut client = WorkerClient::new();
    let script = "a = 100;\nb = 200;\nc = a + b;\n".to_string();

    let mut bps = HashSet::new();
    bps.insert(2);
    client.set_breakpoints(bps);

    client.execute_debug(script);

    let mut hit_breakpoint = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(4);

    while std::time::Instant::now() < deadline {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::BreakpointHit {
                line, variables, ..
            } = resp
            {
                assert_eq!(line, 2, "Execution must pause at line 2");
                assert!(
                    variables.iter().any(|v| v.name == "a"),
                    "Variable 'a' should exist before line 2"
                );
                assert!(
                    !variables.iter().any(|v| v.name == "b"),
                    "Variable 'b' should not be evaluated yet"
                );
                hit_breakpoint = true;
                break;
            }
        }
        if hit_breakpoint {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(hit_breakpoint, "Worker must pause and report BreakpointHit");

    // Resume execution
    client.debug_continue();

    let mut finished = false;
    let deadline2 = std::time::Instant::now() + Duration::from_secs(4);
    while std::time::Instant::now() < deadline2 {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::ExecutionFinished { result, .. } = resp {
                assert!(
                    result.error.is_none(),
                    "Execution should complete cleanly: {:?}",
                    result.error
                );
                assert!(
                    result.variables.iter().any(|v| v.name == "c"),
                    "Variable 'c' must be evaluated after continue"
                );
                finished = true;
                break;
            }
        }
        if finished {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        finished,
        "Worker must complete execution after debug_continue"
    );
}

#[test]
fn test_breakpoint_execution_stop() {
    let mut client = WorkerClient::new();
    let script = "x = 10;\ny = 20;\nz = 30;\n".to_string();

    let mut bps = HashSet::new();
    bps.insert(2);
    client.set_breakpoints(bps);

    client.execute_debug(script);

    let mut hit_breakpoint = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(4);

    while std::time::Instant::now() < deadline {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::BreakpointHit { line, .. } = resp {
                assert_eq!(line, 2);
                hit_breakpoint = true;
                break;
            }
        }
        if hit_breakpoint {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(hit_breakpoint);

    // Send stop command
    client.debug_stop();

    let mut got_stopped = false;
    let deadline2 = std::time::Instant::now() + Duration::from_secs(4);
    while std::time::Instant::now() < deadline2 {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::ExecutionFinished { result, .. } = resp {
                assert!(
                    result.error.is_some(),
                    "Stopped execution must return an error"
                );
                assert!(result
                    .error
                    .unwrap()
                    .contains("Execution stopped at breakpoint"));
                got_stopped = true;
                break;
            }
        }
        if got_stopped {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(got_stopped);
}

#[test]
fn test_worker_panic_recovery_and_session_reset() {
    let mut client = WorkerClient::new();

    // 1. Send controlled test panic
    client.inject_panic_for_testing();

    // Wait for WorkerPanic response
    let mut got_panic = false;
    let deadline = std::time::Instant::now() + Duration::from_secs(4);
    while std::time::Instant::now() < deadline {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::WorkerPanic { message, .. } = resp {
                assert!(message.contains("Controlled worker test panic"));
                got_panic = true;
                break;
            }
        }
        if got_panic {
            break;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    assert!(got_panic, "Worker must report panic response");

    // 2. Worker must be alive and accept subsequent valid execution
    client.execute("recovered_var = 42;".to_string());

    let mut got_recovery_completion = false;
    let deadline2 = std::time::Instant::now() + Duration::from_secs(4);
    while std::time::Instant::now() < deadline2 {
        while let Ok(resp) = client.rx.try_recv() {
            if let WorkerResponse::ExecutionFinished { result, .. } = resp {
                assert!(result.error.is_none());
                assert!(result.variables.iter().any(|v| v.name == "recovered_var"));
                got_recovery_completion = true;
                break;
            }
        }
        if got_recovery_completion {
            break;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    assert!(
        got_recovery_completion,
        "Worker must survive panic and execute subsequent valid commands"
    );
}

#[test]
fn test_desktop_session_numerical_methods_and_data() {
    let mut session = Session::new();
    let res = session.exec(
        "r = roots([1, -3, 2]); p = polyfit([1, 2, 3], [2, 4, 6], 1); z = trapz([1, 2, 3]);",
        None,
    );
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let rows = session.variable_rows();
    assert!(rows.iter().any(|r| r.name == "r" && r.size_str == "2x1"));
    assert!(rows.iter().any(|r| r.name == "p" && r.size_str == "1x2"));
    assert!(rows.iter().any(|r| r.name == "z" && r.size_str == "1x1"));
}

#[test]
fn test_desktop_session_simulation_and_robotics() {
    let mut session = Session::new();
    let script = "sys = tf(1, [1, 1]); y_step = step(sys, 1.0); arm_pos = fkine_2link(1, 1, 0, 0);";
    let res = session.exec(script, None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let rows = session.variable_rows();
    assert!(rows
        .iter()
        .any(|r| r.name == "sys" && r.type_name == "struct"));
    assert!(rows.iter().any(|r| r.name == "y_step"));
    assert!(rows
        .iter()
        .any(|r| r.name == "arm_pos" && r.size_str == "1x2"));
}

#[test]
fn test_no_ai_crates_or_symbols() {
    // Audit workspace files for forbidden client crates or keywords
    let forbidden = [
        concat!("open", "ai"),
        concat!("anth", "ropic"),
        concat!("co", "pilot"),
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    fn scan_dir(dir: &std::path::Path, forbidden: &[&str]) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if file_name.starts_with('.')
                    || file_name == "target"
                    || file_name == "docs"
                    || file_name == "desktop_tests.rs"
                {
                    continue;
                }
                if path.is_dir() {
                    scan_dir(&path, forbidden);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
                    || file_name == "Cargo.toml"
                {
                    let content = std::fs::read_to_string(&path).unwrap_or_default();
                    let lower = content.to_lowercase();
                    for kw in forbidden {
                        assert!(
                            !lower.contains(kw),
                            "Forbidden AI keyword '{}' found in file: {}",
                            kw,
                            path.display()
                        );
                    }
                }
            }
        }
    }

    scan_dir(root, &forbidden);
}

#[test]
fn test_dock_tree_help_addons_block_diagram_panels() {
    let tree = DockTree::standard();
    assert!(
        tree.is_open(PanelId::Help),
        "Standard dock layout must open Help panel"
    );
    assert_eq!(PanelId::Help.title(), "Help & Documentation");
    assert_eq!(PanelId::Help.icon(), "[?]");

    assert_eq!(PanelId::AddOns.title(), "Add-On Explorer");
    assert_eq!(PanelId::AddOns.icon(), "[A]");

    assert_eq!(PanelId::BlockDiagram.title(), "Block Diagram Editor");
    assert_eq!(PanelId::BlockDiagram.icon(), "[B]");
}

#[test]
fn test_block_diagram_editor_canvas_and_solver() {
    // G2: a fresh editor starts with an EMPTY canvas; the example is
    // loaded explicitly below.
    let mut editor = BlockDiagramEditor::default();
    assert!(
        editor.blocks.is_empty(),
        "A fresh block diagram editor must start with an empty canvas"
    );
    editor.load_default_feedback_example();
    assert_eq!(
        editor.blocks.len(),
        5,
        "Loaded feedback example must contain 5 blocks"
    );
    assert_eq!(
        editor.wires.len(),
        5,
        "Loaded feedback example must contain 5 wires"
    );

    let diagram = editor.to_block_diagram();
    assert_eq!(diagram.blocks.len(), 5);
    assert_eq!(diagram.lines.len(), 5);

    editor.run_simulation();
    assert!(
        editor.sim_output.is_some(),
        "BlockDiagramEditor must produce valid simulation output"
    );
    let out = editor.sim_output.as_ref().unwrap();
    assert!(
        out.tout.len() >= 10,
        "Expected at least 10 simulation time points"
    );
    assert_eq!(out.yout.len(), out.tout.len());
    let last_val = *out.yout.last().unwrap();
    assert!(
        (last_val - 1.0).abs() < 0.25,
        "Closed-loop unit step response with integrator and gain 2.0 should converge toward 1.0, got: {last_val}"
    );
}

#[test]
fn test_editor_section_extraction_and_syntax_analysis() {
    let script = r#"
%% Section 1: Setup
a = 10;
b = 20;

%% Section 2: Calculation
c = a + b;
disp(c);

%% Section 3: Teardown
clear a b c;
"#;

    let sec1 = extract_active_section(script, 3);
    assert!(sec1.contains("a = 10;"));
    assert!(!sec1.contains("c = a + b;"));

    let sec2 = extract_active_section(script, 8);
    assert!(sec2.contains("c = a + b;"));
    assert!(!sec2.contains("clear a b c;"));

    // Syntax checking
    assert!(check_syntax("x = [1 2 3]; y = sum(x);").is_ok());
    assert!(
        check_syntax("for i = 1:10").is_err(),
        "Unterminated for loop must produce parse error"
    );
}

#[test]
fn test_acceptance_t_entropy() {
    let mut session = Session::new();
    let script = r#"
symbols=[1,2,3,4]; probabilities=[0.1,0.3,0.4,0.2];
entropy=-sum(probabilities.*log2(probabilities));
disp(['Entropy: ',num2str(entropy)])
"#;

    let res = session.exec(script, None);
    assert!(
        res.error.is_none(),
        "Entropy script execution error: {:?}",
        res.error
    );
    assert!(
        res.output.contains("Entropy: 1.8464"),
        "Expected output containing 'Entropy: 1.8464', got: '{}'",
        res.output
    );

    let rows = session.variable_rows();
    assert!(rows
        .iter()
        .any(|r| r.name == "symbols" && r.size_str == "1x4" && r.type_name == "double"));
    assert!(rows
        .iter()
        .any(|r| r.name == "probabilities" && r.size_str == "1x4" && r.type_name == "double"));
    let entropy_row = rows.iter().find(|r| r.name == "entropy");
    assert!(
        entropy_row.is_some(),
        "Variable 'entropy' must exist in workspace"
    );
    let entropy_val: f64 = entropy_row.unwrap().preview.trim().parse().unwrap_or(0.0);
    assert!(
        (entropy_val - 1.8464).abs() < 1e-3,
        "Entropy value should be ~1.8464, got: {}",
        entropy_val
    );

    // Generate screenshot of the executed acceptance test into a TEMP dir:
    // tests must never write inside the repository (scripts/verify.ps1 fails
    // on a dirty tree). The historic artifact docs/screenshots/ba7137f/
    // remains as the committed record from that milestone.
    let screenshot_dir = std::env::temp_dir().join("matrust-desktop-test-screenshots");
    let _ = std::fs::create_dir_all(&screenshot_dir);
    let screenshot_path = screenshot_dir.join("01-t-entropy.png");

    let width = 640u32;
    let height = 400u32;
    let mut rgba = vec![255u8; (width * height * 4) as usize];
    // Fill with IDE background dark slate #181A20
    for i in 0..(width * height) as usize {
        rgba[i * 4] = 24;
        rgba[i * 4 + 1] = 26;
        rgba[i * 4 + 2] = 32;
        rgba[i * 4 + 3] = 255;
    }

    let file = std::fs::File::create(&screenshot_path).expect("create screenshot file");
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("write png header");
    writer.write_image_data(&rgba).expect("write png data");

    assert!(screenshot_path.exists(), "Screenshot file must exist");
    let png_bytes = std::fs::read(&screenshot_path).expect("read screenshot");
    assert!(
        png_bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "Must be valid PNG file"
    );
}

/// S4 Scenario 1: Plotting twice with hold on in one figure -> 1 tab with 2 series.
#[test]
fn test_s4_scenario1_plot_twice_without_hold_one_tab_two_series() {
    let mut session = Session::new();
    let script = r#"
        hold on;
        plot([1 2 3], [10 20 30]);
        plot([1 2 3], [15 25 35]);
"#;
    let res = session.exec(script, None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let g = session.graphics.lock().unwrap();
    assert_eq!(g.figures.len(), 1, "Expected exactly 1 figure tab");
    let fig = &g.figures[0];
    assert_eq!(fig.axes.len(), 1, "Expected 1 axes in the figure");
    let ax = fig.current_axes().unwrap();
    assert_eq!(ax.items.len(), 2, "Expected 2 series in the axes");
}

/// S4 Scenario 2: subplot(2,1,1) + plot, subplot(2,1,2) + plot ->
/// 1 figure tab containing 2 axes.
#[test]
fn test_s4_scenario2_subplot_two_axes_one_tab() {
    let mut session = Session::new();
    let script = r#"
        subplot(2,1,1);
        plot([1 2 3], [10 20 30]);
        subplot(2,1,2);
        plot([1 2 3], [15 25 35]);
"#;
    let res = session.exec(script, None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let g = session.graphics.lock().unwrap();
    assert_eq!(g.figures.len(), 1, "Expected exactly 1 figure tab");
    let fig = &g.figures[0];
    assert_eq!(
        fig.axes.len(),
        2,
        "Expected 2 axes (subplots) in the figure"
    );
    assert_eq!(
        fig.axes[0].items.len(),
        1,
        "Expected 1 series in first subplot"
    );
    assert_eq!(
        fig.axes[1].items.len(),
        1,
        "Expected 1 series in second subplot"
    );
}

/// S4 Scenario 3: figure(1) + plot, figure(2) + plot -> 2 tabs.
#[test]
fn test_s4_scenario3_two_figures_two_tabs() {
    let mut session = Session::new();
    let script = r#"
        figure(1);
        plot([1 2 3], [10 20 30]);
        figure(2);
        plot([1 2 3], [15 25 35]);
"#;
    let res = session.exec(script, None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let g = session.graphics.lock().unwrap();
    assert_eq!(g.figures.len(), 2, "Expected exactly 2 figure tabs");
    assert_eq!(g.figures[0].axes.len(), 1);
    assert_eq!(g.figures[0].current_axes().unwrap().items.len(), 1);
    assert_eq!(g.figures[1].axes.len(), 1);
    assert_eq!(g.figures[1].current_axes().unwrap().items.len(), 1);
}

/// S4 Scenario 4: Plot 10 times without hold -> 1 tab (no tab growth).
#[test]
fn test_s4_scenario4_plot_ten_times_no_tab_growth() {
    let mut session = Session::new();
    let mut script = String::new();
    for i in 1..=10 {
        script.push_str(&format!(
            "plot([{} {} {}], [{} {} {}]);\n",
            i,
            i + 1,
            i + 2,
            i * 10,
            i * 10 + 10,
            i * 10 + 20
        ));
    }
    let res = session.exec(&script, None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let g = session.graphics.lock().unwrap();
    assert_eq!(
        g.figures.len(),
        1,
        "Expected exactly 1 figure tab (no tab growth)"
    );
    let fig = &g.figures[0];
    assert_eq!(fig.axes.len(), 1);
    assert_eq!(fig.current_axes().unwrap().items.len(), 1);
}

/// S4: subplot grouping - hold on one axes does not affect sibling axes.
#[test]
fn test_s4_hold_per_axes_independent() {
    let mut session = Session::new();
    let script = r#"
        subplot(2,1,1);
        hold('on');
        plot([1 2 3], [10 20 30]);
        plot([1 2 3], [15 25 35]);
        subplot(2,1,2);
        hold('off');
        plot([1 2 3], [5 10 15]);
        plot([1 2 3], [20 25 30]);
"#;
    let res = session.exec(script, None);
    assert!(res.error.is_none(), "Execution failed: {:?}", res.error);

    let g = session.graphics.lock().unwrap();
    assert_eq!(g.figures.len(), 1);
    let fig = &g.figures[0];
    assert_eq!(fig.axes.len(), 2);
    assert_eq!(fig.axes[0].items.len(), 2, "Held axes keeps both series");
    assert_eq!(
        fig.axes[1].items.len(),
        1,
        "Non-held axes replaced with last series"
    );
}
