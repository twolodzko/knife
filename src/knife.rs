use crate::set::Set;
use std::{fmt::Display, str::FromStr};

#[derive(Debug)]
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

type Fields = Set<usize>;

#[derive(Debug, PartialEq, Clone)]
pub struct Knife {
    fields: Fields,
    min: usize,
    max: Option<usize>,
}

impl Fields {
    // Smallest value in the `Set`, `1` or more
    fn min(&self) -> usize {
        use crate::set::Set::{Mixed, Range, Value};
        match self {
            Value(val) => *val,
            Range(val, _) => val.unwrap_or(1),
            Mixed(values) => values.iter().map(|x| x.min()).min().unwrap_or(1),
        }
    }

    // Largest value in the `Set`, if set is unbounded, it's `None`
    fn max(&self) -> Option<usize> {
        use crate::set::Set::{Mixed, Range, Value};
        match self {
            Value(val) => Some(*val),
            Range(_, val) => *val,
            Mixed(values) => {
                let mut acc = 0;
                for val in values {
                    acc = acc.max(val.max()?);
                }
                Some(acc)
            }
        }
    }
}

impl Knife {
    fn new(fields: Fields) -> Self {
        let min = fields.min();
        let max = fields.max();
        Self { fields, min, max }
    }

    /// Extract specific fields from a string
    #[inline]
    pub fn extract(&self, string: &str) -> Vec<String> {
        string
            .split_whitespace()
            .enumerate()
            // optimization: skip fields lower than the lowest field to extract
            .skip(self.min - 1)
            // optimization: skip fields higher than the highest field to extract
            .take_while(|(i, _)| match self.max {
                Some(max) => i <= &max,
                // there is no upper bound, take all
                None => true,
            })
            .filter_map(|(i, field)| {
                if self.should_extract(i + 1) {
                    Some(field.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if field is contained by the list of fields to extract
    #[inline]
    fn should_extract(&self, index: usize) -> bool {
        self.fields.contains(&index)
    }
}

impl FromStr for Knife {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Set::from_str(s)?))
    }
}

#[inline]
fn parse_uint(chars: &[char]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }
    // those characters are guaranteed to be numbers,
    // because the parser below only allows numbers in the main for loop
    debug_assert!(chars.iter().all(|c| c.is_ascii_digit()));

    Some(chars.iter().rev().enumerate().fold(0, |acc, (i, c)| {
        acc + (*c as usize - 48) * usize::pow(10, i as u32)
    }))
}

#[inline]
fn greater_than_zero(num: Option<usize>) -> Result<(), Error> {
    if let Some(num) = num {
        if num < 1 {
            return Err(Error::StartsAtOne);
        }
    }
    Ok(())
}

impl FromStr for Fields {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Set::{Range, Value};

        #[inline]
        fn collect(
            fields: &mut Vec<Fields>,
            value: &[char],
            range_start: Option<usize>,
            is_range: bool,
        ) -> Result<(), Error> {
            let num = parse_uint(value);
            greater_than_zero(num)?;

            if is_range {
                if let (Some(min), Some(max)) = (range_start, num) {
                    #[allow(clippy::comparison_chain)]
                    if min == max {
                        // is is just a single value
                        fields.push(Value(min));
                        return Ok(());
                    } else if min > max {
                        // Range defined in reversed order
                        fields.push(Range(num, range_start));
                        return Ok(());
                    }
                }
                fields.push(Range(range_start, num))
            } else if let Some(num) = num {
                fields.push(Value(num))
            }
            // if there was no value, we don't care
            Ok(())
        }

        let mut fields = Vec::new();
        let mut range_start = None;
        let mut acc = Vec::new();
        let mut is_range = false;

        // the parser
        for c in s.chars() {
            match c {
                '0'..='9' => {
                    // collect the digits
                    acc.push(c)
                }
                '-' => {
                    // it is a range, try parsing the lower bound and start parsing the upper bound
                    range_start = parse_uint(&acc);
                    greater_than_zero(range_start)?;
                    acc.clear();
                    is_range = true;
                }
                ',' => {
                    // collect previous value and start parsing new one
                    collect(&mut fields, &acc, range_start, is_range)?;
                    acc.clear();
                    range_start = None;
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
        collect(&mut fields, &acc, range_start, is_range)?;

        match fields.len() {
            0 => Err(Error::Empty),
            1 => Ok(fields.first().unwrap().clone()),
            _ => Ok(Set::Mixed(fields)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Fields, Knife,
        Set::{Mixed, Range, Value},
    };
    use std::str::FromStr;
    use test_case::test_case;

    #[test_case(&[], None; "empty")]
    #[test_case(&['7'], Some(7); "single digit")]
    #[test_case(&['4', '2'], Some(42); "two digits")]
    #[test_case(&['1', '0', '3', '7'], Some(1037); "four digits")]
    fn parse_uint(input: &[char], expected: Option<usize>) {
        assert_eq!(super::parse_uint(input), expected);
    }

    #[test_case("7", Value(7); "single value")]
    #[test_case(",7,,,", Value(7); "ignore redundant commas")]
    #[test_case("17", Value(17); "double-digit value")]
    #[test_case("1,2,3", Mixed(vec![Value(1),Value(2),Value(3)]); "comma-separated values")]
    #[test_case("-3", Range(None, Some(3)); "range without start")]
    #[test_case("1-3", Range(Some(1), Some(3)); "simple range")]
    #[test_case("3-1", Range(Some(1), Some(3)); "reversed range")]
    #[test_case("42-42", Value(42); "not range but value")]
    #[test_case("1-2, 4-5", Mixed(vec![Range(Some(1), Some(2)), Range(Some(4), Some(5))]); "two ranges")]
    #[test_case("4-5, 1-2", Mixed(vec![Range(Some(4), Some(5)), Range(Some(1), Some(2))]); "two ranges reversed")]
    #[test_case("-3, 4, 5-7, 9-", Mixed(vec![Range(None, Some(3)), Value(4), Range(Some(5), Some(7)), Range(Some(9), None)]); "mixed")]
    fn from_str(input: &str, expected: Fields) {
        assert_eq!(Knife::from_str(input).unwrap(), Knife::new(expected));
    }

    #[test_case(""; "empty")]
    #[test_case("0"; "zero")]
    #[test_case("0-5"; "indexing starts at 1")]
    #[test_case("1-%^&5"; "invalid chars")]
    #[test_case("a-z"; "non-numbers")]
    #[test_case("1-5, 3, X, 7-9"; "invalid char in the middle")]
    fn from_str_raises_error(example: &str) {
        assert!(Knife::from_str(example).is_err());
    }

    #[test_case("1", 1, true; "single value does contain itself")]
    #[test_case("1", 5, false; "single value does not contain other")]
    #[test_case("1-5", 3, true; "range does contain")]
    #[test_case("1-5", 7, false; "range does not contain")]
    #[test_case("1-5, 7", 7, true; "mixed does contain 1")]
    #[test_case("1-5, 7", 6, false; "mixed does not contain 1")]
    #[test_case("1, 2, 3", 3, true; "mixed does contain 2")]
    #[test_case("1, 2, 3", 4, false; "mixed does not contain 2")]
    #[test_case("-3, 5-7, 9-", 10, true; "ranges contain")]
    #[test_case("-3, 5-7, 9-", 8, false; "ranges not contain")]
    fn knife_should_extract(spec: &str, example: usize, expected: bool) {
        assert!(Knife::from_str(spec).unwrap().should_extract(example) == expected);
    }

    #[test_case("1", "Mary had a little lamb.", &["Mary"]; "single field exists")]
    #[test_case("10", "Mary had a little lamb.", &[]; "field does not exist")]
    #[test_case("3-4", "Mary had a little lamb.", &["a", "little"]; "exists in range")]
    #[test_case("1, 3-4", "Mary had a little lamb.", &["Mary", "a", "little"]; "exists in mixed")]
    fn knife_extract(spec: &str, example: &str, expected: &[&str]) {
        let knife = Knife::from_str(spec).unwrap();
        assert_eq!(knife.extract(example), expected);
    }

    #[test_case(Value(7), Some(7); "single value")]
    #[test_case(Range(Some(1), Some(5)), Some(5); "upper bound of the range")]
    #[test_case(Range(None, Some(3)), Some(3); "upper bound of the left-open range")]
    #[test_case(Range(Some(10), None), None; "no max for right-open range")]
    #[test_case(Mixed(vec![Value(4), Value(2), Value(5), Value(1)]), Some(5); "max of mixed values")]
    #[test_case(Mixed(vec![Value(4), Value(5), Range(Some(2), Some(3)), Value(1)]), Some(5); "max of mixed values and ranges is value")]
    #[test_case(Mixed(vec![Value(4), Value(5), Range(Some(2), Some(6)), Value(1)]), Some(6); "max of mixed values and ranges is range")]
    #[test_case(Mixed(vec![Range(Some(1), Some(4)), Range(Some(2), Some(3))]), Some(4); "max of ranges")]
    #[test_case(Mixed(vec![Range(Some(1), Some(4)), Range(Some(2), Some(3)), Range(Some(2), None), Range(Some(2), Some(100))]), None; "max of ranges with right-open range")]
    fn set_max(example: Fields, expected: Option<usize>) {
        assert_eq!(example.max(), expected)
    }
}
