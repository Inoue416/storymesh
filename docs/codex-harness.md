# Codex development harness

This repository uses a small, measurable Codex harness to balance delivery speed,
artifact quality, and token usage. `AGENTS.md` is the only project guide loaded on
every task; detailed operating notes stay here so they do not consume context on
every turn.

## Start a task

Choose the cheapest reasoning mode appropriate for the risk:

```sh
scripts/codex-task fast       # docs, mechanical changes, known one-file fixes
scripts/codex-task standard   # normal feature and bug work (default)
scripts/codex-task deep       # architecture, security, subtle correctness
```

The wrapper keeps the user's selected model and only changes reasoning effort and
response verbosity. This avoids locking the repository to a model or price tier.
For a non-interactive run, add `--exec`.

Start with a compact prompt that removes ambiguity early:

```text
Goal: <one observable outcome>
Acceptance: <behaviors that prove completion>
Scope: <important inclusions or exclusions>
Verify: <commands or scenarios that must pass>
```

Codex reads `.codex/config.toml` only for a trusted project. If the settings appear
to be ignored, mark this repository trusted in Codex and start a new session.

## Feedback and quality gates

Use the narrowest gate while iterating and the full gate before handoff:

```sh
mise run quick    # rustfmt check + tests; optimized for the edit loop
mise run handoff  # select the minimum final gate and record worktree evidence
mise run verify   # harness audit + rustfmt + Clippy -D warnings + tests
mise run harness  # harness structure, context budget, modes, metrics, whitespace
```

CI runs the same full gate on pushes and pull requests. Clippy over all targets and
features already performs type checking, so the full gate does not repeat a
separate `cargo check`. `mise run check` remains available for fast type-only work.

`mise run handoff` is the Codex-facing final command. The `storymesh-handoff` skill
selects `harness` for documentation and prompt-only diffs and `verify` for code,
configuration, CI, hook, or script diffs. A successful run records the exact
worktree fingerprint under `.codex-runs/`; any later edit invalidates that evidence.

## Lifecycle hooks

Project hooks are defined inline in `.codex/config.toml` and run only after the
repository and the exact hook definitions are trusted. Use `/hooks` in Codex CLI
after cloning the repository or changing a hook.

- `SessionStart` reports at most 12 dirty paths, avoiding a broad startup scan while
  making pre-existing user changes visible.
- `Stop` checks the handoff fingerprint. If changed files are unverified, it asks for
  one continuation to run `mise run handoff`; `stop_hook_active` prevents loops.

Both hooks are deterministic local scripts. They do not read conversation
transcripts, call the network, run tests automatically, or persist prompt content.

## Measure token usage

JSON logging makes cost regressions visible without spending tokens on a separate
evaluation run:

```sh
scripts/codex-task standard --json-log .codex-runs/task.jsonl \
  "Implement the accepted task and run mise run verify"
scripts/codex-metrics .codex-runs/task.jsonl
```

The summary reports total input, cached input, uncached input, output, reasoning
output, and cache-hit percentage. Run logs are ignored by Git. Compare similar
tasks rather than using a single global token target; architecture work and
documentation edits have very different needs.

Track these signals when tuning the harness:

| Goal | Primary signal | Regression signal |
| --- | --- | --- |
| Speed | warm `mise run quick` duration | repeated broad scans or duplicate checks |
| Quality | `mise run verify` and CI pass | escaped defect or missing acceptance test |
| Cost | uncached input and output tokens | growing `AGENTS.md` or unnecessary deep mode |

`mise run harness` enforces an 8 KiB ceiling for `AGENTS.md`. Prefer links to this
document over copying long explanations into always-loaded instructions.

## Operating policy

- Use `fast` by default only when the solution and verification path are already
  clear. Use `standard` for normal implementation.
- Move to `deep` after evidence of subtle risk or repeated failed attempts, not
  simply because a task is large.
- Resume a relevant thread when its context is still useful. Start a new thread
  when the old context is unrelated or misleading.
- Keep acceptance criteria in the prompt and durable repository facts in
  `AGENTS.md`; keep task-specific investigation out of persistent instructions.

## Official references

- [Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [Codex configuration reference](https://developers.openai.com/codex/config-reference)
- [Codex non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode)
