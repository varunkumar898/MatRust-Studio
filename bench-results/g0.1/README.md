# G0.1-VERIFY benchmark and transcript evidence

**Canonical binary:** `re/targets/rustlab-g0.1-verified.exe`
(5,416,960 bytes, SHA-256
`80E6AF2442D97CED5E67145D0DF2B9961EF5957ED178049C689457B67CD5CA58`).

| Path | What it is | Canonical? |
| :--- | :--- | :--- |
| `bench-run1.txt` | Measured run 1, canonical binary | Yes |
| `bench-run2.txt` | Measured run 2, canonical binary | Yes |
| `bench-run3.txt` | Measured run 3, canonical binary | Yes |
| `repl.txt` | 19-prompt CLI transcript, canonical binary | Yes |
| `cold/bench-run1.txt` | First invocation after the release build (cold OS/AV cache) | No |
| `cold/bench-run2.txt` | Second invocation of the cold sequence | No |
| `cold/bench-run3.txt` | Third invocation of the cold sequence | No |
| `preclean-F6A302EB/bench-run1.txt` | Measured run with the pre-`cargo clean` build `F6A302EB...` | No |
| `preclean-F6A302EB/bench-run2.txt` | Measured run with the pre-`cargo clean` build `F6A302EB...` | No |
| `preclean-F6A302EB/bench-run3.txt` | Measured run with the pre-`cargo clean` build `F6A302EB...` | No |

The canonical three-run medians and spreads are parsed in
`docs/verify-logs/2026-10-28-g0.1-verify-bench-summary.txt`. The first
invocation after a release build was a cold-cache outlier and was not used for
the final table; the three canonical runs are the measured set.

> **Encoding note.** The benchmark logs were captured through the Windows
> console code page, so the `±` and `µs` glyphs appear as `┬▒` and `┬╡`. The
> numeric columns and all ASCII text are unchanged, and the files are UTF-8.
> The raw logs are retained exactly as captured rather than rewritten.

The 1,000,000-row CSV workload is verified separately:
`docs/verify-logs/2026-10-28-g0.1-verify-csv-1m-rows.txt` (1,000,000 lines,
8,888,896 bytes) and `...-csv-readback-1m.txt` (read back as `[1000000 1]`,
first value 1, last value 1000000).
