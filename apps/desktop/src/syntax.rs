//! Background syntax checking for the editor.
//!
//! Parsing runs on a worker thread after a 300 ms quiet period. The UI only
//! receives plain diagnostic records; it never blocks on the parser.

use rustlab_cli::format::{format_eval_error, span_to_location};
use rustlab_eval::EvalError;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::time::{Duration, Instant};

/// Quiet time before a changed buffer is parsed again.
pub const SYNTAX_DEBOUNCE: Duration = Duration::from_millis(300);

/// One parser diagnostic, positioned for painting and for the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxDiagnostic {
    /// One-based line.
    pub line: usize,
    /// One-based character column.
    pub column: usize,
    /// Highlight length in characters.
    pub length: usize,
    /// Same text the Run path prints for this parse error.
    pub message: String,
}

/// Parse one source buffer and return at most one parser diagnostic.
///
/// The parser reports the first error and stops, so the list is either empty or
/// contains one entry. No warning is invented when the parser is happy.
pub fn analyze_syntax(source: &str) -> Vec<SyntaxDiagnostic> {
    match rustlab_parse::parse(source) {
        Ok(_) => Vec::new(),
        Err(error) => {
            let span = error.span();
            let (line, column, length, _) = span_to_location(source, span);
            let message = format_eval_error(&EvalError::Parse(error), source);
            vec![SyntaxDiagnostic {
                line,
                column,
                length: length.max(1),
                message,
            }]
        }
    }
}

struct Job {
    id: u64,
    source: String,
}

struct Report {
    id: u64,
    source: String,
    diagnostics: Vec<SyntaxDiagnostic>,
}

/// Non-blocking syntax checker with one parser worker.
pub struct SyntaxService {
    job_tx: Option<Sender<Job>>,
    result_tx: Sender<Report>,
    result_rx: Receiver<Report>,
    pending_source: Option<String>,
    last_edit_at: Option<Instant>,
    last_sent_source: Option<String>,
    latest_sent_id: u64,
}

impl Default for SyntaxService {
    fn default() -> Self {
        let (result_tx, result_rx) = channel();
        Self {
            job_tx: None,
            result_tx,
            result_rx,
            pending_source: None,
            last_edit_at: None,
            last_sent_source: None,
            latest_sent_id: 0,
        }
    }
}

impl SyntaxService {
    /// Remember an edited buffer and restart the quiet-period timer.
    pub fn observe(&mut self, source: &str) {
        if self.last_sent_source.as_deref() == Some(source)
            || self.pending_source.as_deref() == Some(source)
        {
            return;
        }
        self.pending_source = Some(source.to_string());
        self.last_edit_at = Some(Instant::now());
    }

    /// True while a check is waiting for the quiet period or the worker.
    pub fn is_pending(&self) -> bool {
        self.pending_source.is_some()
    }

    fn ensure_worker(&mut self) {
        if self.job_tx.is_some() {
            return;
        }
        let (job_tx, job_rx) = channel::<Job>();
        let result_tx = self.result_tx.clone();
        let spawn = std::thread::Builder::new()
            .name("rustlab-syntax".to_string())
            .spawn(move || {
                while let Ok(job) = job_rx.recv() {
                    let diagnostics = analyze_syntax(&job.source);
                    if result_tx
                        .send(Report {
                            id: job.id,
                            source: job.source,
                            diagnostics,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            });
        if spawn.is_ok() {
            self.job_tx = Some(job_tx);
        }
    }

    /// Poll the worker and return fresh diagnostics for `current_source`.
    pub fn poll(&mut self, current_source: &str, now: Instant) -> Option<Vec<SyntaxDiagnostic>> {
        let mut newest = None;
        loop {
            match self.result_rx.try_recv() {
                Ok(report) => {
                    if report.id == self.latest_sent_id
                        && report.source == current_source
                        && self.pending_source.is_none()
                    {
                        newest = Some(report.diagnostics);
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        if let Some(source) = self.pending_source.clone() {
            let due = self
                .last_edit_at
                .map(|at| now.duration_since(at) >= SYNTAX_DEBOUNCE)
                .unwrap_or(true);
            if due {
                self.ensure_worker();
                self.latest_sent_id = self.latest_sent_id.saturating_add(1);
                self.last_sent_source = Some(source.clone());
                self.pending_source = None;
                self.last_edit_at = None;
                if let Some(tx) = &self.job_tx {
                    let _ = tx.send(Job {
                        id: self.latest_sent_id,
                        source,
                    });
                }
            }
        }
        newest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_source_has_no_diagnostics() {
        assert!(analyze_syntax("x = 1 + 2;\n").is_empty());
    }

    #[test]
    fn invalid_source_reports_position_and_run_message() {
        let source = "x = (1 + 2;\n";
        let diagnostics = analyze_syntax(source);
        assert_eq!(diagnostics.len(), 1);
        let first = &diagnostics[0];
        assert_eq!(first.line, 1);
        assert!(first.column >= 1);
        assert!(first.length >= 1);
        let run_message = rustlab_cli::format::format_eval_error(
            &rustlab_eval::EvalError::Parse(rustlab_parse::parse(source).unwrap_err()),
            source,
        );
        assert_eq!(first.message, run_message);
    }

    #[test]
    fn worker_returns_after_quiet_period() {
        let mut service = SyntaxService::default();
        service.observe("x = (1 + 2;\n");
        assert!(service.is_pending());
        let now = Instant::now();
        assert!(service.poll("x = (1 + 2;\n", now).is_none());
        let later = now + SYNTAX_DEBOUNCE + Duration::from_millis(20);
        assert!(service.poll("x = (1 + 2;\n", later).is_none());
        // Wait for the parser thread without blocking the UI loop in production.
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut result = None;
        while Instant::now() < deadline && result.is_none() {
            std::thread::sleep(Duration::from_millis(10));
            result = service.poll("x = (1 + 2;\n", Instant::now());
        }
        assert_eq!(result.map(|d| d.len()), Some(1));
    }
}
