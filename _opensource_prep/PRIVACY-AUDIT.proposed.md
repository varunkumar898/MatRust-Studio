# Privacy and history audit (proposal)

## Scope and findings

The working-tree scan covers UTF-8/text files outside `.git/` and `target/`,
including ignored logs, but excludes the generated `privacy-*.tsv` inventory
files to avoid the report counting itself. It searched for the Windows account
name and common project-account variants, Windows paths under `Users`, and
email addresses. Matches overlap when a line contains both a username and a
path.

| Location | Results |
|---|---|
| Current working tree | 2,801 match rows in 78 files: 1,407 username matches, 1,392 Windows-path matches, and 2 email-address matches. |
| Reachable Git history | 7,045 matching changed-file snapshot rows in 76 files: 3,543 username matches, 3,498 Windows-path matches, and 4 email-address matches. Commit and parent snapshots can both be reported for a changed file. |
| Commit metadata | A personal Gmail identity is present in 60 reachable commits. The exact commit hashes and redacted author/committer fields are in `audit-logs/privacy-gmail-commits.tsv`. Git metadata has no source-file line number. |

The two current file-content email matches and four historical file-content
email matches are project-local placeholder identities, not the personal
Gmail address. The personal email appears in Git author/committer metadata.
The inventory masks email local-parts and does not store the matched source
line text.

Large concentrations of local paths are in verification and benchmark logs,
including:

- `docs/verify-logs/2026-11-03-gui-s1-baseline-verify.log` (many path lines)
- `docs/verify-logs/2026-10-31-verify.log`
- `docs/verify-logs/2026-11-03-gui-s2-final-verify.log`
- `docs/verify-logs/2026-11-03-gui-s2-hygiene-verify.log`
- `bench-results/a312/verify-a312-final.utf8.log`
- `docs/re-notes/CALL-LOG.md`

The exact file, commit, line, and match category are in
`audit-logs/privacy-working-tree.tsv` and
`audit-logs/privacy-history.tsv`. These tables intentionally do not reproduce
the private path or email values. Some username matches are intentional
project/owner references rather than local filesystem paths.

## Required cleanup before public release

1. Redact local paths from the working-tree logs and any project documentation
   that should be public. Re-scan the full staging tree, including ignored
   files that may accidentally be copied.
2. Decide whether to remove or anonymize personal commit metadata. Rewriting
   Git history changes commit hashes and requires coordinated replacement of
   every published branch and tag; see `HISTORY-CLEANUP-PLAN.proposed.md`.
3. Keep the audit tables and raw logs private. Some command logs under
   `_opensource_prep/audit-logs/` also contain local paths; do not copy the
   audit folder into the public repository.
4. No credential was established by this username/path/email scan. Run a
   separate secret scanner before release. If a real credential is found,
   revoke/rotate it first; rewriting history alone does not invalidate it.
