extern crate common;

use std::string::ToString;
use common::{Location, NO_LOCATION};

pub trait IError {
    fn get_msg(&self) -> String;
}

pub struct Error {
    pub loc: Location,
    pub object: Box<IError>,
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self.loc {
            NO_LOCATION => format!("*** Error: {}", self.object.get_msg()),
            loc => format!("*** Error at {}: {}", loc, &self.object.get_msg()),
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
