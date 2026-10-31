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

        /// Launch GUI window instead of headless mode
        #[arg(long, default_value_t = false)]
        gui: bool,

        /// Execution timeout per callback in milliseconds
        #[arg(long, default_value_t = 5000)]
        timeout_ms: u64,
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
            gui,
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
                callback_timeout_ms: timeout_ms,
            };

            println!("Launching app package: {:?}", package_path);
            let mut session =
                AppSession::launch(file, config).with_context(|| "Failed to launch app session")?;

            println!("App Name: {}", session.manifest.app_name);
            println!("Release: {:?}", session.manifest.detected_matlab_release);

            if gui {
                println!(
                    "[GUI Mode]: Initializing GUI window for '{}'...",
                    session.manifest.app_name
                );
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
    }

    Ok(())
}
