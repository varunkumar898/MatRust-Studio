# R0 Retractions

Mistakes made during the R0 documentation session and how they were fixed.
All are process/documentation mistakes; no product code or test behavior changed.

1. **Tag placed on the wrong commit.** The first `git tag r0` ran before the R0
   commit existed, so `r0` pointed at `95dcbca` (the g0.1 commit). Fixed by
   force-moving `r0` to the final R0 commit and confirming it with
   `git show-ref --tags`.

2. **Roadmap was not verbatim and was reordered.** The first `docs/ROADMAP.md`
   paraphrased descriptions, dropped the G0 a-e details and golden values, and
   placed B4 before B3. The file was rewritten from AGENTS.md section 5. A
   mechanical whitespace-normalized comparison (ignoring the inserted G0.1 row)
   reports `EXACT_MATCH=True` over 5,955 characters; output saved in
   `docs/verify-logs/2026-10-27-r0-roadmap-verbatim.txt`.

3. **STATE.md did not contain the requested next milestone.** The first edit
   changed only the current milestone and left the old "Next milestone is G1"
   text. Fixed by adding `**Next Milestone:** G0.1` and removing the G1 claim.
   Note: tag `g0.1` exists, so the roadmap status column records G0.1 as DONE
   while the owner instruction keeps it as the next milestone. This conflict is
   flagged for owner confirmation.

4. **First verification log was truncated.** Capturing `scripts/verify.ps1`
   through `Start-Transcript` produced a 37-line file without the test-result
   lines. The log was regenerated with a full stdout/stderr redirect
   (48,717 bytes, 1,172 lines), and the three test totals were parsed from its
   `test result:` lines: 196 / 196 / 195.

5. **STATE.md became stale after `cargo clean`.** The file still claimed a
   `target/release/rustlab.exe` existed after R0 removed it. The section now
   records the preserved G0.1 build in `re/targets/rustlab.exe` (size 5,400,064
   bytes; SHA-256 `78a31a97...`, verified in this session) and states that the
   release binary is absent. R0 did not rebuild it.
