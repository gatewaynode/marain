//! Indentation state machine.
//!
//! Tracks the indent stack and bracket depth; decides per line whether to
//! emit INDENT / DEDENT(s) / nothing. Indentation inside any open bracket
//! is suppressed (Python rule: inside `()`, `[]`, `{}` indentation is
//! layout and emits no synthetic tokens).

pub(super) struct IndentState {
    stack: Vec<u32>,
    bracket_depth: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum LineStartOutcome {
    NoChange,
    Indent,
    Dedents(u32),
    Inconsistent,
}

impl IndentState {
    pub fn new() -> Self {
        Self {
            stack: vec![0],
            bracket_depth: 0,
        }
    }

    pub fn enter_bracket(&mut self) {
        self.bracket_depth += 1;
    }

    pub fn exit_bracket(&mut self) {
        self.bracket_depth = self.bracket_depth.saturating_sub(1);
    }

    pub fn is_in_bracket(&self) -> bool {
        self.bracket_depth > 0
    }

    /// Consult the indent stack at line start. Returns the outcome describing
    /// what (if any) INDENT/DEDENT tokens to emit.
    pub fn line_start(&mut self, indent: u32) -> LineStartOutcome {
        if self.is_in_bracket() {
            return LineStartOutcome::NoChange;
        }
        let current = *self
            .stack
            .last()
            .expect("indent stack invariant: non-empty");
        if indent > current {
            self.stack.push(indent);
            LineStartOutcome::Indent
        } else if indent < current {
            let mut count: u32 = 0;
            while *self.stack.last().expect("non-empty") > indent {
                self.stack.pop();
                count += 1;
            }
            if *self.stack.last().expect("non-empty") != indent {
                return LineStartOutcome::Inconsistent;
            }
            LineStartOutcome::Dedents(count)
        } else {
            LineStartOutcome::NoChange
        }
    }

    /// Emit DEDENTs to drain the stack to baseline. Called at EOF; returns
    /// the count of DEDENTs to emit.
    pub fn finalize(&mut self) -> u32 {
        let mut count = 0;
        while self.stack.len() > 1 {
            self.stack.pop();
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero_no_change() {
        let mut s = IndentState::new();
        assert_eq!(s.line_start(0), LineStartOutcome::NoChange);
    }

    #[test]
    fn indent_pushes_stack() {
        let mut s = IndentState::new();
        assert_eq!(s.line_start(4), LineStartOutcome::Indent);
        assert_eq!(s.line_start(4), LineStartOutcome::NoChange);
    }

    #[test]
    fn dedent_pops_one_level() {
        let mut s = IndentState::new();
        s.line_start(4);
        assert_eq!(s.line_start(0), LineStartOutcome::Dedents(1));
    }

    #[test]
    fn dedent_pops_multiple_levels() {
        let mut s = IndentState::new();
        s.line_start(4);
        s.line_start(8);
        s.line_start(12);
        assert_eq!(s.line_start(0), LineStartOutcome::Dedents(3));
    }

    #[test]
    fn dedent_to_intermediate_level() {
        let mut s = IndentState::new();
        s.line_start(4);
        s.line_start(8);
        assert_eq!(s.line_start(4), LineStartOutcome::Dedents(1));
    }

    #[test]
    fn dedent_to_unstacked_level_is_inconsistent() {
        let mut s = IndentState::new();
        s.line_start(4);
        s.line_start(8);
        // stack is [0, 4, 8]; dedent to 2 — no match.
        assert_eq!(s.line_start(2), LineStartOutcome::Inconsistent);
    }

    #[test]
    fn bracket_suppresses_indent() {
        let mut s = IndentState::new();
        s.enter_bracket();
        assert_eq!(s.line_start(4), LineStartOutcome::NoChange);
        assert_eq!(s.line_start(0), LineStartOutcome::NoChange);
        s.exit_bracket();
        assert_eq!(s.line_start(4), LineStartOutcome::Indent);
    }

    #[test]
    fn nested_brackets_require_full_close() {
        let mut s = IndentState::new();
        s.enter_bracket();
        s.enter_bracket();
        assert!(s.is_in_bracket());
        s.exit_bracket();
        assert!(s.is_in_bracket());
        s.exit_bracket();
        assert!(!s.is_in_bracket());
    }

    #[test]
    fn exit_bracket_saturates_at_zero() {
        let mut s = IndentState::new();
        s.exit_bracket();
        s.exit_bracket();
        assert!(!s.is_in_bracket());
    }

    #[test]
    fn finalize_drains_stack() {
        let mut s = IndentState::new();
        s.line_start(4);
        s.line_start(8);
        assert_eq!(s.finalize(), 2);
    }

    #[test]
    fn finalize_at_baseline_is_zero() {
        let mut s = IndentState::new();
        assert_eq!(s.finalize(), 0);
    }
}
