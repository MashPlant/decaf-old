use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Loc(pub i32, pub i32);

pub const NO_LOC: Loc = Loc(-1, -1);

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}