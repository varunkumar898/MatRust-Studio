#![deny(clippy::unwrap_used)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mlapp_container::AppContainer;
use mlapp_host::{AppSession, HostConfig, HostNotification};
use mlapp_lang::{analyze_compatibility, extract_code_from_document_xml, parse_app_classdef};
use mlapp_render::{HeadlessRenderer, RenderBackend};
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mlapp")]
#[command(
    about = "MATLAB App Designer (.mlapp) and Installer (.mlappinstall) runtime & inspection CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect an .mlapp or .mlappinstall package and print a detailed JSON report
    Inspect {
        /// Path to .mlapp or .mlappinstall file
        package_path: PathBuf,

        /// Pretty-print the JSON output
        #[arg(long, default_value_t = true)]
        pretty: bool,
    },
    /// Statically analyze an .mlapp package and output a 0-100 compatibility score
    CheckCompat {
        /// Path to .mlapp or .mlappinstall file
        package_path: PathBuf,

        /// Pretty-print the JSON output
        #[arg(long, default_value_t = true)]
        pretty: bool,
    },
    /// Extract entries from an .mlapp or .mlappinstall container safely
    Extract {
        /// Path to .mlapp or .mlappinstall file
        package_path: PathBuf,

        /// Output directory destination
        #[arg(short, long)]
        out_dir: PathBuf,
    },
    /// Run an app package lifecycle, execute startupFcn and render layout
    Run {
        /// Path to .mlapp or .mlappinstall file
        package_path: PathBuf,

        /// Allow file I/O permissions
        #[arg(long, default_value_t = false)]
        allow_file_io: bool,

        /// Allow network permissions
        #[arg(long, default_value_t = false)]
        allow_network: bool,

        /// Allow system/shell command execution
        #[arg(long, default_value_t = false)]
        allow_system: bool,

        /// Allow dynamic evaluation (eval, evalc, evalin, assignin)
        #[arg(long, default_value_t = false)]
        allow_eval: bool,

        /// Allow external language interop (Java, Python, C libraries)
        #[arg(long, default_value_t = false)]
        allow_interop: bool,

        /// Allow delegation of unsupported functions to a real MATLAB installation
        #[arg(long, default_value_t = false)]
        allow_matlab: bool,

        /// Allow delegation of unsupported functions to a local GNU Octave installation
        #[arg(long, default_value_t = false)]
        allow_octave: bool,

        /// Launch GUI window instead of headless mode
        #[arg(long, default_value_t = false)]
        gui: bool,

        /// Path to JSON autotest script for automated UI execution
        #[arg(long)]
        autotest: Option<std::path::PathBuf>,

        /// Execution timeout per callback in milliseconds
        #[arg(long, default_value_t = 5000)]
        timeout_ms: u64,
    },
    /// Inspect a Simulink model (.slx) and print a structural and compatibility report
    SlxInspect {
        /// Path to .slx model file
        model_path: PathBuf,

        /// Output report as JSON
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Statically analyze a block diagram JSON model and print diagnostic errors/warnings
    Check {
        /// Path to block diagram .json file
        file_path: PathBuf,

        /// Output diagnostics as JSON array
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Generate standalone, allocation-free ISO C99 code for a block diagram model
    Codegen {
        /// Path to block diagram .json file
        file_path: PathBuf,

        /// Output directory destination for generated C source and header files
        #[arg(short, long, default_value = "codegen_out")]
        out_dir: PathBuf,

        /// Optional model name identifier for C structs and functions
        #[arg(long)]
        name: Option<String>,

        /// Step size in seconds (default: 0.001)
        #[arg(long)]
        dt: Option<f64>,

        /// Stop time in seconds (default: 1.0)
        #[arg(long)]
        stop_time: Option<f64>,
    },
    /// Run a parameter sweep across a range of values for a block parameter
    Sweep {
        /// Path to block diagram .json file
        file_path: PathBuf,

        /// Target block SID or Name
        #[arg(short, long)]
        block: String,

        /// Parameter name (e.g. "R", "Gain", "Value", "C", "L")
        #[arg(short, long)]
        param: String,

        /// Comma-separated list of float values (e.g. "1.0,2.0,5.0,10.0")
        #[arg(short, long)]
        values: String,

        /// Simulation stop time in seconds
        #[arg(long, default_value_t = 1.0)]
        stop_time: f64,

        /// Optional output directory to export CSV files
        #[arg(short, long)]
        out_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect {
            package_path,
            pretty,
        } => {
            let file = File::open(&package_path)
                .with_context(|| format!("Failed to open package at {:?}", package_path))?;
            let mut container = AppContainer::open(file).with_context(|| {
                "Failed to parse container (format or security limit violation)"
            })?;
            let report = container
                .inspect()
                .with_context(|| "Failed to inspect container entries")?;

            let json_str = if pretty {
                serde_json::to_string_pretty(&report)?
            } else {
                serde_json::to_string(&report)?
            };
            println!("{}", json_str);
        }
        Commands::CheckCompat {
            package_path,
            pretty,
        } => {
            let file = File::open(&package_path)
                .with_context(|| format!("Failed to open package at {:?}", package_path))?;
            let mut container =
                AppContainer::open(file).with_context(|| "Failed to parse container")?;

            let doc_xml = container
                .read_entry_to_string("matlab/document.xml")
                .with_context(|| "matlab/document.xml not found in package")?;
            let code = extract_code_from_document_xml(&doc_xml)
                .with_context(|| "Failed to extract code from document.xml")?;
            let class_def = parse_app_classdef(&code)
                .with_context(|| "Failed to parse App Designer classdef")?;

            let compat = analyze_compatibility(&class_def);
            let json_str = if pretty {
                serde_json::to_string_pretty(&compat)?
            } else {
                serde_json::to_string(&compat)?
            };
            println!("{}", json_str);
        }
        Commands::Extract {
            package_path,
            out_dir,
        } => {
            let file = File::open(&package_path)
                .with_context(|| format!("Failed to open package at {:?}", package_path))?;
            let mut container =
                AppContainer::open(file).with_context(|| "Failed to parse container")?;

            container
                .extract_all(&out_dir)
                .with_context(|| format!("Failed to extract package to {:?}", out_dir))?;

            for entry in container.entries() {
                println!(
                    "Extracted: {} ({} bytes, role: {:?})",
                    entry.name, entry.uncompressed_size, entry.role
                );
            }
            println!("Extraction complete to {:?}", out_dir);
        }
        Commands::Run {
            package_path,
            allow_file_io,
            allow_network,
            allow_system,
            allow_eval,
            allow_interop,
            allow_matlab,
            allow_octave,
            gui,
            autotest,
            timeout_ms,
        } => {
            let file = File::open(&package_path)
                .with_context(|| format!("Failed to open package at {:?}", package_path))?;

            let config = HostConfig {
                allow_file_io,
                allow_network,
                allow_system,
                allow_eval,
                allow_interop,
                allow_matlab,
                allow_octave,
                callback_timeout_ms: timeout_ms,
            };

            println!("Launching app package: {:?}", package_path);
            let mut session =
                AppSession::launch(file, config).with_context(|| "Failed to launch app session")?;

            println!("App Name: {}", session.manifest.app_name);
            println!("Release: {:?}", session.manifest.detected_matlab_release);

            if let Some(test_path) = autotest {
                let script = mlapp_render::AutotestScript::from_file(&test_path).map_err(|e| {
                    anyhow::anyhow!("Failed to load autotest script from {:?}: {e}", test_path)
                })?;

                if gui {
                    println!(
                        "[GUI Autotest Mode]: Launching GUI window for '{}' with autotest script...",
                        session.manifest.app_name
                    );
                    mlapp_render::egui_backend::run_gui_window_with_autotest(session, Some(script))
                        .map_err(|e| anyhow::anyhow!("GUI autotest execution failed: {e}"))?;
                    println!("\nGUI autotest script passed successfully.");
                    return Ok(());
                } else {
                    println!(
                        "[Headless Autotest Mode]: Running autotest script on '{}'...",
                        session.manifest.app_name
                    );
                    mlapp_render::run_autotest_headless(&mut session, &script)
                        .map_err(|e| anyhow::anyhow!("Headless autotest execution failed: {e}"))?;
                    println!("\nHeadless autotest script passed successfully.");
                    return Ok(());
                }
            }

            if gui {
                println!(
                    "[GUI Mode]: Opening native window for '{}'...",
                    session.manifest.app_name
                );
                mlapp_render::egui_backend::run_gui_window(session)
                    .map_err(|e| anyhow::anyhow!("Failed to open the GUI window: {e}"))?;
                println!("\nApp session closed successfully.");
                return Ok(());
            }

            // Render layout statically with HeadlessRenderer
            let mut renderer = HeadlessRenderer::new();
            let _ = renderer.render_tree(&session.ui_tree);
            println!(
                "\nRendered UI Layout ({} components):",
                renderer.rendered_log.len()
            );
            for line in &renderer.rendered_log {
                println!("  {line}");
            }

            // Check any notifications from startupFcn
            std::thread::sleep(std::time::Duration::from_millis(50));
            while let Some(notif) = session.poll_notification() {
                match notif {
                    HostNotification::Mutations(m) => println!("\n[UI Mutation]: {:?}", m),
                    HostNotification::Alert { title, message } => {
                        println!("\n[Alert Dialog] {title}: {message}")
                    }
                    HostNotification::Error(e) => eprintln!("\n[Worker Error]: {e}"),
                    HostNotification::Terminated => println!("\nSession terminated."),
                }
            }

            session.shutdown();
            println!("\nApp session closed successfully.");
        }
        Commands::SlxInspect { model_path, json } => {
            let file = File::open(&model_path)
                .with_context(|| format!("Failed to open .slx model file at {:?}", model_path))?;
            let limits = mlapp_container::ContainerLimits::default();
            let (_diagram, report) = mlapp_sim::inspect_slx(file, limits)
                .with_context(|| format!("Failed to inspect .slx model at {:?}", model_path))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("=== Simulink Model Inspection Report ===");
                println!("Model Name: {}", report.model_name);
                println!("Release:    {}", report.release);
                println!("Version:    {}", report.version);
                println!("Total Blocks: {}", report.total_blocks);
                println!("Total Lines:  {}", report.total_lines);
                println!("Fully Supported by Solver: {}", report.is_solver_supported);
                println!("\nBlock Type Breakdown:");
                for (b_type, count) in &report.block_type_counts {
                    println!("  - {b_type}: {count}");
                }
                if !report.required_toolboxes.is_empty() {
                    println!("\nRequired Toolboxes:");
                    for tb in &report.required_toolboxes {
                        println!("  - {tb}");
                    }
                }
                if !report.unsupported_blocks.is_empty() {
                    println!("\nUnsupported Blocks for Native Simulation:");
                    for blk in &report.unsupported_blocks {
                        println!("  - {blk}");
                    }
                }
            }
        }
        Commands::Check { file_path, json } => {
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read diagram file at {:?}", file_path))?;
            let diagram: mlapp_sim::BlockDiagram =
                serde_json::from_str(&content).with_context(|| {
                    format!("Failed to parse JSON BlockDiagram from {:?}", file_path)
                })?;
            let diags = mlapp_sim::check_diagram(&diagram, diagram.simulation_config.as_ref());

            if json {
                println!("{}", serde_json::to_string_pretty(&diags)?);
            } else if diags.is_empty() {
                println!("No diagnostics found. Diagram is clean.");
            } else {
                println!(
                    "{:<12} | {:<8} | {:<16} | {:<40} | Suggested Fix",
                    "Code", "Severity", "Location", "Message"
                );
                println!(
                    "{:-<12}-+-{:-<8}-+-{:-<16}-+-{:-<40}-+---------------------",
                    "", "", "", ""
                );
                for d in &diags {
                    let loc = if d.block_sids.is_empty() {
                        "-".to_string()
                    } else {
                        d.block_sids.join(", ")
                    };
                    let fix = d.typical_fix().unwrap_or("-");
                    println!(
                        "{:<12} | {:<8} | {:<16} | {:<40} | {}",
                        d.code,
                        d.severity.as_str(),
                        loc,
                        d.message,
                        fix
                    );
                }
            }

            let has_error = diags
                .iter()
                .any(|d| d.severity == mlapp_sim::DiagnosticSeverity::Error);
            let has_warning = diags
                .iter()
                .any(|d| d.severity == mlapp_sim::DiagnosticSeverity::Warning);

            if has_error {
                std::process::exit(2);
            } else if has_warning {
                std::process::exit(1);
            } else {
                std::process::exit(0);
            }
        }
        Commands::Codegen {
            file_path,
            out_dir,
            name,
            dt,
            stop_time,
        } => {
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read diagram file at {:?}", file_path))?;
            let diagram: mlapp_sim::BlockDiagram = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse diagram JSON at {:?}", file_path))?;

            let model_name = name.unwrap_or_else(|| {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("model")
                    .to_string()
            });

            let config = mlapp_sim::CodegenConfig {
                model_name: model_name.clone(),
                include_main: true,
                step_size: dt,
                stop_time,
            };

            let generated = mlapp_sim::generate_c(&diagram, &config).map_err(|diags| {
                for d in &diags {
                    eprintln!("[{}] {}: {}", d.severity.as_str(), d.code, d.message);
                }
                anyhow::anyhow!("Code generation aborted due to model validation errors.")
            })?;

            std::fs::create_dir_all(&out_dir)
                .with_context(|| format!("Failed to create output directory at {:?}", out_dir))?;

            let header_path = out_dir.join(format!("{}.h", generated.model_name));
            let source_path = out_dir.join(format!("{}.c", generated.model_name));

            std::fs::write(&header_path, &generated.header)
                .with_context(|| format!("Failed to write header at {:?}", header_path))?;
            std::fs::write(&source_path, &generated.source)
                .with_context(|| format!("Failed to write source at {:?}", source_path))?;

            println!(
                "Successfully generated C99 code for model '{}':",
                generated.model_name
            );
            println!("  - Header: {:?}", header_path);
            println!("  - Source: {:?}", source_path);

            if let Some(main_src) = &generated.main_runner {
                let main_path = out_dir.join(format!("{}_main.c", generated.model_name));
                std::fs::write(&main_path, main_src)
                    .with_context(|| format!("Failed to write main runner at {:?}", main_path))?;
                println!("  - Runner: {:?}", main_path);
            }
        }
        Commands::Sweep {
            file_path,
            block,
            param,
            values,
            stop_time,
            out_dir,
        } => {
            let content = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read diagram file at {:?}", file_path))?;
            let diagram: mlapp_sim::BlockDiagram = serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse diagram JSON at {:?}", file_path))?;

            let parsed_values: Result<Vec<f64>, _> =
                values.split(',').map(|s| s.trim().parse::<f64>()).collect();
            let parsed_values = parsed_values
                .map_err(|e| anyhow::anyhow!("Invalid float value in --values: {}", e))?;

            if parsed_values.is_empty() {
                anyhow::bail!("--values must contain at least one float value");
            }

            let sweep_cfg = mlapp_sim::ParameterSweepConfig {
                block_id: block,
                param_name: param,
                values: parsed_values,
                sim_config: mlapp_sim::SimulationConfig {
                    stop_time,
                    ..Default::default()
                },
            };

            let results =
                mlapp_sim::run_parameter_sweep(&diagram, &sweep_cfg).map_err(|diags| {
                    for d in &diags {
                        eprintln!("[{}] {}: {}", d.severity.as_str(), d.code, d.message);
                    }
                    anyhow::anyhow!("Parameter sweep aborted due to model validation errors.")
                })?;

            println!("Parameter Sweep Results ({} runs):", results.len());
            println!("{:<14} | Final Signal Values", "Param Value");
            println!("---------------+----------------------------------------");
            for r in &results {
                let mut vals: Vec<_> = r.final_values.iter().collect();
                vals.sort_by_key(|(k, _)| (*k).clone());
                let summary: Vec<String> = vals
                    .iter()
                    .map(|(k, v)| format!("{}={:.4e}", k, v))
                    .collect();
                println!("{:<14.6e} | {}", r.param_value, summary.join(", "));
            }

            if let Some(dir) = out_dir {
                std::fs::create_dir_all(&dir).with_context(|| {
                    format!("Failed to create sweep output directory {:?}", dir)
                })?;
                for (i, r) in results.iter().enumerate() {
                    let csv_path = dir.join(format!("run_{:03}_val_{:.4e}.csv", i, r.param_value));
                    let mut csv = String::new();
                    let sig_names: Vec<String> = r.output.signals.keys().cloned().collect();
                    csv.push_str("time");
                    for name in &sig_names {
                        csv.push_str(&format!(",{}", name));
                    }
                    csv.push('\n');
                    for (step_idx, &t) in r.output.tout.iter().enumerate() {
                        csv.push_str(&format!("{:.6}", t));
                        for name in &sig_names {
                            let val = r
                                .output
                                .signals
                                .get(name)
                                .and_then(|v| v.get(step_idx))
                                .unwrap_or(&0.0);
                            csv.push_str(&format!(",{:e}", val));
                        }
                        csv.push('\n');
                    }
                    std::fs::write(&csv_path, csv)?;
                }
                println!("Exported trajectory CSV files to {:?}", dir);
            }
        }
    }

    Ok(())
}
