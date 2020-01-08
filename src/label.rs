use anyhow::*;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq)]
pub struct Label(pub String); // TODO: &'static str
impl From<String> for Label {
    fn from(x: String) -> Label {
        // Label(&*Box::leak(x.into_boxed_str()))
        Label(x)
    }
}
impl FromStr for Label {
    type Err = ();
    fn from_str(x: &str) -> Result<Label, ()> {
        Ok(Label::from(x.to_string()))
    }
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl AsRef<std::ffi::OsStr> for Label {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_ref()
    }
}
