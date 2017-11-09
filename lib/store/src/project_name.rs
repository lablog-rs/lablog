use std::convert::Into;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FResult;
use std::str::FromStr;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Default)]
pub struct ProjectName(String);

impl Display for ProjectName {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        write!(f, "{}", self.0)
    }
}

impl<'a> Into<&'a str> for &'a ProjectName {
    fn into(self) -> &'a str {
        (self.0).as_str()
    }
}

pub const PROJECT_SEPPERATOR: &str = ".";

impl ProjectName {
    /// generates a unix path out of the project name
    pub fn normalize_path(&self) -> String {
        self.0.replace(PROJECT_SEPPERATOR, "/")
    }
}

impl From<String> for ProjectName {
    fn from(string: String) -> Self {
        ProjectName(string)
    }
}

impl<'a> From<&'a str> for ProjectName {
    fn from(string: &'a str) -> Self {
        ProjectName(string.into())
    }
}

impl FromStr for ProjectName {
    type Err = String;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(string.into())
    }
}

#[cfg(test)]
mod project_name_test {
    use project_name::ProjectName;

    #[test]
    fn normalize_path() {
        let expected = "/test/test/test";
        let got = ProjectName(".test.test.test".to_string()).normalize_path();

        assert_eq!(expected, got)
    }
}
