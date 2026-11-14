# RustLab Master Prompt (Legacy Reference)

> **Historical Reference Note:** This is the legacy master prompt preserved for rule reference per `docs/AGENT-PROTOCOL.md` P15.
> Active protocols are defined in `docs/AGENT-PROTOCOL.md` and active workplans in `docs/WORKPLAN.md` and `docs/WORKPLAN-PHASES.md`.

RUSTLAB MASTER PROMPT v1
(Read this whole file at the start of every session. Then read docs/STATE.md and docs/ENGINEERING-CHARTER.md.)

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
For block-diagram and circuit simulation (milestones M0..M12),
docs/ENGINEERING-CHARTER.md is the single source of truth.

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
R6 ONE TASK PER SESSION. One TASK per session; several tasks only with BATCH n,
   see P6 of docs/AGENT-PROTOCOL.md. Finish, update docs/STATE.md, commit, and
   STOP. Wait for approval. Do not start the next task.
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
   starts with a dot (use <workspace>/re/targets, <workspace>/re/projects). Logs are UTF-8.
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
REA runs in ANTIGRAVITY (config ~/.gemini/config/mcp_config.json,
command: node ~/.antigravity/rea/scripts/rea.mjs mcp). It is not
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
 2 Copy the binary to re/targets\ (no dotted path elements). Compute
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
   trend, strings (local paths like <user-home> leak via panic messages;
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
[OBSOLETE: Replaced by docs/WORKPLAN.md and docs/WORKPLAN-PHASES.md. Milestones G0 through M12 are completed.]

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
 3 [OBSOLETE: Replaced by current task assignment in docs/WORKPLAN.md.]
