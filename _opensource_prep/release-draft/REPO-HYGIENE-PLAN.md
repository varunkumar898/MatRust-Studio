# Repository hygiene plan (do not run without reviewing)

These commands are proposals only. They remove paths from the Git index but
leave local files in place. Review `git status` and make a backup before
running them.

## Currently tracked targets

The following are tracked: `build_debug.log`, `build_release.log`, `mylog.txt`,
`now.txt`, `MASTER-PROMPT.md`, `docs/MASTER-PROMPT.md`, one file under
`re-ghidra/`, and 287 paths under `docs/verify-logs/`. The two
`MASTER-PROMPT.md` files are similar but not byte-identical; compare and
reconcile them before removing either one.

Proposed index removal:

```powershell
git rm --cached -- build_debug.log build_release.log mylog.txt now.txt MASTER-PROMPT.md docs/MASTER-PROMPT.md
git rm -r --cached --ignore-unmatch -- re-ghidra docs/verify-logs
git status --short
```

This leaves files in the local working tree. Review each staged removal before
committing. Do not remove verification evidence until privacy-sensitive paths
have been checked and any required audit records are safely retained outside
the public repository.

## AI instruction file placement

`AGENTS.md` and `GEMINI.md` are currently at the repository root. Keep a short
root file if the corresponding tool needs automatic discovery; put longer
human-readable policy under `docs/ai-development/` and link to it from the
root instructions. Avoid maintaining two full copies of `MASTER-PROMPT.md`;
keep one canonical policy document and use a small pointer where discovery
requires it.

## Proposed `.gitignore` additions

The current ignore file already ignores `*.log` but explicitly re-allows
`docs/verify-logs/*.log`. Tracked logs stay tracked until removed from the
index. Add narrow root-level and evidence-folder rules rather than ignoring
all `.txt` files or all logs (some fixtures and documentation may be
intentional):

```gitignore
# Local build and scratch logs
/build_debug.log
/build_release.log
/mylog.txt
/now.txt

# Local analysis workspace and machine-specific verification transcripts
/re-ghidra/
/docs/verify-logs/
```

Review the contents of `docs/verify-logs/` first; this rule prevents new files
from being added, but does not stop tracking files already committed.

## Review commands

```powershell
git ls-files -- build_debug.log build_release.log mylog.txt now.txt MASTER-PROMPT.md docs/MASTER-PROMPT.md
git ls-files -- re-ghidra docs/verify-logs
git diff --cached --name-status
git status --short
```
