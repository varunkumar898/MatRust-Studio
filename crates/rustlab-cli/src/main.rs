//! RustLab CLI: Interactive REPL, script executor, and benchmark runner.

use clap::{Parser, Subcommand};
use rustlab_cli::bench;
use rustlab_cli::format::format_eval_error;
use rustlab_cli::repl;
use rustlab_stdlib::new_interpreter;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

#[derive(Parser)]
#[command(name = "rustlab")]
#[command(author = "RustLab Systems Architecture")]
#[command(version = "0.1.0")]
#[command(about = "Clean-room MATLAB-compatible numerical computing engine in Rust")]
struct Cli {
    /// MATLAB script file to execute
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Inline code expression to evaluate
    #[arg(short = 'e', long = "eval", value_name = "CODE")]
    eval: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the comprehensive numerical performance benchmark suite
    Bench,
}

fn main() {
    let cli = Cli::parse();

    // Check subcommand
    if let Some(Commands::Bench) = cli.command {
        bench::run_benchmarks();
        return;
    }

    // Shorthand: `rustlab bench`
    if let Some(ref path) = cli.file {
        if path.to_str() == Some("bench") {
            bench::run_benchmarks();
            return;
        }
    }

    // Inline expression evaluation mode: `rustlab -e "..."`
    if let Some(code) = cli.eval {
        let mut interp = new_interpreter();
        match interp.eval_script(&code) {
            Ok(_) => {
                for out in interp.output_log.drain(..) {
                    print!("{out}");
                }
            }
            Err(err) => {
                let formatted = format_eval_error(&err, &code);
                eprintln!("{formatted}");
                exit(1);
            }
        }
        return;
    }

    // Script file execution mode: `rustlab file.m`
    if let Some(path) = cli.file {
        let code = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error: Cannot read file '{}': {err}", path.display());
                exit(1);
            }
        };

        let mut interp = new_interpreter();
        match interp.eval_script(&code) {
            Ok(_) => {
                for out in interp.output_log.drain(..) {
                    print!("{out}");
                }
            }
            Err(err) => {
                let formatted = format_eval_error(&err, &code);
                eprintln!("{formatted}");
                exit(1);
            }
        }
        return;
    }

    // Interactive REPL mode
    if let Err(err) = repl::start_repl() {
        eprintln!("REPL Error: {err}");
        exit(1);
    }
}
