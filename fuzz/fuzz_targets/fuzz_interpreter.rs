#![cfg_attr(not(target_os = "windows"), no_main)]

use rustlab_eval::{EvalError, Interpreter};
use rustlab_math::Value;
use rustlab_parse::Span;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub fn fuzz_one(data: &[u8]) {
    // Only process valid UTF-8 strings under 64KB
    if data.len() > 65536 {
        return;
    }
    let s = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // First ensure the parser handles it without panicking
    let _ = rustlab_parse::parse(s);

    // Initialize an isolated interpreter with standard library builtins
    let mut interp = Interpreter::new();
    rustlab_stdlib::register_all(&mut interp);

    // Block disk I/O builtins to keep fuzzing purely in-memory
    let sandbox_stub = |_args: &[Value], _nargout: usize| -> Result<Vec<Value>, EvalError> {
        Err(EvalError::UserError {
            identifier: "RUSTLAB:SandboxViolation".to_string(),
            message: "File I/O disabled in fuzzing sandbox".to_string(),
            span: Span::default(),
        })
    };
    interp.register_builtin("readmatrix", sandbox_stub);
    interp.register_builtin("writematrix", sandbox_stub);
    interp.register_builtin("readtable", sandbox_stub);
    interp.register_builtin("writetable", sandbox_stub);

    // Tight recursion bound
    interp.max_recursion_depth = 16;

    // Enforce execution timeout via cancellation token
    let cancel_token = Arc::new(AtomicBool::new(false));
    interp.cancel_token = Some(cancel_token.clone());

    let token_clone = cancel_token.clone();
    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        token_clone.store(true, Ordering::Relaxed);
    });

    // Execute script; must not panic regardless of input
    let _ = interp.eval_script(s);

    cancel_token.store(true, Ordering::Relaxed);
    let _ = watchdog.join();
}

#[cfg(not(target_os = "windows"))]
libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    fuzz_one(data);
});

#[cfg(target_os = "windows")]
fn main() {
    println!("Fuzzing on Windows runs via smoke test runner.");
}
