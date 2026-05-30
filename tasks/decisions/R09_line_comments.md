# R9 — Line comments

_Closed 2026-05-25. Decision rationale archive. Summary list lives in [`ARCHITECTURE.md` §12.3](../../ARCHITECTURE.md#123-decisions); this file holds the *why* per decision._

## Comment-only lines transparent to indent state

**Decision:** at line start, after leading whitespace is consumed, the dispatcher peeks two bytes ahead for `//`. If found, the comment is consumed and the iteration continues *without invoking the indent state* — identical to the blank-line path.

**Why:** PRD §4.12 — a `//` line inside an indented block must neither open a new block nor close the current one. Folding comment-only lines into the existing blank-line transparency mechanism keeps the indent state machine unchanged.

## Mid-line `//` is the simple case

**Decision:** dispatcher hits `/` mid-line, peeks `/`, scans-to-EOL, `continue`s.

**Why:** indent state was already decided at line start before any tokens were emitted. The comment is transparent to it — no extra plumbing.

## `/*` rejected with a targeted diagnostic

**Decision:** dedicated variant `LexError::BlockCommentsDeferred { span }`. Span covers exactly the two-byte `/*`. Message: `block comments are reserved syntax; use // for a line comment (PRD §4.12)`.

**Why:** the generic `UnexpectedChar` path would leave the user guessing. A targeted diagnostic catches likely intent and points at the workaround. The dedicated variant also reserves `/*` against being claimed by any future proposal.

**Considered:** generic `UnexpectedChar`. Rejected: poor UX, and `/*` is *known* to be coming in v0.3+.

## Bare `/` stays `UnexpectedChar`

**Decision:** bare `/` (not followed by `/` or `*`) continues to fire `UnexpectedChar`.

**Why:** division is `divisus per` per PRD §4.4; `/` has no standalone use today. Forward-compatible: the v0.3 block-comment work only adds an arm to the `/` dispatcher; the `BlockCommentsDeferred` variant retires then.

## `\n` left for the existing newline handler

**Decision:** the comment scanner stops at `\n` exclusive. The lexer's normal end-of-line processing then advances past `\n` and sets `at_line_start = true`.

**Why:** comment-induced off-by-one bugs in error reporting are categorically avoided. The newline handler owns one responsibility; the comment scanner doesn't reach into it.

## `Cursor::peek_at(offset)` joins the cursor API

**Decision:** add `peek_at(offset)` to the cursor primitives.

**Why:** two-character openers (existing `::`, future `..` per R14, now `//` and `/*`) all need to peek ahead. The save-pos / advance / restore dance is fragile; `peek_at` is three lines and pays for itself across multiple call sites.
