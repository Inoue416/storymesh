---
name: storymesh-handoff
description: Select and run the smallest valid Storymesh verification gate from the current Git diff. Use when Codex is preparing to hand off repository changes, deciding whether docs-only work needs the harness or code/config work needs the full gate, checking whether the current worktree was verified, or reporting final validation. Do not use for implementation work before the change is ready to verify.
---

# Verify a Storymesh change

1. Finish the smallest coherent change and its behavior-focused tests.
2. Run a focused test while iterating when the change affects Rust behavior. Exercise
   CLI-visible behavior with `mise exec -- cargo run -- <args>`.
3. Preview the selected handoff gate when useful:

   ```sh
   .agents/skills/storymesh-handoff/scripts/check-change --print
   ```

4. Run the adaptive handoff gate exactly once after the final edit:

   ```sh
   mise run handoff
   ```

   This runs `mise run harness` for documentation and prompt-only diffs, and
   `mise run verify` for Rust, configuration, CI, hook, or script diffs. It records
   evidence only when the gate succeeds and the worktree fingerprint stays unchanged.
5. If the gate fails, fix the cause and rerun it. Never describe an unexecuted gate as
   passing.
6. Review `git diff --check`, `git diff --stat`, and the final diff. Report the selected
   gate, its result, and any remaining risk.
