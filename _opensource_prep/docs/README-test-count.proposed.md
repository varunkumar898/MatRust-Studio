## Suggested README verification wording

The current workspace test listing contains **449 active test entries**,
reported by `cargo test --workspace -- --list`. This is a count of listed
tests, not a coverage percentage. The checked-in `docs/test-manifest.txt`
currently contains 393 entries and should be reconciled with the proposed
manifest before publishing. CI currently compares only the two counts; it
does not check whether the test names match.

The earlier README statement of 336 tests is stale and should be removed.
Avoid saying that 449 tests pass unless the three test commands below have
actually been run successfully for the revision being released.

### Regenerate the manifest (PowerShell)

From the repository root, the in-place command is:

```powershell
$current_tests = (cargo test --workspace --locked -- --list | Select-String "^\w.*: test$") |
    ForEach-Object { $_.Line.Trim() }
$current_tests | Set-Content -Encoding utf8 docs/test-manifest.txt
```

For review without modifying the existing manifest, change the destination to
`_opensource_prep\docs\test-manifest.proposed.txt`. The proposed file in this
directory was produced from the current workspace listing with `--locked`.
The same test-list command also runs in `scripts/verify.ps1` (currently without
`--locked`); inspect its output and review test additions/removals before
accepting a regenerated baseline.
