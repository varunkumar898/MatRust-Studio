# Final pre-publish checklist (proposal)

| Item | Status | What must be done manually |
|---|---|---|
| Branch changes | Review needed | Review `audit-logs/branch-commit-groups.txt`. The branch is 45 commits ahead of `main`; decide which GUI/plot implementation commits belong in the release. No changes were reverted. |
| Latest mixed-scope commit | Review needed | `f863b14348a2` changes 25 files outside `_opensource_prep/`, including layout/surface-plot code and verification evidence. Decide whether to keep it or make a selective revert commit. |
| Username and local paths | Release blocker | Redact the working-tree and historical file occurrences listed in the privacy TSVs. Keep audit logs private. |
| Personal commit email | Release blocker if preserving history | Decide on a clean new repository or a coordinated `git-filter-repo` rewrite. The current reachable history has the personal identity in 60 commit metadata records. |
| Secrets | Not certified | This scan did not prove the absence of credentials. Run a dedicated secret scanner; revoke any real credential found. |
| Test manifest | Proposed, not installed | Review `_opensource_prep/docs/test-manifest.proposed.txt` (449 listed entries) and replace the old 393-entry manifest only after reviewing added/removed tests. |
| README test claim | Proposed, not installed | Replace the stale 336 figure using `_opensource_prep/docs/README-test-count.proposed.md`; do not claim tests pass without running the release revision. |
| CI manifest guard | Gap identified | CI compares counts only. Add a name-level set comparison so equal counts cannot hide different test names. |
| Dependency licenses | Review required | Review BSL-1.0, OFL-1.1, Ubuntu-font-1.0, and Unlicense packages and preserve third-party/font notices in source and binary distributions. No GPL-family license was found in the current inventory. |
| Source-code license | Verify before release | Confirm both license texts and every crate’s manifest declaration are included in the release tree. |
| Safety and compatibility claims | Review required | Keep the Alpha qualifier; confirm coverage percentages, MSRV, and partial safety-audit wording against the exact release commit. |
| Screenshots and fixtures | Review required | Check each image/fixture for private data and confirm all real `.mlapp`/`.slx` samples are excluded. |
| Names and trademarks | Review required | Verify crate/PyPI names and the MatRust name are available/appropriate; retain the trademark disclaimer and seek legal advice if needed. |
| Release repository and settings | Not configured here | Choose public visibility; protect `main`; require CI; enable Issues, Discussions, private vulnerability reporting, and Dependabot alerts. |
| Release process | Not started | Create reviewed `v0.1.0-alpha` notes and verify the release archive contains no prep files, build output, user-specific logs, or secrets. |
| Push/rewrite/revert | Not done | No push, history rewrite, or revert was performed in this task. |
