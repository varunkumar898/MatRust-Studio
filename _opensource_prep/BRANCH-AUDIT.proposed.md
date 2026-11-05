# Branch audit (proposal)

Scope: `gui/polish-1`, HEAD `81c28b0a02098b4dcdec4f9f6208240946d50191`,
compared with `main`. The branch is 45 commits ahead. The exhaustive
commit/file/status/subject inventory is in
`audit-logs/branch-commit-groups.txt` (397 non-prep path-change records);
`audit-logs/branch-files-main-to-head.tsv` provides a tabular alternative.
The inventory groups files under the commit that changed them, so the commit
subject explains the purpose of that individual change.

The commits collectively implement desktop/editor and file-browser work,
plotting and surface plots, parser/statistics fixes, test and snapshot
updates, design notes, and verification evidence. These are implementation
changes, not just launch-preparation documents.

## Latest commits since `04ec463`

Commit `f863b14348a2` is titled “Add verification logs for plot staged
surface, style commit, and tests after cleanup,” but changes 25 files outside
`_opensource_prep/`:

- `apps/desktop/src/dock.rs` — desktop docking/layout wiring.
- `apps/desktop/src/layout_contract.rs` — new layout contract module.
- `apps/desktop/src/lib.rs` — exports the new module.
- `apps/desktop/src/ui_registry.rs` — UI registry changes for the layout.
- `crates/rustlab-plot/src/lib.rs` — surface-plot model changes.
- `crates/rustlab-stdlib/src/plot_builtins.rs` — surface plotting builtin changes.
- `docs/layout-audit.md` — layout design/audit notes.
- 18 `docs/verify-logs/2026-11-04-*` files — plot, layout, test, and style
  command evidence. Their exact names are in the exhaustive TSV inventory.

So the commit message understates its scope: it includes code and design
changes as well as logs. The preceding commit `04ec463` changes two files
outside `_opensource_prep/`, both plot milestone documentation files; their
names and status are in the same inventory.

Commit `81c28b0` only adds a launch-preparation README under
`_opensource_prep/`.

## Revert plan (not executed)

1. Decide whether the plot/layout code and prior GUI/plot commits are intended
   product work. Do not treat their presence as accidental based only on the
   latest commit message.
2. If only `f863b14`'s non-prep changes are unwanted, prepare a new, reviewed
   revert commit limited to its 25 listed non-prep paths. Preserve the
   `_opensource_prep/` files in that commit.
3. If the entire branch is unwanted, review the complete 45-commit inventory
   first, then make a new branch or explicit revert commits. Do not reset or
   rewrite shared history without owner approval.
4. Run the normal checks on the resulting branch before merging. No revert or
   push was performed for this audit.
