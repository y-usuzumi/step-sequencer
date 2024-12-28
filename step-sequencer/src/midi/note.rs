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
