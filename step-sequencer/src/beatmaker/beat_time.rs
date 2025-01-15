use serde::Serialize;

use crate::project::F;

#[derive(PartialEq, Eq, Ord, Clone, Copy, Debug, Serialize)]

pub struct BeatTime {
    frac: F,
}

/// Models the current time in beats (integral + fraction)
/// Fraction part ensures precise timing of tempo scale.
impl BeatTime {
    pub fn new(frac: F) -> Self {
        Self { frac }
    }

    pub fn zero() -> Self {
        Default::default()
    }

    pub fn integral(&self) -> usize {
        usize::try_from(self.frac.trunc()).unwrap() as usize
    }

    pub fn fraction(&self) -> F {
        self.frac.fract()
    }

    pub fn add_fraction(&self, frac: F) -> Self {
        Self {
            frac: self.frac + frac,
        }
    }

    pub fn add_integral(&self, integral: usize) -> Self {
        Self {
            frac: self.frac + F::from(integral),
        }
    }

    pub fn ceil(&self) -> Self {
        Self {
            frac: self.frac.ceil(),
        }
    }

    pub fn floor(&self) -> Self {
        Self {
            frac: self.frac.floor(),
        }
    }

    pub fn stretch(&self, tempo_scale: F) -> Self {
        Self {
            frac: self.frac * tempo_scale,
        }
    }

    pub fn compress(&self, tempo_scale: F) -> Self {
        Self {
            frac: self.frac / tempo_scale,
        }
    }
}

impl std::fmt::Display for BeatTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.frac)
    }
}

impl PartialOrd for BeatTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.frac.partial_cmp(&other.frac);
    }
}

impl Default for BeatTime {
    fn default() -> Self {
        Self { frac: F::from(0) }
    }
}

#[cfg(test)]
mod tests {
    use crate::project::F;

    use super::BeatTime;

    #[test]
    fn test_beat_time_stretch() {
        let beat_time = BeatTime::new(F::new(11u64, 3u64));
        let stretched = beat_time.stretch(F::new(4u64, 5u64));
        assert_eq!(stretched, BeatTime::new(F::new(44u64, 15u64)));
    }

    #[test]
    fn test_beat_time_compress() {
        let beat_time = BeatTime::new(F::new(11u64, 3u64));
        let compressed = beat_time.compress(F::new(4u64, 5u64));
        assert_eq!(compressed, BeatTime::new(F::new(55u64, 12u64)));
    }
}
