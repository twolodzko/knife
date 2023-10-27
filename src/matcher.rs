use std::{
    cmp::Ordering,
    iter::{Enumerate, Skip, Take},
    usize,
};

/// The indexes to be matched
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pattern {
    Value(usize),
    Range(usize, usize),
}

impl Pattern {
    fn min(self) -> usize {
        use Pattern::{Range, Value};
        match self {
            Value(val) => val,
            Range(val, _) => val,
        }
    }

    fn max(self) -> usize {
        use Pattern::{Range, Value};
        match self {
            Value(val) => val,
            Range(_, val) => val,
        }
    }
}

/// `Matcher` matched the pattern iteratively, in linear time or faster
#[derive(Debug, PartialEq, Clone)]
pub struct Matcher {
    position: usize,
    pattern: Vec<Pattern>,
    min: usize,
    max: usize,
}

impl Matcher {
    /// Create new `Matcher`
    pub fn new(mut pattern: Vec<Pattern>) -> Self {
        // the patterns need to be sorted if we want to iterate over them
        pattern.sort_by_key(|x| x.min());

        // the bounds are known
        let min = pattern.iter().map(|x| x.min()).min().unwrap_or(0);
        let max = pattern.iter().map(|x| x.max()).max().unwrap_or(usize::MAX);

        Self {
            position: 0,
            pattern,
            min,
            max,
        }
    }

    /// Use `Matcher` to extract the values at matching indexes from the `iterable`
    pub fn iter<I>(self, iterable: I) -> MatcherIterator<I>
    where
        I: Iterator,
    {
        MatcherIterator::new(self, iterable)
    }

    /// Check if `index` matches one of the patterns
    fn matches(&mut self, index: usize) -> bool {
        use Pattern::{Range, Value};

        if self.position >= self.pattern.len() {
            // exhausted the patterns
            return false;
        }

        let pattern = self.pattern[self.position];
        match pattern {
            Value(ref val) => match index.cmp(val) {
                Ordering::Less => {
                    // index is not yet there
                    false
                }
                Ordering::Greater => {
                    // check the next pattern
                    self.position += 1;
                    self.matches(index)
                }
                Ordering::Equal => {
                    // it's a match, move to the next pattern
                    self.position += 1;
                    true
                }
            },
            Range(min, max) => {
                if index < min {
                    // index is not yet there
                    false
                } else if index < max {
                    // within the range
                    true
                } else if index == max {
                    // reached the boundary, move to the next pattern
                    self.position += 1;
                    true
                } else {
                    // check the next pattern
                    self.position += 1;
                    self.matches(index)
                }
            }
        }
    }
}

/// Iterator returning items filtered using the `Matcher`
pub struct MatcherIterator<I>
where
    I: Iterator,
{
    matcher: Matcher,
    iterable: Skip<Take<Enumerate<I>>>,
}

impl<I: Iterator> MatcherIterator<I> {
    fn new(matcher: Matcher, iterable: I) -> Self {
        let iterable = iterable
            .enumerate()
            // optimization: skip indexes outside of the range of any pattern
            .take(matcher.max.saturating_add(1))
            .skip(matcher.min);
        Self { matcher, iterable }
    }
}

impl<I: Iterator> Iterator for MatcherIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, value) = self.iterable.next()?;
        if self.matcher.matches(index) {
            Some(value)
        } else {
            // skip this item, try the next one
            self.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Matcher,
        Pattern::{self, Range, Value},
    };
    use test_case::test_case;

    #[test_case(&[], 0, false; "empty")]
    #[test_case(&[Value(5)], 0, false; "smaller than value")]
    #[test_case(&[Value(5)], 5, true; "equal than value")]
    #[test_case(&[Value(5)], 6, false; "higher than value")]
    #[test_case(&[Range(3, 5)], 2, false; "smaller than range min")]
    #[test_case(&[Range(3, 5)], 3, true; "equal to range min")]
    #[test_case(&[Range(3, 5)], 4, true; "within the range")]
    #[test_case(&[Range(3, 5)], 5, true; "equal to range max")]
    #[test_case(&[Range(3, 5)], 6, false; "higher than range max")]
    #[test_case(&[Value(1), Value(2), Value(3)], 1, true; "matched by first value")]
    #[test_case(&[Value(1), Value(2), Value(3)], 2, true; "matched by second value")]
    #[test_case(&[Value(1), Value(2), Value(3)], 3, true; "matched by third value")]
    #[test_case(&[Range(1, 3), Range(5, 7)], 0, false; "smaller than any range")]
    #[test_case(&[Range(1, 3), Range(5, 7)], 2, true; "matched by first range")]
    #[test_case(&[Range(1, 3), Range(5, 7)], 6, true; "matched by second range")]
    #[test_case(&[Range(1, 3), Range(5, 7)], 9, false; "higher than any range")]
    #[test_case(&[Range(1, 3), Range(5, 7)], 4, false; "higher than first range and lower than second")]
    #[test_case(&[Range(1, 3), Value(5), Range(6, 7)], 5, true; "matched by value in mixed patterns")]
    #[test_case(&[Range(1, 3), Value(5), Range(6, 7)], 6, true; "matched by second range in mixed patterns")]
    fn matches(pattern: &[Pattern], example: usize, expected: bool) {
        let mut matcher = Matcher::new(pattern.to_vec());
        assert_eq!(matcher.matches(example), expected);
    }

    #[test]
    fn lower_than_any_value() {
        let mut matcher = Matcher::new(vec![Value(1), Value(2), Value(3)]);
        assert!(!matcher.matches(0), "not matched");
        assert_eq!(matcher.position, 0, "index not incremented");
    }

    #[test]
    fn higher_than_any_value() {
        let mut matcher = Matcher::new(vec![Value(1), Value(2), Value(3)]);
        assert!(!matcher.matches(6), "not matched");
        assert_eq!(matcher.position, 3, "index was incremented");

        assert!(!matcher.matches(7), "not matched");
        assert_eq!(matcher.position, 3, "index was not incremented again");
    }

    #[test]
    fn patterns_overlap() {
        let mut matcher = Matcher::new(vec![Value(2), Value(2), Value(2)]);

        assert!(matcher.matches(2), "first value was correctly matched");
        assert_eq!(matcher.position, 1, "index was incremented");

        assert!(
            !matcher.matches(3),
            "second value was correctly not matched"
        );
        assert_eq!(matcher.position, 3, "indexes were skipped as expected");
    }

    #[test_case(&[], &[false, false, false, false, false, false, false, false, false, false]; "empty")]
    #[test_case(&[Value(0)], &[true, false, false, false, false, false, false, false, false, false]; "value was first")]
    #[test_case(&[Value(9)], &[false, false, false, false, false, false, false, false, false, true]; "value was last")]
    #[test_case(&[Range(0, 2)], &[true, true, true, false, false, false, false, false, false, false]; "range subset at beginning")]
    #[test_case(&[Range(3, 5)], &[false, false, false, true, true, true, false, false, false, false]; "range subset at middle")]
    #[test_case(&[Range(8, 12)], &[false, false, false, false, false, false, false, false, true, true]; "range subset at tail")]
    #[test_case(&[Range(0, 9)], &[true, true, true, true, true, true, true, true, true, true]; "whole range")]
    #[test_case(&[Range(0, 100)], &[true, true, true, true, true, true, true, true, true, true]; "could be more")]
    #[test_case(&[Range(20, 50)], &[false, false, false, false, false, false, false, false, false, false]; "range was outside")]
    #[test_case(
        &[Value(2), Value(5)],
        &[false, false, true, false, false, true, false, false, false, false];
        "two values")
    ]
    #[test_case(
        &[Value(4), Value(13)],
        &[false, false, false, false, true, false, false, false, false, false];
        "two values but one matched")
    ]
    #[test_case(
        &[Value(2), Range(3, 5)],
        &[false, false, true, true, true, true, false, false, false, false];
        "value and range")
    ]
    #[test_case(
        &[Range(2, 5), Range(7, 8)],
        &[false, false, true, true, true, true, false, true, true, false];
        "two ranges")
    ]
    #[test_case(
        &[Range(2, 4), Range(3, 5)],
        &[false, false, true, true, true, true, false, false, false, false];
        "overlapping ranges")
    ]
    #[test_case(
        // patterns are sorted by min, so in case of overlaps this can happen
        &[Value(1), Range(1, 3), Value(1), Range(1, 5), Range(1, 4)],
        &[false, true, true, true, true, true, false, false, false, false];
        "edge case pattern")
    ]
    fn match_whole_pattern(pattern: &[Pattern], expected: &[bool]) {
        let mut matcher = Matcher::new(pattern.to_vec());
        let result: Vec<bool> = (0..=9).map(|x| matcher.matches(x)).collect();
        assert_eq!(&result, expected);
    }

    #[test_case(&[], &[]; "empty")]
    #[test_case(&[Value(5)], &[5]; "single value")]
    #[test_case(&[Range(2, 5)], &[2, 3, 4, 5]; "subset")]
    #[test_case(&[Range(7, 12)], &[7, 8, 9]; "range exceeds input")]
    #[test_case(&[Range(2, 4), Range(7, 8)], &[2, 3, 4, 7, 8]; "two ranges")]
    fn iterate(pattern: &[Pattern], expected: &[u32]) {
        let matcher = Matcher::new(pattern.to_vec());
        let iter = matcher.iter(0..=9);
        let result: Vec<u32> = iter.collect();
        assert_eq!(result, expected);
    }
}