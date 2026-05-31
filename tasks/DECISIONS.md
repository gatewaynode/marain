# Marain — Decision Archive

Per-round design-decision rationale. Each round's framing produces a slate of sub-decisions (lettered A-1, B-3, etc.); the slate's *what* lives in `ARCHITECTURE.md` §N.3 as a summary list, while the *why* / *alternatives considered* / *trade-offs* per decision live in this directory, one file per round.

## Protocol

When closing a round:

1. **Author the per-round file** at `tasks/decisions/RNN_<slug>.md` (e.g. `R13_functio_calls.md`). One H2 per decision, three lines per H2: `**Decision:**`, `**Why:**`, optionally `**Considered:**` / `**Result:**` / pattern links.
2. **Slim `ARCHITECTURE.md` §N.3** to a summary bullet list — one line per decision, naming the decision but not the rationale. First line of §N.3 links back to this file.
3. **Add an index entry below**, dated.
4. **Mid-implementation reframes and additions** count as decisions — extract them too, with `**Caught:**` / `**Cost:**` lines so the framing-quality signal survives.

Mechanically: a future-Claude reading `ARCHITECTURE.md` learns *what* was decided in one scroll; clicking into the per-round file recovers *why* without bloating ARCH for everyone who doesn't need the depth.

## Index

| Round | Title | Closed | Archive |
| ----- | ----- | ------ | ------- |
| R14+R15 | `pro` loops + range tokens + `nihil` | 2026-05-31 | [R14_15_pro_ranges_nihil.md](decisions/R14_15_pro_ranges_nihil.md) |
| R13 | `functio` declarations + `redde` returns + function calls | 2026-05-30 | [R13_functio_calls.md](decisions/R13_functio_calls.md) |
| R11+R12 | Operator expressions + control flow | 2026-05-29 | [R11_12_operators_control_flow.md](decisions/R11_12_operators_control_flow.md) |
| R10 | Block parsing + `si` | 2026-05-29 | [R10_block_si.md](decisions/R10_block_si.md) |
| R9 | Line comments | 2026-05-25 | [R09_line_comments.md](decisions/R09_line_comments.md) |

Rounds R1–R8 (foundation: crate layout, span model, error model, lexer, parser, codegen, CLI, testing harness) are backfill candidates — extract on demand when revisiting a round's rationale.

## Related

- `tasks/TODO.md` — round tracker (the *plan*).
- `tasks/CONTINUITY.md` — cross-session notes (the *state*).
- `tasks/LESSONS.md` — pattern lessons (the *meta*).
- `tasks/notes/` — one-off conversation captures that didn't fit a closed-round file (e.g. mid-round framing notes).
