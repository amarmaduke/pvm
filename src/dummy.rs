use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dummy(i32);

impl FromStr for Dummy {
    type Err = usize;

    fn from_str(_ : &str) -> Result<Self, Self::Err> {
        Ok(Dummy(0))
    }
}
