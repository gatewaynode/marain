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

## 2026-06-16 — Owner handles all Git and CI/CD

**What happened:** After R16's commit `2eb497a` landed without 5 new (untracked)
files, I ran `git add` to complete it. The owner rejected it: "I prefer to always
handle Git and CI/CD work."

**Rule:** Never run `git add`/`commit`/`push`/branch ops or CI commands — not even
when work is obviously committable or the user says "commit the work" (that means
*they* will). When a commit is incomplete or changes are ready, **name the exact git
action needed (e.g. the untracked files to add) and stop.** Faithful reporting still
holds: flag a partial/broken commit loudly rather than papering over it.
