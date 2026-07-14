# Storymesh agent guide

## Mission

Build `storymesh`, a Rust CLI that detects missing Storybook stories and reports
coverage. Keep changes small, observable, and reversible.

## Source of truth

- `README.md`: supported behavior and user-facing commands.
- `Cargo.toml`: Rust/toolchain and dependency constraints.
- `mise.toml`: canonical development and verification commands.
- Existing code and tests: current behavior. Do not invent requirements when the
  repository can answer the question.

## Work loop

1. Inspect only the files relevant to the request; use `rg`/`rg --files` first.
2. For non-trivial work, state acceptance criteria and a short plan before editing.
3. Implement the smallest coherent change and add behavior-focused tests.
4. Run the narrowest useful check while iterating, then the required handoff gate.
5. Review `git diff --check`, `git diff --stat`, and the final diff before reporting.

Do not discard unrelated user changes. Do not add a dependency when the standard
library or an existing crate is sufficient. Do not delegate work unless the task
explicitly asks for parallel agents.

## Commands

Use `mise` because a bare `cargo` may not be on `PATH`.

```sh
mise run quick    # fast local feedback: format + tests
mise run handoff  # adaptive final gate with worktree evidence
mise run verify   # full handoff gate: harness + format + Clippy + tests
mise run harness  # validate the Codex harness itself
mise run format
mise run lint
mise run test
mise run check
```

Run a focused `mise exec -- cargo test <name>` during iteration when possible.

## Verification matrix

- Documentation or prompt-only change: `mise run handoff` selects the harness.
- Rust behavior or test change: focused test, then `mise run handoff` selects verify.
- `Cargo.toml`, `Cargo.lock`, `mise.toml`, CI, or harness script change:
  `mise run handoff` selects verify.
- CLI-visible behavior: also exercise the affected command through
  `mise exec -- cargo run -- <args>`.

Never claim a check passed unless it ran successfully in the current worktree.
If a check cannot run, report the exact command and reason.

## Rust quality bar

- Keep domain logic separate from CLI parsing and process exit behavior.
- Return actionable errors with path or operation context.
- Avoid `unwrap`, `expect`, and panics in production paths.
- Test observable behavior, edge cases, and failure modes; avoid tests coupled to
  implementation details.
- Keep public APIs documented and names precise. Format with rustfmt and treat all
  Clippy warnings as errors.

## Context and token discipline

- Prefer local evidence over broad searches; do not browse for stable repo facts.
- Batch independent reads and checks. Do not repeatedly read unchanged files.
- Keep plans and progress updates concise. Escalate reasoning depth only for
  architecture, security, subtle correctness, or repeated failed attempts.
- Ask one focused question only when repository evidence cannot resolve a material
  ambiguity; otherwise state a safe assumption and proceed.

## Handoff

Report: outcome, key files changed, checks run, and any remaining risk or follow-up.
Do not include a play-by-play of routine tool calls.
