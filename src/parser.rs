use crate::matcher::Pattern::{self, Range, Value};
use std::{cmp::Ordering, fmt::Display};

const MIN: usize = 1;
const MAX: usize = usize::MAX;

#[derive(Debug, PartialEq)]
pub enum Error {
    CannotParse,
    StartsAtOne,
    Empty,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        let msg = match self {
            CannotParse => "cannot parse the pattern",
            Empty => "no fields specified",
            StartsAtOne => "numbering starts at 1",
        };
        write!(f, "{}", msg)
    }
}

/// Translate from 1-based indexing to 0-based
#[inline]
fn change_base(value: usize) -> usize {
    if value == usize::MAX {
        value
    } else {
        value - 1
    }
}

impl Pattern {
    /// Validate the value and transform from 1-based indexing to 0-based, return `Pattern::Value`
    fn maybe_value(val: usize) -> Result<Self, Error> {
        if val < MIN {
            return Err(Error::StartsAtOne);
        }
        Ok(Value(change_base(val)))
    }

    /// Validate the values and transform from 1-based indexing to 0-based, return `Pattern::Value` or `Pattern::Range`
    fn maybe_range(min: usize, max: usize) -> Result<Self, Error> {
        if min < MIN {
            return Err(Error::StartsAtOne);
        }
        match min.cmp(&max) {
            Ordering::Less => Ok(Range(change_base(min), change_base(max))),
            Ordering::Greater => Self::maybe_range(max, min),
            Ordering::Equal => Self::maybe_value(min),
        }
    }
}

/// Try parsing characters as an integer
#[inline]
fn try_parse_usize(chars: &[char]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }
    // those characters are guaranteed to be numbers,
    // because the parser below only allows numbers in the main for loop
    debug_assert!(chars.iter().all(|c| c.is_ascii_digit()));

    Some(chars.iter().fold(0, |acc, c| acc * 10 + *c as usize - 48))
}

/// Parse patterns from a string
pub fn from_str(s: &str) -> Result<Vec<Pattern>, Error> {
    /// On reaching the boundary of the field collect it
    #[inline]
    fn collect(
        patterns: &mut Vec<Pattern>,
        digits: &[char],
        range_start: usize,
        is_range: bool,
    ) -> Result<(), Error> {
        let num = try_parse_usize(digits);
        if is_range {
            let range_end = num.unwrap_or(MAX);
            patterns.push(Pattern::maybe_range(range_start, range_end)?);
        } else if let Some(num) = num {
            patterns.push(Pattern::maybe_value(num)?)
        }
        // if there was no value, we don't care
        Ok(())
    }

    let mut patterns = Vec::new();
    let mut range_start = MIN;
    let mut digits = Vec::new();
    let mut is_range = false;

    // the parser
    for c in s.chars() {
        match c {
            '0'..='9' => {
                // collect the digits
                digits.push(c)
            }
            '-' | ':' => {
                // it is a range, try parsing the lower bound and start parsing the upper bound
                range_start = try_parse_usize(&digits).unwrap_or(MIN);
                digits.clear();
                is_range = true;
            }
            ',' => {
                // collect previous value and start parsing new one
                collect(&mut patterns, &digits, range_start, is_range)?;
                digits.clear();
                range_start = MIN;
                is_range = false;
            }
            c => {
                if !c.is_whitespace() {
                    return Err(Error::CannotParse);
                }
            }
        };
    }

    // the last pattern is not delimited by `,` so we need to collect it here
    collect(&mut patterns, &digits, range_start, is_range)?;

    if patterns.is_empty() {
        Err(Error::Empty)
    } else {
        Ok(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use crate::matcher::Pattern::{self, Range, Value};
    use test_case::test_case;

    #[test_case(&[], None; "empty")]
    #[test_case(&['0'], Some(0); "zero")]
    #[test_case(&['9', '9'], Some(99); "ninety-nine")]
    #[test_case(&['7'], Some(7); "single digit")]
    #[test_case(&['4', '2'], Some(42); "two digits")]
    #[test_case(&['1', '0', '3', '7'], Some(1037); "four digits")]
    fn try_parse_usize(input: &[char], expected: Option<usize>) {
        assert_eq!(super::try_parse_usize(input), expected);
    }

    #[test_case("7", &[Value(6)]; "single value")]
    #[test_case(",7,,,", &[Value(6)]; "ignore redundant commas")]
    #[test_case("17", &[Value(16)]; "double-digit value")]
    #[test_case("1,2,3", &[Value(0),Value(1),Value(2)]; "comma-separated values")]
    #[test_case("-3", &[Range(0, 2)]; "range without start")]
    #[test_case("3-", &[Range(2, usize::MAX)]; "range without end")]
    #[test_case("-", &[Range(0, usize::MAX)]; "range without start and end")]
    #[test_case("1-3", &[Range(0, 2)]; "simple range")]
    #[test_case("3-1", &[Range(0, 2)]; "reversed range")]
    #[test_case("42-42", &[Value(41)]; "not range but value")]
    #[test_case("1-2, 4-5", &[Range(0, 1), Range(3, 4)]; "two ranges")]
    #[test_case("-3, 4, 5-7, 9-", &[Range(0, 2), Value(3), Range(4, 6), Range(8, usize::MAX)]; "mixed")]
    #[test_case("1:3,:5,5:", &[Range(0, 2), Range(0, 4), Range(4, usize::MAX)]; "ranges defined using colons")]
    fn from_str(input: &str, expected: &[Pattern]) {
        assert_eq!(super::from_str(input).unwrap(), expected);
    }

    #[test_case(""; "empty")]
    #[test_case("0"; "zero")]
    #[test_case("0-5"; "indexing starts at 1")]
    #[test_case("1-%^&5"; "invalid chars")]
    #[test_case("a-z"; "non-numbers")]
    #[test_case("1-5, 3, X, 7-9"; "invalid char in the middle")]
    fn from_str_raises_error(example: &str) {
        assert!(super::from_str(example).is_err());
    }

    #[test_case(0, Err(Error::StartsAtOne); "zero")]
    #[test_case(42, Ok(Value(41)); "value")]
    fn maybe_value(example: usize, expected: Result<Pattern, Error>) {
        assert_eq!(Pattern::maybe_value(example), expected)
    }

    #[test_case(0, 5, Err(Error::StartsAtOne); "zero")]
    #[test_case(5, 0, Err(Error::StartsAtOne); "zero in reversed")]
    #[test_case(2, 5, Ok(Range(1, 4)); "range")]
    #[test_case(5, 2, Ok(Range(1, 4)); "range reversed")]
    fn maybe_range(min: usize, max: usize, expected: Result<Pattern, Error>) {
        assert_eq!(Pattern::maybe_range(min, max), expected)
    }
}
