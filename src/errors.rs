use super::loc::*;
use super::config::*;
use std::fmt;

pub trait IError {
    fn get_msg(&self) -> String;
}

pub struct Error {
    pub loc: Loc,
    pub error: Box<IError>,
}

impl Error {
    pub fn new<E: IError + 'static>(loc: Loc, error: E) -> Error {
        Error {
            loc,
            error: Box::new(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.loc {
            NO_LOC => write!(f, "*** Error: {}", self.error.get_msg()),
            loc => write!(f, "*** Error at {}: {}", loc, &self.error.get_msg()),
        }
    }
}

pub struct UnterminatedStr {
    pub string: String,
}

impl IError for UnterminatedStr {
    fn get_msg(&self) -> String {
        format!("unterminated string constant {}", self.string)
    }
}

pub struct NewlineInStr {
    pub string: String,
}

impl IError for NewlineInStr {
    fn get_msg(&self) -> String {
        format!("illegal newline in string constant {}", self.string)
    }
}

pub struct IntTooLarge {
    pub string: String,
}

impl IError for IntTooLarge {
    fn get_msg(&self) -> String {
        format!("integer literal {} is too large", self.string)
    }
}

pub struct UnrecognizedChar {
    pub ch: char,
}

impl IError for UnrecognizedChar {
    fn get_msg(&self) -> String {
        format!("unrecognized character '{}'", self.ch)
    }
}

pub struct ConflictDeclaration {
    pub earlier: Loc,
    pub name: &'static str,
}

impl IError for ConflictDeclaration {
    fn get_msg(&self) -> String {
        format!("declaration of '{}' here conflicts with earlier declaration at {}", self.name, self.earlier)
    }
}

pub struct ClassNotFound {
    pub name: &'static str,
}

impl IError for ClassNotFound {
    fn get_msg(&self) -> String {
        format!("class '{}' not found", self.name)
    }
}

pub struct CyclicInheritance;

impl IError for CyclicInheritance {
    fn get_msg(&self) -> String {
        "illegal class inheritance (should be a cyclic)".to_owned()
    }
}

pub struct SealedInheritance;

impl IError for SealedInheritance {
    fn get_msg(&self) -> String {
        "illegal class inheritance from sealed class".to_owned()
    }
}

pub struct NoMainClass;

impl IError for NoMainClass {
    fn get_msg(&self) -> String {
        format!("no legal Main class named {} was found", MAIN_CLASS)
    }
}
