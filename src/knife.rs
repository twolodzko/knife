use crate::fields::{Error, Fields};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Knife {
    fields: Fields,
    min: usize,
    max: usize,
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
            // optimization: skip unnecessary fields
            .take(self.max)
            .skip(self.min - 1)
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
        Ok(Self::new(Fields::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::Knife;
    use std::str::FromStr;
    use test_case::test_case;

    #[test_case("1", 1, true; "single value does contain itself")]
    #[test_case("1", 5, false; "single value does not contain other")]
    #[test_case("1-5", 3, true; "range does contain")]
    #[test_case("1-5", 7, false; "range does not contain")]
    #[test_case("1-5, 7", 7, true; "mixed range does contain")]
    #[test_case("1-5, 7", 6, false; "mixed range does not contain")]
    #[test_case("1, 2, 3", 3, true; "mixed values does contain")]
    #[test_case("1, 2, 3", 4, false; "mixed values does not contain")]
    #[test_case("-3, 5-7, 9-", 10, true; "ranges contain")]
    #[test_case("-3, 5-7, 9-", 8, false; "ranges not contain")]
    fn should_extract(spec: &str, example: usize, expected: bool) {
        assert!(Knife::from_str(spec).unwrap().should_extract(example) == expected);
    }

    #[test_case("1", "Mary had a little lamb.", &["Mary"]; "single field exists")]
    #[test_case("10", "Mary had a little lamb.", &[]; "field does not exist")]
    #[test_case("3-4", "Mary had a little lamb.", &["a", "little"]; "exists in range")]
    #[test_case("1, 3-4", "Mary had a little lamb.", &["Mary", "a", "little"]; "exists in mixed")]
    #[test_case("4-", "Mary had a little lamb.", &["little", "lamb."]; "take tail")]
    #[test_case("5", "Mary had a little lamb.", &["lamb."]; "last one")]
    fn extract(spec: &str, example: &str, expected: &[&str]) {
        let knife = Knife::from_str(spec).unwrap();
        assert_eq!(knife.extract(example), expected);
    }
}
