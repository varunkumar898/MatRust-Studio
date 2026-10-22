use std::io::{self, BufRead, Write};

use matlab_runtime as rt;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        repl();
    } else if args[0] == "-e" {
        if args.len() < 2 {
            eprintln!("usage: matlabr -e '<source>' | matlabr <script.m> | matlabr (REPL)");
            std::process::exit(2);
        }
        exec_and_print(&args[1]);
    } else {
        match std::fs::read_to_string(&args[0]) {
            Ok(source) => exec_and_print(&source),
            Err(err) => {
                eprintln!("matlabr: cannot read '{}': {}", args[0], err);
                std::process::exit(2);
            }
        }
    }
}

fn exec_and_print(source: &str) {
    match rt::run_source(source) {
        Ok(result) => print_run(&result),
        Err(d) => {
            eprint!("{}", d.render(source));
            std::process::exit(1);
        }
    }
}

fn print_run(result: &rt::RunResult) {
    for line in &result.stdout {
        println!("{}", line);
    }
    for out in &result.outputs {
        let label = out.label.as_deref().unwrap_or("ans");
        println!("{} =\n{}", label, rt::format_value(&out.value));
    }
}

fn repl() {
    let mut env = rt::Env::new();
    let stdin = io::stdin();
    println!("matlabr 0.1.0 (interpreter milestone 1) -- type 'exit' to quit");
    loop {
        print!("matlabr> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match trimmed {
            "exit" | "quit" | "exit()" | "quit()" => break,
            "clear" => {
                env.clear();
                continue;
            }
            "who" => {
                for name in env.variable_names() {
                    println!("{}", name);
                }
                continue;
            }
            _ => {}
        }
        match rt::exec_source(&mut env, trimmed) {
            Ok(result) => print_run(&result),
            Err(d) => eprint!("{}", d.render(trimmed)),
        }
    }
}
