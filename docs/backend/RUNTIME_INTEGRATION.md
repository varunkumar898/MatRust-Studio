# MatRust Studio Runtime Integration

## 1. Unified Runtime Principles

1. **Single Execution Engine**: There is only one execution runtime across the entire application. The Command Window, the Editor, the Import Tool, Dataset Explorer, and simulation/robotics tools all dispatch commands and queries into the shared `Session` and `Interpreter`.
2. **True State Persistence**: Any variable defined in the Command Window (e.g. `A = [1 2; 3 4]`) is immediately accessible to scripts, workspace inspection, figure plotting, and dataset inspection tools.
3. **No Numerical Fabrication**: No GUI component computes numerical approximations inside egui render loops. All calculations flow through validated backend routines.
4. **Asynchronous Non-Blocking Execution**: Computations run in a dedicated background worker (`rustlab-worker`). The GUI remains 100% interactive, capable of rendering animations, responding to cancellation, or pausing at breakpoints.

## 2. Execution Sequence Diagram

```text
User Action (Command/Script/App)
       │
       ▼
egui Event Handler (UI Thread)
       │
       ▼ WorkerRequest::Execute
WorkerClient Channel
       │
       ▼
Session::exec (Background Thread)
       │
       ▼
Parser -> AST -> Interpreter::eval
       │
       ▼
Standard Library Builtin Dispatch
(Linalg / Stats / Numerical / ML / Robotics)
       │
       ▼
Environment State Update (Variables stored)
       │
       ▼
GraphicsState Update (Plots added to FigureState)
       │
       ▼ WorkerResponse::ExecutionFinished
WorkerClient Receiver (UI Thread Poll)
       │
       ▼
Workspace Browser, Command Output, and Figures Panel updated
```

## 3. Panic Resilience & State Recovery

The execution worker wraps evaluation in `std::panic::catch_unwind`.
When an unhandled panic occurs:
1. The panic is trapped before crossing thread boundaries.
2. The session captures the panic diagnostic message.
3. The session is safely restored, preventing corrupt interpreter state.
4. The IDE displays an error notification in the status bar and Command Window.
5. Subsequent commands continue to evaluate cleanly.
