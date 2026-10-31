//! Integration tests for MatRust Studio Desktop application backend, docking, editor folding, worker engine, and debugger.

use matrust_studio::dock::{DockTree, DropZone, PanelId};
use matrust_studio::editor::find_foldable_blocks;
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
    assert_eq!(fig.x_label, "Time (s)");
    assert_eq!(fig.y_label, "Voltage (V)");
    assert!(fig.grid);
    assert_eq!(fig.items.len(), 1);

    match &fig.items[0] {
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
    assert_eq!(g2.figures[0].items.len(), 0);
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
