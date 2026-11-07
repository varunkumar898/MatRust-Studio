//! Bit-for-bit regression test comparing the new M1 ExecutionEngine against the old Gauss-Seidel solver from main (ef3fbb9).

use mlapp_sim::model::{Block, BlockDiagram, BlockType};
use mlapp_sim::solver::{simulate_diagram, SimulationConfig, SimulationOutput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =========================================================================
// OLD GAUSS-SEIDEL SOLVER IMPLEMENTATION (Verbatim from main commit ef3fbb9)
// =========================================================================

struct OldBlockState {
    continuous_state: f64,
    discrete_state: f64,
    #[allow(dead_code)]
    derivative: f64,
    output: f64,
}

fn old_simulate_diagram(
    diagram: &BlockDiagram,
    config: &SimulationConfig,
) -> Result<SimulationOutput, String> {
    if config.step_size <= 0.0 {
        return Err("Step size must be positive".into());
    }
    if config.stop_time < config.start_time {
        return Err("Stop time must be >= start time".into());
    }

    let num_steps = ((config.stop_time - config.start_time) / config.step_size).round() as usize;

    let mut states: HashMap<String, OldBlockState> = HashMap::new();
    for b in &diagram.blocks {
        let init_cond = b
            .get_param("InitialCondition")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        states.insert(
            b.sid.clone(),
            OldBlockState {
                continuous_state: init_cond,
                discrete_state: init_cond,
                derivative: 0.0,
                output: 0.0,
            },
        );
    }

    let mut tout = Vec::with_capacity(num_steps + 1);
    let mut yout = Vec::with_capacity(num_steps + 1);
    let mut signals: HashMap<String, Vec<f64>> = HashMap::new();

    let log_blocks: Vec<&Block> = diagram
        .blocks
        .iter()
        .filter(|b| {
            matches!(
                b.block_type,
                BlockType::Scope | BlockType::ToWorkspace | BlockType::Outport
            )
        })
        .collect();

    for b in &log_blocks {
        signals.insert(b.name.clone(), Vec::with_capacity(num_steps + 1));
    }

    let dt = config.step_size;

    for step in 0..=num_steps {
        let t = config.start_time + (step as f64) * dt;

        for b in &diagram.blocks {
            let st = states
                .get_mut(&b.sid)
                .ok_or_else(|| format!("Block not found: {}", b.sid))?;
            match b.block_type {
                BlockType::Constant => {
                    let val = b
                        .get_param("Value")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    st.output = val;
                }
                BlockType::SineWave => {
                    let amp = b
                        .get_param("Amplitude")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let freq = b
                        .get_param("Frequency")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let phase = b
                        .get_param("Phase")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let bias = b
                        .get_param("Bias")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    st.output = amp * (freq * t + phase).sin() + bias;
                }
                BlockType::Step => {
                    let step_t = b
                        .get_param("Time")
                        .or_else(|| b.get_param("StepTime"))
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let before = b
                        .get_param("Before")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let after = b
                        .get_param("After")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    st.output = if t >= step_t { after } else { before };
                }
                BlockType::Integrator => {
                    st.output = st.continuous_state;
                }
                BlockType::UnitDelay => {
                    st.output = st.discrete_state;
                }
                _ => {}
            }
        }

        let max_iter = 20;
        let mut converged = false;

        for _iter in 0..max_iter {
            let mut max_change = 0.0f64;

            for b in &diagram.blocks {
                let inputs = old_get_block_inputs(b, diagram, &states);

                let new_output = match b.block_type {
                    BlockType::Gain => {
                        let gain = b
                            .get_param("Gain")
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(1.0);
                        let u = inputs.first().copied().unwrap_or(0.0);
                        gain * u
                    }
                    BlockType::Sum => {
                        let signs = b.get_param("Inputs").unwrap_or("++");
                        let mut sum = 0.0;
                        let mut sign_chars = signs.chars().filter(|&c| c == '+' || c == '-');
                        for &u in &inputs {
                            let s = sign_chars.next().unwrap_or('+');
                            if s == '-' {
                                sum -= u;
                            } else {
                                sum += u;
                            }
                        }
                        sum
                    }
                    BlockType::Product => {
                        if inputs.is_empty() {
                            0.0
                        } else {
                            inputs.iter().product()
                        }
                    }
                    BlockType::Saturation => {
                        let lower = b
                            .get_param("LowerLimit")
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(-1.0);
                        let upper = b
                            .get_param("UpperLimit")
                            .and_then(|s| s.parse::<f64>().ok())
                            .unwrap_or(1.0);
                        let u = inputs.first().copied().unwrap_or(0.0);
                        u.clamp(lower, upper)
                    }
                    BlockType::Scope | BlockType::ToWorkspace | BlockType::Outport => {
                        inputs.first().copied().unwrap_or(0.0)
                    }
                    _ => states.get(&b.sid).map(|s| s.output).unwrap_or(0.0),
                };

                if let Some(st) = states.get_mut(&b.sid) {
                    let diff = (new_output - st.output).abs();
                    if diff > max_change {
                        max_change = diff;
                    }
                    st.output = new_output;
                }
            }

            if max_change < 1e-9 {
                converged = true;
                break;
            }
        }

        if !converged {
            return Err(format!(
                "Algebraic loop failed to converge after {max_iter} iterations"
            ));
        }

        tout.push(t);

        let primary_val = log_blocks
            .first()
            .and_then(|b| states.get(&b.sid).map(|s| s.output))
            .unwrap_or_else(|| {
                diagram
                    .blocks
                    .last()
                    .and_then(|b| states.get(&b.sid).map(|s| s.output))
                    .unwrap_or(0.0)
            });
        yout.push(primary_val);

        for b in &log_blocks {
            if let Some(st) = states.get(&b.sid) {
                if let Some(vec) = signals.get_mut(&b.name) {
                    vec.push(st.output);
                }
            }
        }

        for b in &diagram.blocks {
            let inputs = old_get_block_inputs(b, diagram, &states);
            let u = inputs.first().copied().unwrap_or(0.0);

            if let Some(st) = states.get_mut(&b.sid) {
                match b.block_type {
                    BlockType::Integrator => {
                        st.derivative = u;
                        st.continuous_state += dt * u;
                    }
                    BlockType::UnitDelay => {
                        st.discrete_state = u;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(SimulationOutput {
        tout,
        yout,
        signals,
    })
}

fn old_get_block_inputs(
    block: &Block,
    diagram: &BlockDiagram,
    states: &HashMap<String, OldBlockState>,
) -> Vec<f64> {
    let num_inputs = block.input_ports();
    let mut inputs = vec![0.0; num_inputs];

    let incoming_lines: Vec<_> = diagram
        .lines
        .iter()
        .filter(|l| l.dst.block_sid == block.sid || l.dst.block_sid == block.name)
        .collect();

    for line in incoming_lines {
        let src_val = diagram
            .find_block(&line.src.block_sid)
            .and_then(|src_block| states.get(&src_block.sid).map(|st| st.output))
            .unwrap_or(0.0);
        let port_idx = if line.dst.port_index > 0 {
            line.dst.port_index - 1
        } else {
            0
        };
        if port_idx < num_inputs {
            inputs[port_idx] = src_val;
        } else {
            inputs.push(src_val);
        }
    }

    inputs
}

// =========================================================================
// DIAGRAM FACTORIES FOR ALL PRE-EXISTING MODELS
// =========================================================================

fn make_step_response_feedback() -> BlockDiagram {
    let mut diagram = BlockDiagram::new("StepResponseFeedback");
    let step = Block::new("1", "StepSource", BlockType::Step)
        .with_param("Time", "1.0")
        .with_param("Before", "0.0")
        .with_param("After", "1.0");
    let sum = Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "+-");
    let integ =
        Block::new("3", "Integrator", BlockType::Integrator).with_param("InitialCondition", "0.0");
    let scope = Block::new("4", "Scope", BlockType::Scope);
    let gain = Block::new("5", "FeedbackGain", BlockType::Gain).with_param("Gain", "2.0");

    diagram.add_block(step);
    diagram.add_block(sum);
    diagram.add_block(integ);
    diagram.add_block(scope);
    diagram.add_block(gain);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram.add_line("3", 1, "4", 1);
    diagram.add_line("4", 1, "5", 1);
    diagram.add_line("5", 1, "2", 2);
    diagram
}

fn make_gain_model(gain_val: &str) -> BlockDiagram {
    let mut diagram = BlockDiagram::new(format!("GainModel_{gain_val}"));
    let step = Block::new("1", "Step", BlockType::Step)
        .with_param("Time", "0.5")
        .with_param("Before", "0.0")
        .with_param("After", "1.0");
    let gain = Block::new("2", "Gain", BlockType::Gain).with_param("Gain", gain_val);
    let scope = Block::new("3", "Scope", BlockType::Scope);

    diagram.add_block(step);
    diagram.add_block(gain);
    diagram.add_block(scope);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "3", 1);
    diagram
}

fn make_saturation_and_gain() -> BlockDiagram {
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
    diagram
}

fn make_sine_wave() -> BlockDiagram {
    let mut diagram = BlockDiagram::new("SineGen");
    let sine = Block::new("1", "SineSource", BlockType::SineWave)
        .with_param("Amplitude", "2.5")
        .with_param("Frequency", "3.0")
        .with_param("Phase", "0.5")
        .with_param("Bias", "1.2");
    let scope = Block::new("2", "Scope", BlockType::Scope);

    diagram.add_block(sine);
    diagram.add_block(scope);
    diagram.add_line("1", 1, "2", 1);
    diagram
}

fn make_sample_feedback() -> BlockDiagram {
    let mut diagram = BlockDiagram::new("SampleFeedback");
    let step = Block::new("1", "Step", BlockType::Step)
        .with_param("Time", "1.0")
        .with_param("Before", "0.0")
        .with_param("After", "1.0");
    let sum = Block::new("2", "Sum", BlockType::Sum).with_param("Inputs", "|+-");
    let gain = Block::new("3", "Gain", BlockType::Gain).with_param("Gain", "2.0");
    let integrator =
        Block::new("4", "Integrator", BlockType::Integrator).with_param("InitialCondition", "0.0");
    let scope = Block::new("5", "Scope", BlockType::Scope);

    diagram.add_block(step);
    diagram.add_block(sum);
    diagram.add_block(gain);
    diagram.add_block(integrator);
    diagram.add_block(scope);

    diagram.add_line("1", 1, "2", 1);
    diagram.add_line("2", 1, "4", 1);
    diagram.add_line("4", 1, "5", 1);
    diagram.add_line("4", 1, "3", 1);
    diagram.add_line("3", 1, "2", 2);
    diagram
}

#[derive(Serialize, Deserialize)]
struct ReferenceFixture {
    diagram_name: String,
    num_steps: usize,
    tout: Vec<f64>,
    yout: Vec<f64>,
    yout_bits: Vec<u64>,
    signals: HashMap<String, Vec<f64>>,
    signals_bits: HashMap<String, Vec<u64>>,
}

#[test]
fn test_bit_for_bit_regression_against_old_solver_and_fixtures() {
    let models = vec![
        make_step_response_feedback(),
        make_gain_model("2.0"),
        make_gain_model("5.0"),
        make_saturation_and_gain(),
        make_sine_wave(),
        make_sample_feedback(),
    ];

    let config = SimulationConfig {
        start_time: 0.0,
        stop_time: 1.0,
        step_size: 0.01,
        solver: "FixedStepEuler".to_string(),
    };

    let mut fixture_entries = Vec::new();
    let mut total_signals_checked = 0usize;
    let mut total_points_checked = 0usize;

    for diagram in &models {
        let old_out = old_simulate_diagram(diagram, &config)
            .unwrap_or_else(|e| panic!("old solver failed on {}: {e}", diagram.name));
        let new_out = simulate_diagram(diagram, &config)
            .unwrap_or_else(|e| panic!("new engine failed on {}: {e}", diagram.name));

        // 1. Bit-for-bit comparison of tout
        assert_eq!(
            old_out.tout.len(),
            new_out.tout.len(),
            "{}: tout length mismatch",
            diagram.name
        );
        for i in 0..old_out.tout.len() {
            assert_eq!(
                new_out.tout[i].to_bits(),
                old_out.tout[i].to_bits(),
                "{}: tout[{i}] bit mismatch: new {:?} vs old {:?}",
                diagram.name,
                new_out.tout[i],
                old_out.tout[i]
            );
            total_points_checked += 1;
        }

        // 2. Bit-for-bit comparison of yout
        assert_eq!(
            old_out.yout.len(),
            new_out.yout.len(),
            "{}: yout length mismatch",
            diagram.name
        );
        for i in 0..old_out.yout.len() {
            assert_eq!(
                new_out.yout[i].to_bits(),
                old_out.yout[i].to_bits(),
                "{}: yout[{i}] bit mismatch: new {:?} vs old {:?}",
                diagram.name,
                new_out.yout[i],
                old_out.yout[i]
            );
            total_points_checked += 1;
        }
        total_signals_checked += 1;

        // 3. Bit-for-bit comparison of logged signals
        assert_eq!(
            old_out.signals.len(),
            new_out.signals.len(),
            "{}: signals count mismatch",
            diagram.name
        );
        for (name, old_sig) in &old_out.signals {
            let new_sig = new_out
                .signals
                .get(name)
                .unwrap_or_else(|| panic!("missing signal {name} in new output"));
            assert_eq!(
                old_sig.len(),
                new_sig.len(),
                "{}: signal {} length mismatch",
                diagram.name,
                name
            );
            for i in 0..old_sig.len() {
                assert_eq!(
                    new_sig[i].to_bits(),
                    old_sig[i].to_bits(),
                    "{}: signal {}[{i}] bit mismatch",
                    diagram.name,
                    name
                );
                total_points_checked += 1;
            }
            total_signals_checked += 1;
        }

        let yout_bits = new_out.yout.iter().map(|v| v.to_bits()).collect();
        let signals_bits = new_out
            .signals
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(|val| val.to_bits()).collect()))
            .collect();

        fixture_entries.push(ReferenceFixture {
            diagram_name: diagram.name.clone(),
            num_steps: new_out.tout.len(),
            tout: new_out.tout.clone(),
            yout: new_out.yout.clone(),
            yout_bits,
            signals: new_out.signals.clone(),
            signals_bits,
        });
    }

    // Persist UTF-8 JSON fixture if running or verify against disk fixture if present
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("reference_outputs.json");

    if !fixture_path.exists() {
        let json_data =
            serde_json::to_string_pretty(&fixture_entries).expect("serialize reference fixtures");
        std::fs::write(&fixture_path, json_data).expect("write reference fixture");
    } else {
        let json_data = std::fs::read_to_string(&fixture_path).expect("read reference fixture");
        let loaded: Vec<ReferenceFixture> =
            serde_json::from_str(&json_data).expect("parse reference fixture");
        assert_eq!(loaded.len(), fixture_entries.len());
        for (actual, expected) in fixture_entries.iter().zip(&loaded) {
            assert_eq!(actual.diagram_name, expected.diagram_name);
            for (a, e) in actual.yout_bits.iter().zip(&expected.yout_bits) {
                assert_eq!(a, e);
            }
            for (name, a_sig) in &actual.signals_bits {
                let e_sig = expected
                    .signals_bits
                    .get(name)
                    .expect("expected signal in fixture");
                assert_eq!(a_sig, e_sig);
            }
        }
    }

    println!(
        "Bit-for-bit regression passed: verified {} diagrams, {} signals, {} total points with 0 bit differences.",
        models.len(),
        total_signals_checked,
        total_points_checked
    );
}
