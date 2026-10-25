//! Interactive Read-Eval-Print Loop (REPL) for RustLab.

use crate::format::format_eval_error;
use rustlab_eval::Interpreter;
use rustlab_parse::ParseError;
use rustlab_stdlib::new_interpreter;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;

/// Starts the interactive RustLab REPL.
pub fn start_repl() -> Result<(), Box<dyn std::error::Error>> {
    print_welcome_banner();

    let mut rl = DefaultEditor::new()?;
    let history_path = get_history_path();

    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    let mut interp = new_interpreter();
    let mut buffer = String::new();

    loop {
        let prompt = if buffer.is_empty() { ">> " } else { ".. " };
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                // Handle REPL-specific meta-commands when buffer is empty
                if buffer.is_empty() {
                    if trimmed == "exit" || trimmed == "quit" {
                        break;
                    }
                    if trimmed == "clc" {
                        print!("\x1B[2J\x1B[1;1H");
                        continue;
                    }
                    if trimmed == "clear" {
                        interp.env.clear_locals();
                        continue;
                    }
                    if trimmed == "who" {
                        print_who(&interp);
                        continue;
                    }
                    if trimmed == "whos" {
                        print_whos(&interp);
                        continue;
                    }
                }

                if !buffer.is_empty() {
                    buffer.push('\n');
                }
                buffer.push_str(&line);

                if is_incomplete_input(&buffer) {
                    continue;
                }

                let _ = rl.add_history_entry(&buffer);

                // Execute complete command
                match interp.eval_script(&buffer) {
                    Ok(_) => {
                        // Print collected outputs from expressions without trailing ';'
                        for out in interp.output_log.drain(..) {
                            print!("{out}");
                        }
                    }
                    Err(err) => {
                        let formatted = format_eval_error(&err, &buffer);
                        eprintln!("{formatted}");
                        interp.output_log.clear();
                    }
                }

                buffer.clear();
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C pressed: cancel current multi-line buffer or clear line
                buffer.clear();
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D pressed: exit REPL
                println!();
                break;
            }
            Err(err) => {
                eprintln!("REPL Error: {err}");
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}

fn print_welcome_banner() {
    println!(r#"  ____            _   _           _     "#);
    println!(r#" |  _ \ _   _ ___| |_| |    __ _ | |__  "#);
    println!(r#" | |_) | | | / __| __| |   / _` || '_ \ "#);
    println!(r#" |  _ <| |_| \__ \ |_| |__| (_| || |_) |"#);
    println!(r#" |_| \_\\__,_|___/\__|_____\__,_||_.__/ "#);
    println!();
    println!(" RustLab Numerical Computing Engine (v0.1.0)");
    println!(" Architecture: Clean-Room Rust | IEEE-754 Double Core");
    println!(" Type 'help' for builtins, 'bench' for benchmarks, 'quit' to exit.");
    println!();
}

pub fn is_incomplete_input(source: &str) -> bool {
    let trimmed = source.trim();
    if trimmed.ends_with("...") {
        return true;
    }

    // Check for open block control structures (if, for, while, switch, try, function)
    if let Ok(tokens) = rustlab_parse::tokenize(source) {
        let mut block_depth: isize = 0;
        for tok in &tokens {
            match tok.kind {
                rustlab_parse::TokenKind::If
                | rustlab_parse::TokenKind::For
                | rustlab_parse::TokenKind::While
                | rustlab_parse::TokenKind::Switch
                | rustlab_parse::TokenKind::Try
                | rustlab_parse::TokenKind::Function => {
                    block_depth += 1;
                }
                rustlab_parse::TokenKind::End => {
                    block_depth = (block_depth - 1).max(0);
                }
                _ => {}
            }
        }
        if block_depth > 0 {
            return true;
        }
    }

    matches!(
        rustlab_parse::parse(source),
        Err(ParseError::UnexpectedEof { .. }) | Err(ParseError::UnclosedDelimiter { .. })
    )
}

fn get_history_path() -> Option<PathBuf> {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .ok()
        .map(|h| PathBuf::from(h).join(".rustlab_history"))
}

fn print_who(interp: &Interpreter) {
    let mut names: Vec<_> = interp.env.locals().keys().cloned().collect();
    names.sort();
    if names.is_empty() {
        return;
    }
    println!();
    println!("Your variables are:");
    println!();
    for name in names {
        print!("  {:<16}", name);
    }
    println!();
    println!();
}

fn print_whos(interp: &Interpreter) {
    let locals = interp.env.locals();
    if locals.is_empty() {
        return;
    }
    println!();
    println!(
        "  {:<16} {:<16} {:<12} {:<16}",
        "Name", "Size", "Bytes", "Class"
    );
    println!("  {:-<60}", "");

    let mut names: Vec<_> = locals.keys().cloned().collect();
    names.sort();

    for name in names {
        if let Some(val) = locals.get(&name) {
            let shape = val.shape();
            let size_str = if shape.is_empty() {
                "0x0".to_string()
            } else {
                shape
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("x")
            };
            let bytes = estimate_bytes(val);
            let class_str = val.type_name();

            println!(
                "  {:<16} {:<16} {:<12} {:<16}",
                name, size_str, bytes, class_str
            );
        }
    }
    println!();
}

fn estimate_bytes(val: &rustlab_math::Value) -> usize {
    let count: usize = val.shape().iter().product();
    match val {
        rustlab_math::Value::Double(_) => count * 8,
        rustlab_math::Value::ComplexDouble(_) => count * 16,
        rustlab_math::Value::Single(_) => count * 4,
        rustlab_math::Value::Logical(_) => count,
        rustlab_math::Value::Int8(_) | rustlab_math::Value::UInt8(_) => count,
        rustlab_math::Value::Int16(_) | rustlab_math::Value::UInt16(_) => count * 2,
        rustlab_math::Value::Int32(_) | rustlab_math::Value::UInt32(_) => count * 4,
        rustlab_math::Value::Int64(_) | rustlab_math::Value::UInt64(_) => count * 8,
        rustlab_math::Value::Sparse(s) => {
            s.col_ptr.len() * 8 + s.row_ind.len() * 8 + s.values.len() * 8
        }
        rustlab_math::Value::Char(_) => count * 4,
        rustlab_math::Value::String(sa) => sa.as_slice().iter().map(|s| s.len() + 24).sum(),
        rustlab_math::Value::Cell(_) => count * 24,
        rustlab_math::Value::Struct(_) => count * 64,
        rustlab_math::Value::FunctionHandle(_) => 32,
    }
}
