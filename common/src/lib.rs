use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Location(pub i32, pub i32);

pub const NO_LOCATION: Location = Location(-1, -1);

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.0, self.1)
    }
}