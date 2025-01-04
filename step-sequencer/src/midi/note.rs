use std::{path::Display, str::FromStr, sync::LazyLock};

use regex::Regex;
use thiserror::Error;

use self::PitchClass::*;

use super::Key;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PitchClass {
    C,
    Cs,
    Df,
    D,
    Ds,
    Ef,
    E,
    F,
    Fs,
    Gf,
    G,
    Gs,
    Af,
    A,
    As,
    Bf,
    B,
}

impl std::fmt::Display for PitchClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                C => "C",
                Cs => "C#",
                Df => "Db",
                D => "D",
                Ds => "D#",
                Ef => "Eb",
                E => "E",
                F => "F",
                Fs => "F#",
                Gf => "Gb",
                G => "G",
                Gs => "G#",
                Af => "Ab",
                A => "A",
                As => "A#",
                Bf => "Bb",
                B => "B",
            }
        )
    }
}

impl Into<i8> for PitchClass {
    fn into(self) -> i8 {
        match self {
            C => 0,
            Cs | Df => 1,
            D => 2,
            Ds | Ef => 3,
            E => 4,
            F => 5,
            Fs | Gf => 6,
            G => 7,
            Gs | Af => 8,
            A => 9,
            As | Bf => 10,
            B => 11,
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Note {
    pitch_class: PitchClass,
    octave: i8,
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.pitch_class, self.octave)
    }
}

#[derive(Clone, Debug, Error)]
pub struct ParseNoteError(String);

impl std::fmt::Display for ParseNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid note: {0}", self.0)
    }
}

static NOTE_REGEX: LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new("(?<note>[A-Ga-g][#b]?)(?<octave>-?[0-9]+)").unwrap());

fn get_note_regex() -> &'static Regex {
    return &*NOTE_REGEX;
}

impl FromStr for Note {
    type Err = ParseNoteError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NOTE_REGEX
            .captures(s)
            .map(|cap| {
                let note = cap.name("note").unwrap().as_str();
                let octave = cap.name("octave").unwrap().as_str().parse::<i8>().ok()?;
                match note {
                    "c" | "C" => Some(Note::C(octave)),
                    "c#" | "C#" => Some(Note::Cs(octave)),
                    "db" | "Db" => Some(Note::Df(octave)),
                    "d" | "D" => Some(Note::D(octave)),
                    "d#" | "D#" => Some(Note::Ds(octave)),
                    "eb" | "Eb" => Some(Note::Ef(octave)),
                    "e" | "E" => Some(Note::E(octave)),
                    "f" | "F" => Some(Note::F(octave)),
                    "f#" | "F#" => Some(Note::Fs(octave)),
                    "gb" | "Gb" => Some(Note::Gf(octave)),
                    "g" | "G" => Some(Note::G(octave)),
                    "g#" | "G#" => Some(Note::Gs(octave)),
                    "ab" | "Ab" => Some(Note::Af(octave)),
                    "a" | "A" => Some(Note::A(octave)),
                    "a#" | "A#" => Some(Note::As(octave)),
                    "bb" | "Bb" => Some(Note::Bf(octave)),
                    "b" | "B" => Some(Note::B(octave)),
                    _ => None,
                }
            })
            .flatten()
            .ok_or(ParseNoteError(format!(
                "Invalid note representation: `{0}`",
                s
            )))
    }
}

macro_rules! note_func {
    ($x:ident) => {
        pub const fn $x(octave: i8) -> Self {
            Self {
                pitch_class: $x,
                octave,
            }
        }
    };
}

#[allow(non_snake_case)]
impl Note {
    note_func! {C}
    note_func! {Cs}
    note_func! {Df}
    note_func! {D}
    note_func! {Ds}
    note_func! {Ef}
    note_func! {E}
    note_func! {F}
    note_func! {Fs}
    note_func! {Gf}
    note_func! {G}
    note_func! {Gs}
    note_func! {Af}
    note_func! {A}
    note_func! {As}
    note_func! {Bf}
    note_func! {B}
}

impl Into<Key> for Note {
    fn into(self) -> Key {
        (24 + 12 * self.octave + Into::<i8>::into(self.pitch_class)) as Key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_note() {
        assert_eq!("F#3".parse::<Note>().unwrap(), Note::Fs(3));
        assert_eq!("Gb-2".parse::<Note>().unwrap(), Note::Gf(-2));
        assert!("B1288".parse::<Note>().is_err());
    }
}
