use mlapp_container::ContainerLimits;
use mlapp_sim::{
    export_slx, inspect_slx, simulate_diagram, Block, BlockDiagram, BlockType, SimulationConfig,
};
use std::io::Cursor;

#[test]
fn test_slx_inspect_and_parser() {
    let mut diagram = BlockDiagram::new("TestModel");
    diagram.release = Some("R2024b".to_string());
    diagram.version = Some("1.2".to_string());
    diagram.toolboxes.push("Simulink".to_string());

    diagram.add_block(
        Block::new("1", "Sine Wave", BlockType::SineWave)
            .with_param("Amplitude", "2.5")
            .with_param("Frequency", "10.0"),
    );
    diagram.add_block(Block::new("2", "Amplifier", BlockType::Gain).with_param("Gain", "4.0"));
    diagram.add_block(Block::new("3", "Out1", BlockType::Outport));

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);

    let mut buf = Vec::new();
    export_slx(Cursor::new(&mut buf), &diagram).expect("Failed to export slx");

    let limits = ContainerLimits::default();
    let (parsed_diagram, report) =
        inspect_slx(Cursor::new(buf), limits).expect("Failed to inspect slx");

    assert_eq!(report.model_name, "TestModel");
    assert_eq!(report.release, "R2024b");
    assert_eq!(report.version, "1.2");
    assert_eq!(report.total_blocks, 3);
    assert_eq!(report.total_lines, 2);
    assert!(report.is_solver_supported);
    assert_eq!(report.required_toolboxes, vec!["Simulink"]);

    assert_eq!(parsed_diagram.blocks.len(), 3);
    let sine = parsed_diagram.find_block("Sine Wave").expect("found block");
    assert_eq!(sine.get_param("Amplitude"), Some("2.5"));
    assert_eq!(sine.get_param("Frequency"), Some("10.0"));
}

#[test]
fn test_simulation_first_order_step_response() {
    // System: dy/dt = -y + u, y(0) = 0, u(t) = 1 (step at t=0)
    // Analytical solution: y(t) = 1 - exp(-t)
    let mut diagram = BlockDiagram::new("FirstOrderSystem");

    let step = Block::new("1", "StepInput", BlockType::Step)
        .with_param("Time", "0.0")
        .with_param("Before", "0.0")
        .with_param("After", "1.0");

    let sum = Block::new("2", "SumError", BlockType::Sum).with_param("Inputs", "+-");

    let integrator = Block::new("3", "IntegratorState", BlockType::Integrator)
        .with_param("InitialCondition", "0.0");

    let out = Block::new("4", "OutputSignal", BlockType::Outport);

    diagram.add_block(step);
    diagram.add_block(sum);
    diagram.add_block(integrator);
    diagram.add_block(out);

    // Line 1: StepInput -> Sum (port 1: +)
    diagram.add_line("1", 1, "2", 1);
    // Line 2: Sum -> Integrator (port 1)
    diagram.add_line("2", 1, "3", 1);
    // Line 3: Integrator -> OutputSignal
    diagram.add_line("3", 1, "4", 1);
    // Line 4 (Feedback): Integrator -> Sum (port 2: -)
    diagram.add_line("3", 1, "2", 2);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 5.0,
        step_size: 0.001,
        solver: "FixedStepEuler".to_string(),
    };

    let sim_out = simulate_diagram(&diagram, &config).expect("Simulation failed");

    assert_eq!(sim_out.tout.len(), 5001);
    assert_eq!(sim_out.yout.len(), 5001);

    // Initial value y(0) = 0
    assert!((sim_out.yout[0] - 0.0).abs() < 1e-6);

    // At t = 1.0 (step 1000): y(1) = 1 - e^(-1) ~ 0.6321205588
    let expected_t1 = 1.0 - (-1.0f64).exp();
    let actual_t1 = sim_out.yout[1000];
    assert!(
        (actual_t1 - expected_t1).abs() < 2e-3,
        "Actual {actual_t1} vs Expected {expected_t1}"
    );

    // At t = 5.0 (step 5000): y(5) = 1 - e^(-5) ~ 0.993262053
    let expected_t5 = 1.0 - (-5.0f64).exp();
    let actual_t5 = sim_out.yout[5000];
    assert!(
        (actual_t5 - expected_t5).abs() < 2e-3,
        "Actual {actual_t5} vs Expected {expected_t5}"
    );

    // MATLAB Value conversion roundtrip
    let mat_val = sim_out.to_matlab_value();
    if let rustlab_math::Value::Struct(st) = mat_val {
        assert!(st.fields.contains(&"tout".to_string()));
        assert!(st.fields.contains(&"yout".to_string()));
    } else {
        panic!("Expected Struct Value");
    }
}

#[test]
fn test_simulation_saturation_and_gain() {
    let mut diagram = BlockDiagram::new("SaturatingGain");

    let sine = Block::new("1", "SineSource", BlockType::SineWave)
        .with_param("Amplitude", "5.0")
        .with_param("Frequency", "1.0");

    let sat = Block::new("2", "Limiter", BlockType::Saturation)
        .with_param("LowerLimit", "-2.0")
        .with_param("UpperLimit", "2.0");

    let gain = Block::new("3", "Amp", BlockType::Gain).with_param("Gain", "3.0");

    let out = Block::new("4", "Out", BlockType::Outport);

    diagram.add_block(sine);
    diagram.add_block(sat);
    diagram.add_block(gain);
    diagram.add_block(out);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: std::f64::consts::TAU,
        step_size: 0.01,
        solver: "FixedStepEuler".to_string(),
    };

    let sim_out = simulate_diagram(&diagram, &config).expect("Simulation failed");

    // The maximum saturated output should be 2.0 * 3.0 = 6.0
    let max_val = sim_out
        .yout
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    assert!((max_val - 6.0).abs() < 1e-4);

    let min_val = sim_out.yout.iter().copied().fold(f64::INFINITY, f64::min);
    assert!((min_val - -6.0).abs() < 1e-4);
}

#[test]
fn test_security_dos_file_limits_on_slx() {
    let mut diagram = BlockDiagram::new("TinyLimits");
    diagram.add_block(Block::new("1", "Const", BlockType::Constant));

    let mut buf = Vec::new();
    export_slx(Cursor::new(&mut buf), &diagram).expect("Export failed");

    // Strictly limit max files to 1 (archive has 4 files)
    let limits = ContainerLimits {
        max_file_count: 1,
        max_uncompressed_bytes: 1_000_000,
        max_compression_ratio: 100.0,
    };

    let result = inspect_slx(Cursor::new(buf), limits);
    assert!(result.is_err());
    let err_str = result.err().unwrap().to_string();
    assert!(err_str.contains("maximum file count exceeded"));
}

#[test]
fn test_generate_and_verify_sample_model_fixture() {
    let mut diagram = BlockDiagram::new("simple_feedback");
    diagram.release = Some("R2024b".to_string());
    diagram.version = Some("1.0".to_string());
    diagram.toolboxes.push("Simulink".to_string());

    diagram.add_block(
        Block::new("1", "SineWave", BlockType::SineWave)
            .with_param("Amplitude", "1.0")
            .with_param("Frequency", "1.0")
            .with_param("Phase", "0.0")
            .with_param("Bias", "0.0"),
    );
    diagram.add_block(Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "+-"));
    diagram.add_block(Block::new("3", "Gain", BlockType::Gain).with_param("Gain", "2.0"));
    diagram.add_block(
        Block::new("4", "Integrator", BlockType::Integrator).with_param("InitialCondition", "0.0"),
    );
    diagram.add_block(Block::new("5", "Out1", BlockType::Outport));

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);
    diagram.add_line("4", 1, "5", 1);
    diagram.add_line("4", 1, "2", 2);

    let fixture_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures")
        .join("sample_models");
    std::fs::create_dir_all(&fixture_dir).expect("Create fixture dir");
    let fixture_path = fixture_dir.join("simple_feedback.slx");

    let f = std::fs::File::create(&fixture_path).expect("Create fixture file");
    export_slx(f, &diagram).expect("Export slx fixture");

    // Inspect the generated fixture file
    let f_in = std::fs::File::open(&fixture_path).expect("Open fixture file");
    let (read_diag, report) =
        inspect_slx(f_in, ContainerLimits::default()).expect("Inspect fixture");

    assert_eq!(report.model_name, "simple_feedback");
    assert_eq!(report.total_blocks, 5);
    assert_eq!(report.total_lines, 5);
    assert!(report.is_solver_supported);
    assert_eq!(read_diag.blocks.len(), 5);
}
