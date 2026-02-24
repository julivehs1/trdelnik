//! Parameter ranges and spaces for optimization

use rand::Rng;
use std::collections::HashMap;
use std::ops::RangeInclusive;

/// A range of values for a single parameter
#[derive(Debug, Clone)]
pub enum ParamRange {
    /// Integer range with step
    Int { min: i64, max: i64, step: i64 },
    /// Floating-point range with step
    Float { min: f64, max: f64, step: f64 },
}

impl ParamRange {
    /// Create an integer range with step=1
    pub fn int(range: RangeInclusive<i64>) -> Self {
        Self::Int {
            min: *range.start(),
            max: *range.end(),
            step: 1,
        }
    }

    /// Create an integer range with custom step
    pub fn int_step(min: i64, max: i64, step: i64) -> Self {
        Self::Int { min, max, step }
    }

    /// Create a floating-point range with step
    pub fn float(min: f64, max: f64, step: f64) -> Self {
        Self::Float { min, max, step }
    }

    /// Get all discrete values in this range
    pub fn values(&self) -> Vec<f64> {
        match self {
            Self::Int { min, max, step } => {
                let mut vals = vec![];
                let mut v = *min;
                while v <= *max {
                    vals.push(v as f64);
                    v += step;
                }
                vals
            }
            Self::Float { min, max, step } => {
                let mut vals = vec![];
                let mut v = *min;
                // Use epsilon comparison for floating point
                while v <= *max + f64::EPSILON {
                    vals.push(v);
                    v += step;
                }
                vals
            }
        }
    }

    /// Count of discrete values in this range
    pub fn count(&self) -> usize {
        self.values().len()
    }

    /// Sample a random value from this range
    pub fn sample(&self, rng: &mut impl Rng) -> f64 {
        let values = self.values();
        if values.is_empty() {
            return 0.0;
        }
        values[rng.gen_range(0..values.len())]
    }
}

/// A space of parameters to search over
#[derive(Debug, Clone, Default)]
pub struct ParamSpace {
    /// Parameters with their ranges (Vec for deterministic ordering)
    params: Vec<(String, ParamRange)>,
}

impl ParamSpace {
    /// Create an empty parameter space
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a parameter range
    pub fn add(&mut self, name: String, range: ParamRange) {
        self.params.push((name, range));
    }

    /// Get the total number of combinations
    pub fn total_combinations(&self) -> usize {
        if self.params.is_empty() {
            return 0;
        }
        self.params.iter().map(|(_, r)| r.count()).product()
    }

    /// Check if the space is empty
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Get the number of parameters
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Get parameter names
    pub fn param_names(&self) -> Vec<&str> {
        self.params.iter().map(|(n, _)| n.as_str()).collect()
    }

    /// Iterator over all parameter combinations
    pub fn iter(&self) -> ParamSpaceIter<'_> {
        ParamSpaceIter::new(self)
    }

    /// Sample n random parameter combinations
    pub fn sample(&self, n: usize, rng: &mut impl Rng) -> Vec<HashMap<String, f64>> {
        (0..n)
            .map(|_| {
                self.params
                    .iter()
                    .map(|(name, range)| (name.clone(), range.sample(rng)))
                    .collect()
            })
            .collect()
    }
}

/// Iterator over all parameter combinations in a ParamSpace
pub struct ParamSpaceIter<'a> {
    space: &'a ParamSpace,
    /// Current indices into each parameter's values
    indices: Vec<usize>,
    /// Cached values for each parameter
    values: Vec<Vec<f64>>,
    /// Whether iteration is complete
    done: bool,
}

impl<'a> ParamSpaceIter<'a> {
    fn new(space: &'a ParamSpace) -> Self {
        let values: Vec<Vec<f64>> = space.params.iter().map(|(_, r)| r.values()).collect();
        let done = values.iter().any(|v| v.is_empty());
        let indices = vec![0; space.params.len()];

        Self {
            space,
            indices,
            values,
            done,
        }
    }
}

impl<'a> Iterator for ParamSpaceIter<'a> {
    type Item = HashMap<String, f64>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.space.params.is_empty() {
            return None;
        }

        // Build current combination
        let params: HashMap<String, f64> = self
            .space
            .params
            .iter()
            .enumerate()
            .map(|(i, (name, _))| (name.clone(), self.values[i][self.indices[i]]))
            .collect();

        // Advance indices (like incrementing a multi-digit number)
        let mut carry = true;
        for i in (0..self.indices.len()).rev() {
            if carry {
                self.indices[i] += 1;
                if self.indices[i] >= self.values[i].len() {
                    self.indices[i] = 0;
                    // carry remains true
                } else {
                    carry = false;
                }
            }
        }

        // If we wrapped all the way around, we're done
        if carry {
            self.done = true;
        }

        Some(params)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let total = self.space.total_combinations();
        (total, Some(total))
    }
}

impl<'a> ExactSizeIterator for ParamSpaceIter<'a> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_range() {
        let range = ParamRange::int(1..=5);
        let values = range.values();
        assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(range.count(), 5);
    }

    #[test]
    fn test_int_range_with_step() {
        let range = ParamRange::int_step(0, 10, 2);
        let values = range.values();
        assert_eq!(values, vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!(range.count(), 6);
    }

    #[test]
    fn test_float_range() {
        let range = ParamRange::float(1.0, 2.0, 0.5);
        let values = range.values();
        assert_eq!(values.len(), 3);
        assert!((values[0] - 1.0).abs() < f64::EPSILON);
        assert!((values[1] - 1.5).abs() < f64::EPSILON);
        assert!((values[2] - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_param_space_combinations() {
        let mut space = ParamSpace::new();
        space.add("a".to_string(), ParamRange::int(1..=2));
        space.add("b".to_string(), ParamRange::int(10..=11));

        assert_eq!(space.total_combinations(), 4);

        let combos: Vec<_> = space.iter().collect();
        assert_eq!(combos.len(), 4);

        // Check all combinations exist
        assert!(combos.iter().any(|p| p["a"] == 1.0 && p["b"] == 10.0));
        assert!(combos.iter().any(|p| p["a"] == 1.0 && p["b"] == 11.0));
        assert!(combos.iter().any(|p| p["a"] == 2.0 && p["b"] == 10.0));
        assert!(combos.iter().any(|p| p["a"] == 2.0 && p["b"] == 11.0));
    }

    #[test]
    fn test_empty_space() {
        let space = ParamSpace::new();
        assert_eq!(space.total_combinations(), 0);
        assert_eq!(space.iter().count(), 0);
    }

    #[test]
    fn test_sample() {
        let mut space = ParamSpace::new();
        space.add("x".to_string(), ParamRange::int(1..=100));

        let mut rng = rand::thread_rng();
        let samples = space.sample(10, &mut rng);

        assert_eq!(samples.len(), 10);
        for s in samples {
            assert!(s.contains_key("x"));
            let val = s["x"];
            assert!(val >= 1.0 && val <= 100.0);
        }
    }
}
