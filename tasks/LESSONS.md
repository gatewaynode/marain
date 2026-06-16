# Lessons

Patterns from user corrections. Review at session start and after any compact.

## 2026-06-10 — Planning-doc semantics: ROADMAP = committed, BACKLOG = uncommitted

**What happened:** When creating `tasks/BACKLOG.md`, I framed it as subordinate to
ROADMAP.md ("ROADMAP remains the backlog of record") based on ROADMAP's own header
text. The owner's actual convention is the opposite split: **ROADMAP.md = work
absolutely committed to; BACKLOG.md = work not yet committed, which can be pulled in
or sit indefinitely.**

**Rule:** When a new planning/tracking doc is requested, don't infer its role from
existing docs' self-descriptions — those can be stale. State the role I'm assuming in
the doc's header AND in my summary so the owner can correct it cheaply (that worked
here). Commitment status is the owner's call, never mine: new proposals land in
BACKLOG.md by default and move to ROADMAP.md only on an explicit acceptance decision.
