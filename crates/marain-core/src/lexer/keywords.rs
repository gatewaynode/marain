//! Keyword table: all reserved words in Marain Stage 1.
//!
//! Bare identifier scanning consults this table; sigiled identifiers never
//! do (a sigil unambiguously marks a variable reference per PRD §4.5).
//! The table grows program-by-program per PRD §4.3.

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Keyword {
    // Bindings
    Sit, // let
    Fit, // reassign ("becomes")
    Est, // init / equality ("is")

    // No-punct macros (PRD §4.7)
    Dic,    // println!
    Queror, // eprintln!
    Agmen,  // vec!
    Forma,  // format!

    // Borrows / self
    Tenet, // & / &mut
    Ego,   // self

    // Booleans
    Verum,  // true
    Falsum, // false

    // Control / declarations
    Redde,   // return
    Functio, // fn
    Si,      // if
    Dum,     // while
    Pro,     // for

    // Logical operators
    Et,  // &&
    Vel, // || (also part of `minor vel par`, `maior vel par`)
    Non, // ! (negation prefix; also part of `non est`)

    // Arithmetic and comparison
    Plus,
    Minus,
    Per,     // *
    Modulo,  // %
    Maior,   // greater (part of `maior quam` ≥)
    Minor,   // less    (part of `minor quam` ≤)
    Quam,    // than    (part of `maior/minor quam`)
    Par,     // equal   (part of `minor vel par` ≤)
    Divisus, // divided (part of `divisus per` /)

    // Detonation (sanctioned ALL-CAPS exception, PRD §4.2)
    Detonatio,
}

impl Keyword {
    pub fn lookup(s: &str) -> Option<Self> {
        Some(match s {
            "sit" => Self::Sit,
            "fit" => Self::Fit,
            "est" => Self::Est,
            "dic" => Self::Dic,
            "queror" => Self::Queror,
            "agmen" => Self::Agmen,
            "forma" => Self::Forma,
            "tenet" => Self::Tenet,
            "ego" => Self::Ego,
            "verum" => Self::Verum,
            "falsum" => Self::Falsum,
            "redde" => Self::Redde,
            "functio" => Self::Functio,
            "si" => Self::Si,
            "dum" => Self::Dum,
            "pro" => Self::Pro,
            "et" => Self::Et,
            "vel" => Self::Vel,
            "non" => Self::Non,
            "plus" => Self::Plus,
            "minus" => Self::Minus,
            "per" => Self::Per,
            "modulo" => Self::Modulo,
            "maior" => Self::Maior,
            "minor" => Self::Minor,
            "quam" => Self::Quam,
            "par" => Self::Par,
            "divisus" => Self::Divisus,
            "DETONATIO" => Self::Detonatio,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sit => "sit",
            Self::Fit => "fit",
            Self::Est => "est",
            Self::Dic => "dic",
            Self::Queror => "queror",
            Self::Agmen => "agmen",
            Self::Forma => "forma",
            Self::Tenet => "tenet",
            Self::Ego => "ego",
            Self::Verum => "verum",
            Self::Falsum => "falsum",
            Self::Redde => "redde",
            Self::Functio => "functio",
            Self::Si => "si",
            Self::Dum => "dum",
            Self::Pro => "pro",
            Self::Et => "et",
            Self::Vel => "vel",
            Self::Non => "non",
            Self::Plus => "plus",
            Self::Minus => "minus",
            Self::Per => "per",
            Self::Modulo => "modulo",
            Self::Maior => "maior",
            Self::Minor => "minor",
            Self::Quam => "quam",
            Self::Par => "par",
            Self::Divisus => "divisus",
            Self::Detonatio => "DETONATIO",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_recognizes_dic() {
        assert_eq!(Keyword::lookup("dic"), Some(Keyword::Dic));
    }

    #[test]
    fn lookup_rejects_unknown() {
        assert_eq!(Keyword::lookup("unknown"), None);
        assert_eq!(Keyword::lookup(""), None);
    }

    #[test]
    fn lookup_is_case_sensitive() {
        assert_eq!(Keyword::lookup("DIC"), None);
        assert_eq!(Keyword::lookup("Dic"), None);
        assert_eq!(Keyword::lookup("DETONATIO"), Some(Keyword::Detonatio));
        assert_eq!(Keyword::lookup("detonatio"), None);
    }

    #[test]
    fn round_trip_all_keywords() {
        let all = [
            Keyword::Sit,
            Keyword::Fit,
            Keyword::Est,
            Keyword::Dic,
            Keyword::Queror,
            Keyword::Agmen,
            Keyword::Forma,
            Keyword::Tenet,
            Keyword::Ego,
            Keyword::Verum,
            Keyword::Falsum,
            Keyword::Redde,
            Keyword::Functio,
            Keyword::Si,
            Keyword::Dum,
            Keyword::Pro,
            Keyword::Et,
            Keyword::Vel,
            Keyword::Non,
            Keyword::Plus,
            Keyword::Minus,
            Keyword::Per,
            Keyword::Modulo,
            Keyword::Maior,
            Keyword::Minor,
            Keyword::Quam,
            Keyword::Par,
            Keyword::Divisus,
            Keyword::Detonatio,
        ];
        for kw in all {
            assert_eq!(
                Keyword::lookup(kw.as_str()),
                Some(kw),
                "round trip failed for {kw:?}",
            );
        }
    }
}
