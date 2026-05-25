⏺ Eleven open questions across three groups. Each has my recommended pick + one-line rationale; redirect on any you want to push
  back on, and we'll commit the resolved set to PRD/lexicon afterward.

  Function syntax

  F-1. Parameter-list shape.
  - (a) Rust-style (^nomen: Sermo, ^aetas: Numerus). (b) cum (...) wrapper. (c) keyword-only, no parens.
  - Recommend (a). Parens are structural delimiters, not operators; PRD §4.4's "operators are Latin function words" doesn't claim
  all syntax must be. Already established for collection literals (agmen [...]). Stage 2 case grammar fits inside the parens later.
  
Answer (F-1): I think this makes sense to preserve the Rust style.  While asthetically a little rough, this will help make function  openings stand out, making the code easier to scan.

  F-2. Return-type indicator.
  - (a) Keep ->. (b) dat ("gives"): functio foo() dat Numerus :. (c) Reuse redde: functio foo() redde Numerus :. (d) ad ("to").
  - Recommend (b) dat. New single-word keyword, indicative present 3rd-sg. active — parallel form with est/fit/aequat. Reuse-redde
  (c) re-creates the exact overload pattern we just rejected for est===. Keeping -> (a) breaks §4.4 spirit.

Answer (F-2): I like option (b), I think this works really well with keeping the parenthesis.

  F-3. Defer bundle. Confirm out of v0.2 scope: closures (|x| body), generics (<T>), visibility (pub → publicus). Each is a
  discrete decision space (capture rules; bounds/lifetimes/const-generics; module-boundary visibility) that compounds v0.2 surface.
   v0.3+ adds them.
  - Recommend confirm. v0.2 already has functions + loops + blocks + return types + just-enough operators for loop conditions — a
  lot.
  
Answer (F-3): Confirmed.  Let's keep the scope narrow.

  Loop vocabulary

  L-1. loop (Rust infinite loop).
  - (a) semper ("always"): semper :. (b) iterum ("again"). (c) Skip — write dum verum :. (d) cyclus (noun).
  - Recommend (a) semper. Natural Latin for "forever"; concise. Adverb, not a verb — exception to §4.2's verb-mood pattern, justify
   as "no Latin verb for 'loop forever' fits the mood system." (c) works but reads as a hack.

Answer (L-1): I think "semper" works, so option (a).

  L-2. break.
  - (a) interrumpe (imperative 2nd-sg. of interrumpere). (b) desine ("cease!"). (c) frange ("break!").
  - Recommend (a) interrumpe, no !. PRD §4.2 already floats interrumpe! but as a macro — Rust's break is a keyword, so the ! was
  vestigial. Imperative mood matches §4.2's "action" convention. Side effect: PRD §4.2 macro-example row needs amendment to remove
  the !.

Answer (L-2): Again option (a) works, "interrumpe" is almost English as well so easy for English speakers to pick up on.

  L-3. continue.
  - (a) perge ("go on!", imperative 2nd-sg. of pergere). (b) continua. (c) iterum (conflicts with L-1 alt).
  - Recommend (a) perge. Imperative per §4.2; (b) is too close to English continue to be linguistically interesting.

Amswer (L-3): I think "continua" works best here, so option (b) similar reasoning as L-2.

  L-4. for binding word.
  - (a) in (pro ^x in ^xs :). (b) ex ("from"). (c) per (conflicts with *).
  - Recommend (a) in. Latin in and English/Rust in converge — zero mental cost. Add as a new keyword. Stage 2 may pair in with
  ablative-case iterable.

Answer (L-4): Agreed, keep "in".

  L-5. Range syntax (0..10).
  - (a) Keep .. / ..= as structural punctuation. (b) ab 0 ad 10. (c) 0 ad 10. (d) 0 usque ad 10.
  - Recommend (a) keep .. / ..=. Multiple range variants (a..b, a..=b, ..b, a.., ..) get unwieldy in word form. Same principle as
  PRD §4.3 keeping numeric literals Arabic — keep numeric-domain syntax mechanical.

Answer (L-5): I suppose keeping ".." is reasonable, though that is stretching it.  Document the alternatives in the comments please.

  Block syntax

  B-1. Control-structure-head terminator (closes PRD §4.8's leftover TBD).
  - (a) Python-style :: si ^x aequat 5 :. (b) Implicit (head ends when block begins). (c) Period ..
  - Recommend (a) :. Visually distinguishes head from body; lexer already tracks indentation. No clash with :: (path separator).
  Single : convention covers si / dum / pro / semper / functio heads.

Answer (B-1): I do think Python style serves well here, go with (a).

  B-2. Function-body block intro.
  - (a) Same : shape — functio saluta() dat Sermo :\n    redde "salve".\n. (b) Distinct keyword like incipit ("begins").
  - Recommend (a). Single block-intro convention; function bodies and control-flow bodies are conceptually the same (indented
  statement sequence). Reduces parser branching and reader cognitive load.

Answer (B-2): Option (1) reads well, stick with it.

  B-3. Empty block.
  - (a) Keyword statement: nihil ("nothing") — functio stub() :\n    nihil.\n. (b) Allow zero-statement body (lexer immediately
  dedents). (c) Disallow.
  - Recommend (a) nihil. Python pass idiom in Latin. Visually clear that the block is intentional. (b) makes the lexer state
  machine fussier; (c) forces a no-op anyway.

Answer (b-3): I've always found the Python "pass" to be very useful, lets go with (a) "nihil" just sounds right.

  ---
  Summary of new keywords this would add (8 total): dat, semper, interrumpe, perge, in, nihil, and the new structural : token.
  Existing keywords already in the table that get first use: functio, redde, dum, pro. Plus the .. / ..= range punctuation as new
  lexer tokens.