//! Parameter Sweep and Batch Simulation Engine for Block Diagrams.
//!
//! Enables programmatic parameter sweeping across model blocks with
//! trajectory capture and convergence analysis.

use crate::diagnostics::{check_diagram, Diagnostic, DiagnosticSeverity};
use crate::model::BlockDiagram;
use crate::solver::{simulate_diagram, SimulationConfig, SimulationOutput};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a parameter sweep run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSweepConfig {
    /// Target block identifier (SID or exact Name).
    pub block_id: String,
    /// Parameter key name to vary (e.g. "R", "Gain", "Value", "C", "L").
    pub param_name: String,
    /// List of parameter values to evaluate.
    pub values: Vec<f64>,
    /// Simulation configuration applied to each run.
    pub sim_config: SimulationConfig,
}

/// Output data and metrics for an individual sweep run.
#[derive(Debug, Clone)]
pub struct SweepRunResult {
    /// Parameter value evaluated in this run.
    pub param_value: f64,
    /// Simulation trajectory output.
    pub output: SimulationOutput,
    /// Final values of all recorded block outputs at t = stop_time.
    pub final_values: HashMap<String, f64>,
}

/// Executes a parameter sweep across the specified parameter values.
///
/// Returns `Err(Vec<Diagnostic>)` if the initial model fails validation.
/// If any individual simulation step encounters an execution error, returns `Err`.
pub fn run_parameter_sweep(
    diagram: &BlockDiagram,
    sweep_config: &ParameterSweepConfig,
) -> Result<Vec<SweepRunResult>, Vec<Diagnostic>> {
    // 1. Initial validation
    let diags = check_diagram(diagram, Some(&sweep_config.sim_config));
    let errors: Vec<Diagnostic> = diags
        .into_iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut results = Vec::with_capacity(sweep_config.values.len());

    for &val in &sweep_config.values {
        let mut model_copy = diagram.clone();
        let mut found = false;

        for blk in &mut model_copy.blocks {
            if blk.sid == sweep_config.block_id || blk.name == sweep_config.block_id {
                blk.parameters
                    .insert(sweep_config.param_name.clone(), format!("{val:e}"));
                found = true;
                break;
            }
        }

        if !found {
            return Err(vec![Diagnostic {
                code: "E-PARAM".to_string(),
                severity: DiagnosticSeverity::Error,
                message: format!(
                    "Parameter sweep target block '{}' not found in model.",
                    sweep_config.block_id
                ),
                block_sids: vec![],
                fix: None,
            }]);
        }

        let sim_out = simulate_diagram(&model_copy, &sweep_config.sim_config)
            .map_err(|e| vec![Diagnostic::error("E-SIM", e.to_string(), vec![])])?;

        let mut final_values = HashMap::new();
        for (sig_name, series) in &sim_out.signals {
            if let Some(&last_val) = series.last() {
                final_values.insert(sig_name.clone(), last_val);
            }
        }

        results.push(SweepRunResult {
            param_value: val,
            output: sim_out,
            final_values,
        });
    }

    Ok(results)
}
