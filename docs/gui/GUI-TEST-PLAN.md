# RustLab Desktop GUI Test Plan

**Document:** `docs/gui/GUI-TEST-PLAN.md`  
**Target Milestone:** APP / MATSTUDIO Native Desktop Shell  
**Status:** TEST SPECIFICATION & ACCEPTANCE CRITERIA  

---

## 1. Test Levels & Strategy

The desktop GUI test suite verifies that the interface is completely operational, responsive, crash-resilient, and faithfully connected to the RustLab numerical engine.

```
┌────────────────────────────────────────────────────────┐
│                   GUI SMOKE TESTS                      │
│   Window initialization, panel layout, theme render    │
├────────────────────────────────────────────────────────┤
│             INTEGRATION & PIPELINE TESTS               │
│   Worker thread IPC, script execution, workspace sync  │
├────────────────────────────────────────────────────────┤
│                   UNIT TESTS                           │
│   Document tabs, action dispatch, history ring buffers │
└────────────────────────────────────────────────────────┘
```

---

## 2. Unit Testing Matrix

| Test Suite | Module | Test Cases & Verifications | Expected Outcome |
| :--- | :--- | :--- | :--- |
| **`test_document_lifecycle`** | `editor.rs` | Create new document, modify text, verify dirty flag, save to file, close tab. | Dirty flag updates; file writes cleanly. |
| **`test_command_history`** | `history.rs` | Add commands, navigate Up/Down, test deduplication and maximum buffer capacity. | Deterministic recall; bounds respected. |
| **`test_action_dispatch`** | `actions.rs` | Dispatch `NewScript`, `Save`, `Run`, `ClearConsole`; assert action state transitions. | Central state mutates correctly. |
| **`test_workspace_record_format`** | `workspace.rs` | Convert `rustlab_math::Value` variants (Double, Complex, Sparse, Struct) to records. | Accurate size, class name, and preview. |
| **`test_theme_palette`** | `theme.rs` | Verify dark and light theme contrast ratios and monospaced font mappings. | Meets WCAG AAA readability. |

---

## 3. Integration & Engine Pipeline Tests

Integration tests execute against a real headless background worker thread:

1. **`test_worker_eval_expression`**:
   - Transmit: `ExecutionRequest::EvalCommand("x = [1 2; 3 4]; y = inv(x)")`.
   - Assert: `ExecutionResponse::Output` contains formatted inverse matrix.
   - Assert: `ExecutionResponse::WorkspaceUpdated` contains variables `x` and `y` with 2x2 double attributes.
2. **`test_worker_syntax_error_reporting`**:
   - Transmit: `ExecutionRequest::EvalCommand("x = [1 2; 3")`.
   - Assert: Receives `ExecutionResponse::Error` containing line/column caret pointers without worker crash.
3. **`test_worker_thread_isolation`**:
   - Execute a long-running calculation; verify the UI message receiver does not block and continues polling smoothly.
4. **`test_worker_clear_workspace`**:
   - Execute `clear`; assert `WorkspaceUpdated` reflects empty symbol table.

---

## 4. Quality Gates & Acceptance Criteria

All tests must satisfy the RustLab Quality Gates:
- `cargo fmt --check`: 100% compliant.
- `cargo clippy --all-targets --all-features -- -D warnings`: Zero warnings.
- `cargo test`: All existing 212 numerical golden tests must pass without regression.
- Zero mock interpreters: Every calculation must be executed by `rustlab-eval` and `rustlab-stdlib`.
