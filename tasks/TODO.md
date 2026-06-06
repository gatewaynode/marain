# Task tracking

---
### Format

Task Number:
Description:
Implementation Notes:
---

Task Number: 1
Description: No string concatenation / interpolation in the language. Found during manual testing of `loop_test.lat` (2026-05-31) — wanted to combine a literal with the loop variable (`"Eviligo" <join> ^_i`); tried `sic`, which is not a keyword.
Implementation Notes:
- Not a bug — a design boundary. There is no concat operator at all; `plus` is arithmetic `+` only, and `&str + &str` isn't valid Rust regardless.
- PRD's intended mechanism is f-strings (`f"Eviligo {^_i}"`, §151/§157 "multi-value cases are handled by f-strings") + the `forma`/`format!` macro (§164). f-strings are deferred (not in v0.1 §363, not in v0.2 §4.11 scope → v0.3+). `forma` is implemented but only in the punctuation-free single-arg form, so it can't carry interpolation args.
- `sic` ("thus") was the user's guess; semantically weak for "join", and `et` (the natural Latin "and") is already taken as the boolean `&&` operator (R11+R12).
- Design fork when picked up: (a) implement f-strings per PRD vision [larger: string-interior parsing], or (b) add a concat operator/keyword [smaller, but diverges from PRD — amend vision doc first per CLAUDE.md].
- Verified the loop itself runs end-to-end through cargo: `pro ^_i in 0..=^series :` with `series=10` prints 11 lines. First confirmed runnable v0.2 `pro`/range program.

Task Number: 2
Description: No increment/decrement support, and — underneath it — no reassignment at all. Found during manual testing of `loop_test.lat` (2026-05-31): tried `@series--.` (decrement). Fails at the lexer (`unexpected character '-'`) because `-` is not in Marain's lex alphabet (subtraction is the keyword `minus`).
Implementation Notes:
- `++`/`--` are not a gap: Rust has no increment/decrement operators either; the idiom is `x += 1`. Marain inherits that, so `++`/`--` should never exist.
- THE REAL GAP: reassignment is specced-but-unimplemented. `fit` ("becomes", `=` reassign) is in PRD §94 / §115 (`^x fit 5`), the lexicon (line 46), and is a lexer keyword (`Keyword::Fit`) — but `parse_stmt` has NO `fit` dispatch arm and there is no `Stmt::Assign` variant. So `fit` is reserved + lexed but never parsed/emitted. A `@x` mutable binding can be declared but never mutated.
- Consequence: the Marain increment idiom `@series fit @series plus 1.` can't be written today. Mutation appears to have been skipped across R9–R15 (no round covered it).
- No compound-assignment (`+=` analogue) is specced in the §4.4 operator table, so even after `fit` lands, increment stays explicit: `@x fit @x plus 1.`.
- Likely next implementation round: `fit` reassignment statement (new `Stmt::Assign { target: SigiledIdent, value: Expr }`, dispatch on `Keyword::Fit`, emit `target = value;`). Small; in-spec (no PRD amendment needed). Would also let `loop_test.lat`-style mutation programs run end-to-end.

Task Number: 3
Description: Emitted Rust throws `unused_parens` warnings on first compile. Found in `loop_test.lat` (2026-05-31): `si ^_i minor quam 5 :` emits `if (_i < 5i64) {` → "unnecessary parentheses around `if` condition". The program compiles and runs correctly — this is a cleanliness defect, not a correctness bug.
Implementation Notes:
- Root cause: the R11+R12 decision (ARCHITECTURE §14) to paren-wrap EVERY `BinOp`/`UnaryOp` in `emit_expr` ("paren-everywhere is bulletproof against Rust precedence drift"). The redundant parens are the explicitly-accepted tradeoff of that choice.
- Went unnoticed because emit goldens are string-compare only (never invoke cargo) and the only compiling e2e was hello-world. First surfaced by manual testing of BinOp-in-condition code.
- Blast radius (confirmed by probe): fires wherever a top-level BinOp/UnaryOp sits in a slot Rust's `unused_parens` checks — `if`/`while` conditions, `let` assigned value (`let x = (1i64 + 2i64);`), and (by the same mechanism) `redde` value. Nested operands inside larger expressions are NOT in a checked slot, so they don't warn — only the outermost parens in these positions do.
- Fix options when picked up:
  (a) Precedence-aware emit [elegant; preferred]: wrap a child expr only when its operator precedence/associativity actually requires it vs. the parent. The AST already encodes precedence via tree shape, so it's computable — standard pretty-printer technique. Eliminates all redundant parens, keeps necessary ones. Larger change to emit.rs; reverses the §14 paren-everywhere decision (would need an ARCHITECTURE §14 note + a new decision entry).
  (b) Outermost-strip [surgical]: keep paren-everywhere for nested ops but don't wrap the top-level expr when emitting the checked slots (if/while/for cond, let/return value). Smaller, lower-risk; clears all current warnings since the lint only checks those outermost slots.
- COST either way: every emit fixture / golden currently bakes in paren-everywhere output, so the `*.expected.rs` goldens (and emit unit tests) regenerate. Regen via `MARAIN_UPDATE_GOLDENS=1`, then eyeball-verify.
- Ties to the missing-coverage theme: the goldens never compile their output. A v0.2 done-line e2e (flagged earlier) that runs emitted control-flow code through cargo with `-D warnings` would have caught this automatically and would prevent regressions.

