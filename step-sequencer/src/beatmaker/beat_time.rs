use serde::Serialize;

use crate::project::F;

#[derive(PartialEq, Eq, Ord, Clone, Copy, Debug, Serialize)]

pub struct BeatTime {
    integral: usize,
    frac: F,
}

/// Models the current time in beats (integral + fraction)
/// Fraction part ensures precise timing of tempo scale.
impl BeatTime {
    pub fn new(integral: usize, frac: F) -> Self {
        let mut beat_time = Self { integral, frac };
        beat_time.normalize();
        beat_time
    }

    pub fn zero() -> Self {
        Default::default()
    }

    pub fn integral(&self) -> usize {
        self.integral
    }

    pub fn fraction(&self) -> F {
        self.frac
    }

    fn unnormalize(&mut self) {
        self.frac += F::from(self.integral);
        self.integral = 0;
    }

    fn normalize(&mut self) {
        if self.frac >= F::from(1) {
            self.integral += usize::try_from(self.frac.trunc()).unwrap();
            self.frac -= self.frac.trunc();
        }
    }

    pub fn add_fraction(&self, frac: F) -> Self {
        let mut result = self.clone();
        result.frac += frac;
        result.normalize();
        return result;
    }

    pub fn add_integral(&self, integral: usize) -> Self {
        let mut result = self.clone();
        result.integral += integral;
        result.normalize();
        return result;
    }

    pub fn ceil(&self) -> Self {
        let mut result = self.clone();
        result.frac = result.frac.ceil();
        result.normalize();
        return result;
    }

    pub fn floor(&self) -> Self {
        let mut result = self.clone();
        result.frac = result.frac.floor();
        result.normalize();
        return result;
    }

    pub fn stretch(&self, tempo_scale: F) -> Self {
        let mut result = self.clone();
        result.unnormalize();
        result.frac *= tempo_scale;
        result.normalize();
        return result;
    }

    pub fn compress(&self, tempo_scale: F) -> Self {
        let mut result = self.clone();
        result.unnormalize();
        result.frac /= tempo_scale;
        result.normalize();
        return result;
    }
}

impl std::fmt::Display for BeatTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}+{}", self.integral, self.frac)
    }
}

impl PartialOrd for BeatTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.integral != other.integral {
            return self.integral.partial_cmp(&other.integral);
        }
        return self.frac.partial_cmp(&other.frac);
    }
}

impl Default for BeatTime {
    fn default() -> Self {
        Self {
            integral: 0,
            frac: F::from(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::project::F;

    use super::BeatTime;

    #[test]
    fn test_beat_time_stretch() {
        let beat_time = BeatTime::new(3, F::new(2u64, 3u64));
        let stretched = beat_time.stretch(F::new(4u64, 5u64));
        assert_eq!(stretched, BeatTime::new(2, F::new(14u64, 15u64)));
    }

    #[test]
    fn test_beat_time_compress() {
        let beat_time = BeatTime::new(3, F::new(2u64, 3u64));
        let compressed = beat_time.compress(F::new(4u64, 5u64));
        assert_eq!(compressed, BeatTime::new(4, F::new(7u64, 12u64)));
    }
}
