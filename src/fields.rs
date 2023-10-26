use crate::set::Set::{self, Mixed, Range, Value};
use std::{cmp::Ordering, fmt::Display, str::FromStr};

const MIN: usize = 1;
const MAX: usize = usize::MAX;
pub(crate) type Fields = Set<usize>;

impl Fields {
    /// `Fields` constructor
    fn maybe_value(val: usize) -> Result<Self, Error> {
        if val < MIN {
            return Err(Error::StartsAtOne);
        }
        Ok(Value(val))
    }

    /// `Fields` constructor
    fn maybe_range(min: usize, max: usize) -> Result<Self, Error> {
        if min < MIN {
            return Err(Error::StartsAtOne);
        }
        match min.cmp(&max) {
            Ordering::Less => Ok(Range(min, max)),
            Ordering::Greater => Self::maybe_range(max, min),
            Ordering::Equal => Ok(Value(min)),
        }
    }

    /// `Fields` constructor
    fn maybe_mixed(mut values: Vec<Self>) -> Result<Self, Error> {
        match values.len() {
            0 => Err(Error::Empty),
            1 => Ok(values.first().unwrap().clone()),
            _ => {
                // optimization: later fields may not even exist, so let's check them later
                values.sort_by_key(|x| x.min());
                Ok(Mixed(values))
            }
        }
    }

    /// Smallest value contained in `Fields`, `1` or more
    pub(crate) fn min(&self) -> usize {
        match self {
            Value(val) => *val,
            Range(val, _) => *val,
            Mixed(values) => values.iter().map(|x| x.min()).min().unwrap_or(MIN),
        }
    }

    /// Largest value contained in `Fields`
    pub(crate) fn max(&self) -> usize {
        match self {
            Value(val) => *val,
            Range(_, val) => *val,
            Mixed(values) => values.iter().map(|x| x.max()).max().unwrap_or(MAX),
        }
    }
}

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

impl FromStr for Fields {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /// On reaching the boundary of the field collect it
        #[inline]
        fn collect(
            fields: &mut Vec<Fields>,
            digits: &[char],
            range_start: usize,
            is_range: bool,
        ) -> Result<(), Error> {
            let num = try_parse_usize(digits);
            if is_range {
                let range_end = num.unwrap_or(MAX);
                fields.push(Fields::maybe_range(range_start, range_end)?);
            } else if let Some(num) = num {
                fields.push(Fields::maybe_value(num)?)
            }
            // if there was no value, we don't care
            Ok(())
        }

        let mut fields = Vec::new();
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
                '-' => {
                    // it is a range, try parsing the lower bound and start parsing the upper bound
                    range_start = try_parse_usize(&digits).unwrap_or(MIN);
                    digits.clear();
                    is_range = true;
                }
                ',' => {
                    // collect previous value and start parsing new one
                    collect(&mut fields, &digits, range_start, is_range)?;
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
        collect(&mut fields, &digits, range_start, is_range)?;

        Fields::maybe_mixed(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Error, Fields,
        Set::{Mixed, Range, Value},
        MAX,
    };
    use std::str::FromStr;
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

    #[test_case("7", Value(7); "single value")]
    #[test_case(",7,,,", Value(7); "ignore redundant commas")]
    #[test_case("17", Value(17); "double-digit value")]
    #[test_case("1,2,3", Mixed(vec![Value(1),Value(2),Value(3)]); "comma-separated values")]
    #[test_case("-3", Range(1, 3); "range without start")]
    #[test_case("3-", Range(3, MAX); "range without end")]
    #[test_case("-", Range(1, MAX); "range without start and end")]
    #[test_case("1-3", Range(1, 3); "simple range")]
    #[test_case("3-1", Range(1, 3); "reversed range")]
    #[test_case("42-42", Value(42); "not range but value")]
    #[test_case("1-2, 4-5", Mixed(vec![Range(1, 2), Range(4, 5)]); "two ranges")]
    #[test_case("4-5, 1-2", Mixed(vec![Range(1, 2), Range(4, 5)]); "two ranges reversed")]
    #[test_case("-3, 4, 5-7, 9-", Mixed(vec![Range(1,3), Value(4), Range(5,7), Range(9, MAX)]); "mixed")]
    fn from_str(input: &str, expected: Fields) {
        assert_eq!(Fields::from_str(input).unwrap(), expected);
    }

    #[test_case(""; "empty")]
    #[test_case("0"; "zero")]
    #[test_case("0-5"; "indexing starts at 1")]
    #[test_case("1-%^&5"; "invalid chars")]
    #[test_case("a-z"; "non-numbers")]
    #[test_case("1-5, 3, X, 7-9"; "invalid char in the middle")]
    fn from_str_raises_error(example: &str) {
        assert!(Fields::from_str(example).is_err());
    }

    #[test_case(Value(7), 7; "single value")]
    #[test_case(Range(2, 5), 2; "range")]
    #[test_case(Mixed(vec![Value(4), Value(2), Value(5), Value(1)]), 1; "mixed values")]
    #[test_case(Mixed(vec![Value(4), Value(2), Range(3, 5), Value(5)]), 2; "values and ranges is value")]
    #[test_case(Mixed(vec![Value(4), Value(5), Range(1, 6), Value(2)]), 1; "values and ranges is range")]
    #[test_case(Mixed(vec![Range(3, 4), Range(2, 3)]), 2; "ranges")]
    fn min(example: Fields, expected: usize) {
        assert_eq!(example.min(), expected)
    }

    #[test_case(Value(7), 7; "single value")]
    #[test_case(Range(2, 5), 5; "range")]
    #[test_case(Mixed(vec![Value(4), Value(2), Value(5), Value(1)]), 5; "mixed values")]
    #[test_case(Mixed(vec![Value(4), Value(5), Range(2, 3), Value(1)]), 5; "values and ranges is value")]
    #[test_case(Mixed(vec![Value(4), Value(5), Range(2, 6), Value(1)]), 6; "values and ranges is range")]
    #[test_case(Mixed(vec![Range(1, 4), Range(2, 3)]), 4; "ranges")]
    fn max(example: Fields, expected: usize) {
        assert_eq!(example.max(), expected)
    }

    #[test_case(0, Err(Error::StartsAtOne); "zero")]
    #[test_case(42, Ok(Value(42)); "value")]
    fn maybe_value(example: usize, expected: Result<Fields, Error>) {
        assert_eq!(Fields::maybe_value(example), expected)
    }

    #[test_case(0, 5, Err(Error::StartsAtOne); "zero")]
    #[test_case(5, 0, Err(Error::StartsAtOne); "zero in reversed")]
    #[test_case(2, 5, Ok(Range(2, 5)); "range")]
    #[test_case(5, 2, Ok(Range(2, 5)); "range reversed")]
    fn maybe_range(min: usize, max: usize, expected: Result<Fields, Error>) {
        assert_eq!(Fields::maybe_range(min, max), expected)
    }

    #[test_case(vec![], Err(Error::Empty); "empty")]
    #[test_case(vec![Range(1, 3)], Ok(Range(1, 3)); "single value")]
    #[test_case(vec![Range(1, 3), Value(6)], Ok(Mixed(vec![Range(1, 3), Value(6)])); "sorted")]
    #[test_case(vec![Value(6), Range(1, 3)], Ok(Mixed(vec![Range(1, 3), Value(6)])); "unsorted")]
    fn maybe_mixed(fields: Vec<Fields>, expected: Result<Fields, Error>) {
        assert_eq!(Fields::maybe_mixed(fields), expected)
    }
}
