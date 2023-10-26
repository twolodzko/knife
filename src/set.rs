/// Set contains some values
#[derive(Debug, PartialEq, Clone)]
pub enum Set<T>
where
    T: PartialEq + Ord,
{
    /// Single value
    Value(T),
    /// A range of values
    Range(T, T),
    /// Collection of values and/or ranges
    Mixed(Vec<Set<T>>),
}

impl<T> Set<T>
where
    T: PartialEq + Ord,
{
    /// Check if `value` is contained in the `Set`
    #[inline]
    pub fn contains(&self, value: &T) -> bool {
        use Set::{Mixed, Range, Value};
        match self {
            Value(this) => this == value,
            Range(min, max) => value >= min && value <= max,
            Mixed(values) => values.iter().any(|s| s.contains(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Set::{self, Mixed, Range, Value};
    use test_case::test_case;

    #[test_case(Value(3), 1, false; "value does not contain")]
    #[test_case(Value(3), 3, true; "value does contain")]
    #[test_case(Range(1, 3), 7, false; "range does not contain")]
    #[test_case(Range(1, 3), 2, true; "range does contain in middle")]
    #[test_case(Range(1, 3), 1, true; "range does contain at min")]
    #[test_case(Range(1, 3), 3, true; "range does contain at max")]
    #[test_case(Mixed(vec![Value(1), Range(3, 5)]), 4, true; "mixed does contain")]
    #[test_case(Mixed(vec![Value(1), Range(3, 5)]), 2, false; "mixed does not contain")]
    fn contains(fields: Set<u32>, example: u32, expected: bool) {
        assert!(fields.contains(&example) == expected);
    }
}
