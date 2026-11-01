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
    }

    Ok(())
}
