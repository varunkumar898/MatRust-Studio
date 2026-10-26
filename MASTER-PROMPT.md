<USER_REQUEST>
RUSTLAB MASTER PROMPT v1
(Read this whole file at the start of every session. Then read docs/STATE.md.)

====================================================================
0. MISSION
====================================================================
Build RustLab: a MATLAB-compatible numerical computing platform in Rust
(own clean code), with these 8 capability areas:
 1 Mathematics and calculations      5 AI and machine learning
 2 Data analysis and visualization   6 Robotics and automation
 3 Engineering simulation            7 Programming and algorithm development
 4 Signal and image processing       8 Hardware and embedded systems
Done so far (see docs/STATE.md): lexer/parser, interpreter, column-major
math, stdlib, MAT-file I/O, CLI/REPL, function handles, numeric classes,
N-D arrays, sparse, linear algebra, FFT, statistics + CSV (B2.1.1).
The owner is learning. Every milestone must teach: explain concepts in plain
words, with tiny examples, so the owner can read, debug and extend the code.

LEGAL / CLEAN-ROOM: MATLAB behavior comes only from public documentation,
black-box experiments on outputs you can legitimately obtain, and open
references (GNU Octave docs/behavior, SciPy, LAPACK/netlib). Never copy GPL
code. Never reverse engineer, disassemble or decompile any MathWorks binary,
P-code, or license/protection code. (I am not a lawyer; stay conservative.)

====================================================================
1. HARD RULES (breaking one = stop and report it)
====================================================================
R1 EVIDENCE. Any claim "passes / works / measured" must come from a command
   you ran in THIS session. Save raw output to a file under
   docs/verify-logs/ or bench-results/<milestone>/ (UTF-8), and in your
   report give: the command, the file path, the file SHA-256, and the
   specific lines asked for. Never retype, reformat, abbreviate, or
   "reconstruct" logs or test lists.
R2 Never cite a test name, file, line number, or number you have not just
   printed in this session. Print first, cite second.
R3 UNVERIFIED. If you cannot source a MATLAB behavior (display format, error
   text, error id, algorithm default), write UNVERIFIED in the docs, add
   _unverified to the test name, and add it to docs/BACKLOG.md. Valid sources:
   a MathWorks doc URL you actually opened, or real output from Octave/SciPy
   run on this machine (record the version). Octave is not MATLAB: say which.
R4 TESTS NEVER DECREASE. docs/test-manifest.txt (from cargo test -- --list)
   lists every test. A test may disappear only by an explicit manifest edit
   with a one-line written reason. Per-test explanation required.
R5 SCOPE LOCK. Do only the current milestone. Anything else you notice goes in
   docs/BACKLOG.md. Do not change older milestone behavior without a test
   proving the change and a mention in the report.
R6 ONE MILESTONE PER SESSION. Finish, update docs/STATE.md, commit, tag, and
   STOP. Wait for approval. Do not start the next milestone.
R7 GIT. Commit at every green step. Tag each milestone (e.g. b2.2). Report
   git log --oneline -n 5 and git status.
R8 PERFORMANCE. Release profile only. Say for each number: Rust-API level or
   interpreter level; thread count; machine idle (close other apps); 3 runs;
   median and spread; compared with the contract and with the previous
   recorded value. If a contract is missed, the FIRST line of that section is
   "MISSED". Never mix numbers from different harnesses in one table.
   Warm-up trends (run 1 slower than run 3) mean the harness is wrong: fix it.
R9 ENVIRONMENT (Windows ARM64, PowerShell). Run once:
   git config --global core.pager cat. Never run a command that opens a pager.
   Do not put Ghidra projects or RE targets under any path whose element
   starts with a dot (use C:\Users\varun\Desktop\MATSTUDIO\re\targets, C:\Users\varun\Desktop\MATSTUDIO\re\projects). Logs are UTF-8.
   If your tools are in "plan mode" and reject writes, say so and ask the
   owner to switch; do not pretend the write happened.
R10 DEPENDENCIES. MIT/Apache-2.0/BSD/ISC/Zlib/Unicode only. Every new crate:
   one-line justification, version, license, maintenance note; run
   cargo deny check licenses and save the output. Heavy crates behind features.
R11 QUALITY GATES (all must pass, from cargo clean, via scripts/verify.ps1):
   fmt; clippy -D warnings with --all-features and --no-default-features;
   tests in default, all-features, no-default-features; manifest check.
   Library crates deny unwrap/expect/panic/todo/unimplemented. No unsafe
   without a written justification.
R12 IDS. Error/warning ids that MathWorks documents keep the MATLAB: prefix.
   RustLab-only ids use RUSTLAB:. Every id, message and warning lives in
   docs/spec/error-catalog.md with a source or UNVERIFIED. Settle each id once.
R13 FEATURE FLAGS must not change core semantics. A feature adds functions; it
   must not silently replace core ones. --no-default-features must still build
   and pass its tests.
R14 HONESTY. If you made a mistake, say so first, fix it, and record it in
   docs/milestones/<id>-retractions.md. Do not defend wrong numbers.
R15 SMALL STEPS. Work in steps of at most ~150 lines of change; run the
   fast test for that step; commit. Update docs/STATE.md after every step.

====================================================================
2. CORE CONCEPTS (read; use them to debug yourself)
====================================================================
2.1 Pipeline. source text -> lexer (tokens) -> parser (AST with spans) ->
    evaluator (walks AST, keeps an Environment of variables) -> Value
    (Double, Single, Int8..UInt64, Logical, Char, Cell, Struct, Sparse,
    FunctionHandle) -> builtin functions (stdlib/data/...) -> display
    formatter -> terminal. Errors carry a span, and the CLI draws a caret.
    Crates: parse, math (arrays, Value, ops), eval (interpreter), stdlib
    (builtins), matio (.mat), data (statistics, CSV), cli (REPL, bench).
    New areas become new crates behind cargo features; core crates never
    depend on plotting, ML or hardware crates.
2.2 Invariants (each has a one-line probe; keep them as golden scripts in
    tests/golden/*.m with *.expected files):
 I1  Column-major, 1-based: A=[1 2;3 4]; A(:)'  -> 1 3 2 4
 I2  end is resolved per subscript with a context stack: x=1:5; x(end-1) -> 4
 I3  Closures capture by value at creation: a=5; f=@(x)x+a; a=100; f(1) -> 6
 I4  Integer classes saturate and round half away from zero:
     uint8(250)+uint8(10) -> 255; int8(-2.5) -> -3;
     int8(1)+int16(1) -> error (integer class mismatch)
 I5  NaN ~= NaN; x=[1 NaN 3]; x==x -> 1 0 1
 I6  && and || short-circuit (scalars); & and | are elementwise
 I7  Format recycling: sprintf('%d,',[1 2 3]) -> 1,2,3,
 I8  Broadcasting: [1 2 3]+[10;20] -> 2x3 matrix
 I9  Logical masks: A(A>2) is a column for matrix A; A(mask)=[] deletes
 I10 Deletion: A(:,2)=[] removes a column; reshape(1:6,[],3) is 2x3,
     reshape(1:6,-1,3) is an error
 I11 Singular matrix: inv([1 2;2 4]) gives a WARNING and Inf, not an error
 I12 Warnings go to stderr and do not stop execution; errors stop it
 I13 Values are cloned on assignment unless moved; the interpreter borrows
     identifiers in binary expressions (fast path). Do not reintroduce clones.
 I14 Same input -> same output (deterministic); random functions use a seeded
     documented PRNG (rng(seed)); it is not MATLAB's stream. Say so.
 I15 Display ("format short") is table-driven in ONE module with golden tests.
     Known-good: integers print as integers; non-integers print 4 decimals;
     12345.678 -> 1.2346e+04. Everything else (0.001, 1234.5678, mixed
     matrices, common scale factors) is UNVERIFIED until checked with Octave
     or a MathWorks page; record the source in docs/spec/display-rules.md.
2.3 DEBUG PLAYBOOK (follow in order, write each step in your notes):
 1 Reproduce with the smallest script. Save it as tests/golden/bug_<id>.m.
 2 Locate the layer with probes: dump tokens, dump AST, trace evaluation,
   print the Value (class, shape, first elements). The bug is in the first
   layer whose output is already wrong.
 3 State a hypothesis and one experiment that would disprove it.
 4 Fix the smallest thing. Do not refactor while fixing.
 5 Add a regression test that fails before the fix and passes after
   (show both runs).
 6 Run scripts/verify.ps1 from cargo clean.
 7 Write the cause and lesson in the milestone's LEARNING.md "Debugging notes".
2.4 BUILT-IN SELF-DEBUG TOOLS (milestone G1 creates them; use them forever):
 rustlab --dump-tokens FILE, --dump-ast FILE, --trace FILE (statement by
 statement values), :debug on|off in the REPL, rustlab doctor (versions,
 features, builtin count, docs/help coverage, manifest check, binary SHA-256),
 rustlab selftest (runs every tests/golden/*.m and compares to .expected,
 prints PASS/FAIL per file), rustlab explain <error-id> (prints catalog entry,
 source, and a suggested fix), and a panic hook that prints the last
 statement and a ready-to-paste bug report.

====================================================================
3. REA (reverse-engineering analysis MCP) - HOW TO USE IT
====================================================================
REA runs in ANTIGRAVITY (config C:\Users\varun\.gemini\config\mcp_config.json,
command: node C:\Users\varun\.antigravity\rea\scripts\rea.mjs mcp). It is not
configured in opencode. Tool names seen so far: open_binary, binary_session,
close_binary, analyze_function, address_name, compare_bundles,
inspect_sqlite_database (about 116 tools; list the real names first).
It is for analyzing OUR OWN binaries only: target\release\rustlab.exe, C code
generated by rustlab (compiled with gcc/clang), our own libraries. Never a
MathWorks binary, P-code, or license code. Open-source binaries only for
comparison, with owner approval.
SESSION PROTOCOL (an "R-check"):
 1 List tool names (paste exact names or the exact error). If the server is
   unavailable, report the exact error, mark the R-check SKIPPED (reason),
   and continue the milestone. Never substitute another tool silently.
 2 Copy the binary to C:\Users\varun\Desktop\MATSTUDIO\re\targets\ (no dotted path elements). Compute
   SHA-256 and size in the shell.
 3 open_binary. The session hash must equal the shell hash. If not: STOP and
   report; do not continue with a mismatched binary.
 4 Run only the analysis the milestone asks for (below).
 5 close_binary.
 6 Append a row for every call to docs/re-notes/CALL-LOG.md:
   timestamp, tool, arguments, target + SHA-256, one-line result, purpose.
 7 Write findings as plain English in docs/re-notes/<milestone>-<topic>.md:
   what you looked at, what you saw, what you changed because of it. No pasted
   disassembly or decompiler output or pseudo-code. State which binary build
   (hash) each note applies to; mark notes about older builds as such.
 8 Honest roles: one agent plays both Analyst and Implementer; separation is
   by written notes only. Say so; do not claim more.
WHAT REA CHECKS PER MILESTONE:
 - Hot loops (elementwise add, GEMM, FFT wrapper, new stats loops): vectorized
   or scalar, bounds checks left, whether public functions inline into the
   fast path (this explains the Value::add 6.34 ms vs 0.774 ms mystery).
 - Hygiene: imports (a numeric core should not import networking), binary size
   trend, strings (local paths like C:\Users\varun leak via panic messages;
   fix with --remap-path-prefix in release builds).
 - B8: generated C binary: exported symbols, imports, stack usage, no heap
   use in the restricted subset.

====================================================================
4. MILESTONE PROCEDURE (copy this checklist into docs/milestones/<id>.md)
====================================================================
 1 Read docs/STATE.md, docs/BACKLOG.md, this file. Run scripts/verify.ps1 to
   confirm the starting state (record test counts).
 2 Write the design note: what, why, alternatives, scope IN/OUT, file layout,
   public API, error ids, risks. (No code yet.)
 3 Write golden known-answer tests FIRST (values below or from cited sources).
 4 Implement in small steps (R15). Keep core semantics intact (R13).
 5 For every new builtin: at least 2 tests (golden + edge: empty, scalar,
   NaN/Inf, wrong argument count/type with exact error text, integer classes,
   N-D with dim where relevant), docs/functions/<name>.md, help entry,
   catalog entries. The docs/help coverage test must stay green both ways.
 6 Examples: examples/<area>/*.m, run them for real, save real output
   (examples/<area>/*.out), mention nothing that was not run.
 7 LEARNING.md section: concept in plain words; tiny example; code walk-through
   (key functions); how to debug it; common mistakes; exercises (3).
 8 Benchmarks where speed matters (R8). R-check with REA if code in 3 changed.
 9 scripts/verify.ps1 from cargo clean; save the UTF-8 log; update
   docs/test-manifest.txt and docs/test-count.md.
10 Update docs/STATE.md (current milestone, counts, hashes, open issues).
11 Commit, tag, REPORT (section 6), STOP.

====================================================================
5. ROADMAP (strict order; one milestone per session)
====================================================================
G0 CLOSEOUT of B2.1.1 (no new features)
 a Answer the owed items with raw evidence: purpose of every file in
   bench-results/a312/ and the one-line Cargo.toml additions (rustlab-cli,
   rustlab-matio, rustlab-stdlib, workspace); cargo deny check licenses
   output; reconcile hashes (5F12F389... vs 8fd8d28d..., both 5,177,856 B);
   mark gemm-codegen.md with its build.
 b Real REPL transcript for: x=100000; 12345.678; 1234.5678; 0.001; 0.0012;
   [1e-16 1]; [1 1000000]; [0.001 1000]; [1.5 22]; int8(-2.5); uint8(3.5);
   A=[1 2 3]; A([true false true])=[9 9 9]; 0/0; 0.1+0.2==0.3;
   inv([1 2;2 4]); prctile([1 2 3 4 5],25) (expect 1.75);
   prctile([12.5 18.2 14.1 22 19.8 14.1 16.5],75) (expect 19.4);
   unique([3 1 2 1]) (row output). Compare with Octave where installed and
   record the Octave version; mark UNVERIFIED where it differs or is absent.
 c Fix contract misses: LU 100x100 <= 0.6 ms; interpreter A+B 1e6 <= 3 ms;
   movmean/cummax/zscore on 1e6 O(n) and about 10 ms or less in steady state;
   fair dense baseline in the sparse table; GEMM numbers with thread count.
 d Rename RustLab-only ids to RUSTLAB:. Move 'omitnan' into core sum/mean/
   min/max (R13) so --no-default-features keeps the same semantics.
   Fix csv version text.
 e First real REA R-check on hot loops (section 3), including the public
   Value::add vs fast-path question. Fix the public path if it is slow.
G1 SELF-DEBUG TOOLS (section 2.4) + tests/golden corpus covering I1..I15 +
   docs/LEARNING-core.md explaining sections 2.1-2.3 with worked examples.
B2.2 rustlab-plot: plot-description model (Figure, Axes, Line, Scatter, Bar,
   Histogram, text, legend, limits, scales) independent of any renderer;
   deterministic SVG renderer (fixed number formatting, documented "nice
   ticks" rule, stable ids); builtins figure, plot, scatter, bar, hist, hold,
   subplot, xlabel, ylabel, title, legend, xlim, ylim, grid, saveas('x.svg').
   Snapshot tests (stored .svg files, byte-equal). PNG via resvg is a later
   optional feature. No GUI, no 3D, no timetables.
B2.3 Tables and file formats: table type with typed columns, T.Var access,
   T(mask,:), readtable/writetable (CSV with type inference), sortrows/groupby/
   join, missing values, display; XLSX read via calamine; JSON via serde_json
   (jsonencode/jsondecode). Plain-text fixtures in tests/data.
B1 rustlab-math-tools: fzero (Brent), fsolve (Newton/Broyden), roots
   (companion matrix), polyfit/polyval, interp1 (linear, nearest, spline,
   pchip), spline, diff/gradient, trapz/cumtrapz, integral (adaptive
   Gauss-Kronrod), ode45 (Dormand-Prince), ode23, a stiff solver, fminsearch,
   fminunc (BFGS), lsqnonlin (Levenberg-Marquardt), linprog (simplex),
   quadprog. Symbolic math is OUT (design note only).
   Goldens: fzero(@cos,[1 2]) ~ 1.5708; roots([1 -3 2]) = [2;1];
   polyfit(1:3,[2 4 6],1) ~ [2 0]; integral(@(x)x.^2,0,1) ~ 1/3;
   ode45 for y'=-y, y(0)=1 gives y(1) ~ 0.3679 (tolerance per RelTol/AbsTol);
   fminsearch of (x-1)^2+(y-2)^2 from [0 0] ~ [1 2].
B4 rustlab-signal: filter, filtfilt, butter, cheby1, fir1, freqz, windows
   (hamming, hann, blackman), conv/conv2, xcorr, resample, pwelch,
   spectrogram, periodogram, wavread/wavwrite (hound), imread/imwrite (image
   crate, png/jpeg only), rgb2gray, imresize, imfilter, Sobel, Canny,
   Otsu threshold, morphology, histeq. Images are uint8/double N-D arrays.
   Goldens: filter([1 1]/2,1,[1 2 3 4]) = [0.5 1.5 2.5 3.5];
   butter(2,0.5): b ~ [0.2929 0.5858 0.2929], a ~ [1 0 0.1716];
   hamming(3) = [0.08;1;0.08].
B3 rustlab-sim: tf, ss, zpk, conversions, pole, zero, dcgain, series,
   parallel, feedback, c2d (ZOH), step, impulse, lsim (matrix exponential),
   bode and nyquist DATA, margin; fixed-step and variable-step block-diagram
   engine; example .m systems with expected-behavior tests: RLC circuit, DC
   motor, mass-spring-damper, thermal system, inverted pendulum.
   Goldens: step response of 1/(s+1) at t=1 ~ 0.6321; poles of s^2+2s+2 are
   -1 +/- 1i.
B5 rustlab-ml: preprocessing (normalize, split, k-fold), linear regression,
   logistic regression, k-NN, k-means (seeded), decision tree (CART), random
   forest, linear SVM, PCA (via SVD), naive Bayes, small MLP with
   backpropagation, z-score and isolation-forest anomaly detection. Default:
   from scratch, no linfa, because the owner wants to study the algorithms.
   fit/predict/score API plus Rust traits. Image demo uses a synthetic
   8x8-digit generator (no downloads). LEARNING.md explains backprop step by
   step with a hand-computed 2-layer example that a test reproduces.
B6 rustlab-robotics: rotations (rotx/roty/rotz, Euler, axis-angle,
   quaternions), SE(3), DH parameters, forward/inverse kinematics (analytic
   2-link and 3-link, numeric Jacobian for general), polynomial / trapezoidal
   / minimum-jerk trajectories, PID with anti-windup, LQR via Riccati,
   A*, RRT (seeded), differential-drive and bicycle models.
   Golden: double integrator A=[0 1;0 0], B=[0;1], Q=I, R=1 gives
   K ~ [1 1.7321].
B7 rustlab-lang: script/function loading and path, run/exist/which, tic/toc
   and a profiler, debugger hooks (breakpoints, step, inspect), runtests with
   assert-based tests, parfor with rayon (deterministic results), containers.Map,
   struct/cell display, strings, eval/evalc, num2cell/cell2mat; classdef subset
   and a bytecode VM as DESIGN NOTES first.
B8 rustlab-hw: serial (serialport), ring buffers with timestamps, TCP/UDP,
   MQTT (optional), all behind traits with mocks (tests need no devices);
   C99 code generation from a restricted statically typed subset (fixed-size
   arrays, no dynamic features), readable and MISRA-friendly, with a test
   harness that compiles with gcc/clang and compares to the interpreter to
   1e-12. R-check the generated binary with REA (symbols, imports, stack).
APP The user-facing app: packaged CLI (release zip, versioned), REPL polish
   (help, tab completion, history), optional egui shell (script editor,
   workspace viewer, plot window) as the last milestone.
Language completeness backlog (schedule inside B7 or earlier when a milestone
   needs it): everything recorded in docs/BACKLOG.md.

====================================================================
6. REPORT TEMPLATE (use exactly; keep it short; evidence in files)
====================================================================
 1 Milestone id, tag, commit hash.
 2 One-line verdict: DONE / DONE WITH MISSES / BLOCKED. List every MISSED
   contract and every UNVERIFIED item up front.
 3 Test counts: default / all-features / no-default (from "test result"
   lines only), previous counts, and the name diff of --list between
   default and no-default (explain each name).
 4 Evidence table: item | command | output file | SHA-256 of file.
 5 What I learned / concepts added (point to LEARNING.md section).
 6 Bugs found and fixed (cause, regression test name).
 7 Benchmarks (R8 format), or "no benchmarks in this milestone".
 8 R-check summary (or SKIPPED with reason) and CALL-LOG rows added.
 9 Open issues and BACKLOG additions.
10 "Not done" list. Nothing is allowed to be silently missing.
Then STOP.

====================================================================
7. MISTAKES PREVIOUS SESSIONS MADE - DO NOT REPEAT
====================================================================
 - Retyping or summarizing logs; citing tests/line numbers never printed.
 - Concluding "server absent" from a tool-name error; mixing up which host
   (Antigravity vs opencode) has REA.
 - Mixing benchmark harnesses, profiles, thread counts; claiming a contract
   was met when it was missed; quoting best warm-run numbers.
 - Inventing MATLAB error ids, messages or display outputs.
 - Putting new features into the wrong crate or milestone (plotting in data,
   tables in B2.1, features that override core builtins).
 - Letting a feature flag change core semantics.
 - Stopping early because of plan mode or a pager and not saying so.
 - Calling two roles in one agent "separation".
 - Moving on to the next milestone without approval.

====================================================================
8. START NOW
====================================================================
 1 git config --global core.pager cat. Read this file, docs/STATE.md,
   docs/BACKLOG.md. If STATE.md does not exist, create it from the facts you
   can verify (git log, tags, test counts from a real run).
 2 Run scripts/verify.ps1 from cargo clean and save the log.
 3 Start milestone G0 only. Follow section 4. Report with section 6. STOP.
</USER_REQUEST>
<ADDITIONAL_METADATA>
The current local time is: 2026-10-25T19:46:04+05:30.
</ADDITIONAL_METADATA>
