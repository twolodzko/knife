/// Set contains some values
#[derive(Debug, PartialEq, Clone)]
pub enum Set<T> {
    /// Single value
    Value(T),
    /// A range of values (open or closed)
    Range(Option<T>, Option<T>),
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
            Range(None, None) => true,
            Range(None, Some(max)) => value <= max,
            Range(Some(min), None) => value >= min,
            Range(Some(min), Some(max)) => value >= min && value <= max,
            Mixed(values) => values.iter().any(|s| s.contains(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Set::{self, Mixed, Range, Value};
    use test_case::test_case;

    #[test_case(Value(3), 1, false; "field does not contain")]
    #[test_case(Value(3), 3, true; "field does contain")]
    #[test_case(Range(Some(1), Some(3)), 7, false; "closed range does not contain")]
    #[test_case(Range(Some(1), Some(3)), 2, true; "closed range does contain middle")]
    #[test_case(Range(Some(1), Some(3)), 1, true; "closed range does contain min")]
    #[test_case(Range(Some(1), Some(3)), 3, true; "closed range does contain max")]
    #[test_case(Range(None, None), 3, true; "open range does contain")]
    #[test_case(Range(None, Some(10)), 3, true; "left-open range does contain")]
    #[test_case(Range(None, Some(10)), 11, false; "left-open range does not contain")]
    #[test_case(Range(Some(5), None), 7, true; "right-open range does contain")]
    #[test_case(Range(Some(5), None), 3, false; "right-open range does not contain")]
    #[test_case(Mixed(vec![Value(1), Range(Some(3), Some(5))]), 4, true; "mixed set does contain")]
    #[test_case(Mixed(vec![Value(1), Range(Some(3), Some(5))]), 2, false; "mixed set does not contain")]
    fn contains(fields: Set<u32>, example: u32, expected: bool) {
        assert!(fields.contains(&example) == expected);
    }
}
