use crate::matcher::{Matcher, Pattern};
use crate::parser::{self, Error};
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub struct Knife {
    matcher: Matcher,
}

impl Knife {
    fn new(pattern: Vec<Pattern>) -> Self {
        let matcher = Matcher::new(pattern);
        Self { matcher }
    }

    /// Extract specific fields from a string
    #[inline]
    pub fn extract<'a>(&self, string: &'a str) -> Vec<&'a str> {
        let chunks = string.split_whitespace();
        self.matcher.clone().iter(chunks).collect()
    }
}

impl FromStr for Knife {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(parser::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::Knife;
    use std::str::FromStr;
    use test_case::test_case;

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
