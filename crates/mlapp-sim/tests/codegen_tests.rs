//! Integration tests for Milestone M11: ISO C99 Code Generation.

use mlapp_sim::codegen::{generate_c, CodegenConfig};
use mlapp_sim::model::{Block, BlockDiagram, BlockType};

#[test]
fn test_codegen_signal_model_allocation_free() {
    let mut diagram = BlockDiagram::new("test_feedback");
    let mut step = Block::new("1", "Step", BlockType::Step);
    step.parameters
        .insert("Time".to_string(), "0.0".to_string());
    step.parameters
        .insert("After".to_string(), "1.0".to_string());

    let mut sum = Block::new("2", "Sum", BlockType::Sum);
    sum.parameters
        .insert("Inputs".to_string(), "+-".to_string());

    let mut gain = Block::new("3", "Gain", BlockType::Gain);
    gain.parameters
        .insert("Gain".to_string(), "2.0".to_string());

    let intg = Block::new("4", "Integrator", BlockType::Integrator);
    let scope = Block::new("5", "Scope", BlockType::Scope);

    diagram.blocks = vec![step, sum, gain, intg, scope];
    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);
    diagram.add_line("4", 1, "5", 1);
    diagram.add_line("4", 1, "2", 2);

    let config = CodegenConfig {
        model_name: "feedback_sys".to_string(),
        include_main: true,
        step_size: Some(0.001),
        stop_time: Some(1.0),
    };

    let generated = generate_c(&diagram, &config).expect("Codegen should succeed for clean model");

    assert_eq!(generated.model_name, "feedback_sys");

    // Header checks
    assert!(generated.header.contains("#ifndef FEEDBACK_SYS_H"));
    assert!(generated.header.contains("FEEDBACK_SYS_NUM_BLOCKS 5U"));
    assert!(generated
        .header
        .contains("void feedback_sys_init(feedback_sys_state_t* state);"));
    assert!(generated
        .header
        .contains("void feedback_sys_step(feedback_sys_state_t* state, double dt);"));

    // Source checks
    assert!(generated.source.contains("#include \"feedback_sys.h\""));
    assert!(generated
        .source
        .contains("void feedback_sys_step(feedback_sys_state_t* state, double dt)"));

    // Verify STRICT allocation-free requirement (0 occurrences of dynamic allocation in generated C)
    assert!(!generated.source.contains("malloc"));
    assert!(!generated.source.contains("calloc"));
    assert!(!generated.source.contains("realloc"));
    assert!(!generated.source.contains("free("));

    // Main runner checks
    let runner = generated
        .main_runner
        .expect("Main runner should be included");
    assert!(runner.contains("int main(int argc, char** argv)"));
    assert!(runner.contains("feedback_sys_init(&state);"));
    assert!(runner.contains("feedback_sys_step(&state, dt);"));
}

#[test]
fn test_codegen_circuit_mna_solver() {
    let mut diagram = BlockDiagram::new("rc_circuit");
    let mut vsrc = Block::new("1", "V1", BlockType::DcVoltageSource);
    vsrc.parameters
        .insert("Voltage".to_string(), "10.0".to_string());

    let mut r1 = Block::new("2", "R1", BlockType::Resistor);
    r1.parameters
        .insert("Resistance".to_string(), "1000.0".to_string());

    let mut c1 = Block::new("3", "C1", BlockType::Capacitor);
    c1.parameters
        .insert("Capacitance".to_string(), "1e-6".to_string());

    let gnd = Block::new("4", "GND", BlockType::Ground);
    let vsensor = Block::new("5", "V_Sense", BlockType::VoltageSensor);

    diagram.blocks = vec![vsrc, r1, c1, gnd, vsensor];
    // V1(+) to R1(p)
    diagram.add_electrical_line("1", 0, "2", 0);
    // R1(n) to C1(p) and V_Sense(p)
    diagram.add_electrical_line("2", 1, "3", 0);
    diagram.add_electrical_line("2", 1, "5", 0);
    // C1(n), V1(-), V_Sense(n) to GND
    diagram.add_electrical_line("3", 1, "4", 0);
    diagram.add_electrical_line("1", 1, "4", 0);
    diagram.add_electrical_line("5", 1, "4", 0);

    let config = CodegenConfig {
        model_name: "rc_filter".to_string(),
        include_main: true,
        step_size: Some(0.0001),
        stop_time: Some(0.01),
    };

    let generated = generate_c(&diagram, &config).expect("Circuit codegen should succeed");

    assert!(generated.header.contains("#define RC_FILTER_HAS_CIRCUIT 1"));
    assert!(generated.header.contains("RC_FILTER_MNA_DIM"));
    assert!(generated.source.contains("solve_mna_system"));
    assert!(generated
        .source
        .contains("void rc_filter_step(rc_filter_state_t* state, double dt)"));

    // Verify STRICT allocation-free in circuit C code
    assert!(!generated.source.contains("malloc"));
    assert!(!generated.source.contains("calloc"));
    assert!(!generated.source.contains("free("));
}

#[test]
fn test_codegen_validation_failure_on_error_diagram() {
    let mut diagram = BlockDiagram::new("algebraic_loop_err");
    let gain1 = Block::new("1", "Gain1", BlockType::Gain);
    let gain2 = Block::new("2", "Gain2", BlockType::Gain);

    diagram.blocks = vec![gain1, gain2];
    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "1", 1);

    let config = CodegenConfig::default();
    let err =
        generate_c(&diagram, &config).expect_err("Codegen must reject model with algebraic loop");
    assert!(err.iter().any(|d| d.code == "E-ALG"));
}

#[test]
fn test_codegen_gcc_compilation_if_available() {
    let mut diagram = BlockDiagram::new("oscillator");
    let mut sine = Block::new("1", "Sin", BlockType::SineWave);
    sine.parameters
        .insert("Amplitude".to_string(), "5.0".to_string());
    sine.parameters
        .insert("Frequency".to_string(), "100.0".to_string());

    let mut gain = Block::new("2", "Gain", BlockType::Gain);
    gain.parameters
        .insert("Gain".to_string(), "0.5".to_string());

    let scope = Block::new("3", "Scope", BlockType::Scope);

    diagram.blocks = vec![sine, gain, scope];
    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);

    let config = CodegenConfig {
        model_name: "sine_gain".to_string(),
        include_main: true,
        step_size: Some(0.001),
        stop_time: Some(0.01),
    };

    let generated = generate_c(&diagram, &config).expect("Codegen succeeds");

    // Test gcc compilation if gcc is callable in the current environment
    let gcc_check = std::process::Command::new("gcc").arg("--version").output();

    if let Ok(output) = gcc_check {
        if output.status.success() {
            let temp_dir = std::env::temp_dir().join("matrust_codegen_test");
            let _ = std::fs::create_dir_all(&temp_dir);

            let header_file = temp_dir.join("sine_gain.h");
            let source_file = temp_dir.join("sine_gain.c");
            let main_file = temp_dir.join("sine_gain_main.c");
            let exe_file = temp_dir.join(if cfg!(windows) {
                "sine_gain.exe"
            } else {
                "sine_gain"
            });

            std::fs::write(&header_file, &generated.header).expect("Write header");
            std::fs::write(&source_file, &generated.source).expect("Write source");
            std::fs::write(&main_file, generated.main_runner.as_ref().unwrap())
                .expect("Write main");

            let compile_res = std::process::Command::new("gcc")
                .current_dir(&temp_dir)
                .args([
                    "-std=c99",
                    "-Wall",
                    "-O2",
                    "sine_gain.c",
                    "sine_gain_main.c",
                    "-lm",
                    "-o",
                ])
                .arg(&exe_file)
                .output()
                .expect("Run gcc compiler");

            assert!(
                compile_res.status.success(),
                "GCC compilation failed: {}",
                String::from_utf8_lossy(&compile_res.stderr)
            );

            // Execute the compiled C binary and verify stdout
            let run_res = std::process::Command::new(&exe_file)
                .output()
                .expect("Execute compiled C binary");

            assert!(run_res.status.success());
            let stdout = String::from_utf8_lossy(&run_res.stdout);
            assert!(stdout.contains("time,Sin,Gain,Scope"));
            assert!(stdout.lines().count() >= 10);
        }
    }
}
