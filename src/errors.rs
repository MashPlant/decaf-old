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

pub struct VoidArrayElement;

impl IError for VoidArrayElement {
    fn get_msg(&self) -> String {
        "array element type must be non-void known type".to_owned()
    }
}

pub struct VoidVar {
    pub name: &'static str,
}

impl IError for VoidVar {
    fn get_msg(&self) -> String {
        format!("cannot declare identifier '{}' as void type", self.name)
    }
}

pub struct OverrideVar {
    pub name: &'static str,
}

impl IError for OverrideVar {
    fn get_msg(&self) -> String {
        format!("overriding variable is not allowed for var '{}'", self.name)
    }
}

pub struct BadOverride {
    pub method: &'static str,
    pub parent: &'static str,
}

impl IError for BadOverride {
    fn get_msg(&self) -> String {
        format!("overriding method '{}' doesn't match the type signature in class '{}'", self.method, self.parent)
    }
}

pub struct IncompatibleUnary {
    pub opt: &'static str,
    pub type_: String,
}

impl IError for IncompatibleUnary {
    fn get_msg(&self) -> String {
        format!("incompatible operand: {} {}", self.opt, self.type_)
    }
}

pub struct TestNotBool;

impl IError for TestNotBool {
    fn get_msg(&self) -> String {
        "test expression must have bool type".to_string()
    }
}

pub struct IncompatibleBinary {
    pub left_t: String,
    pub opt: &'static str,
    pub right_t: String,
}

impl IError for IncompatibleBinary {
    fn get_msg(&self) -> String {
        format!("incompatible operands: {} {} {}", self.left_t, self.opt, self.right_t)
    }
}

pub struct BreakOutOfLoop;

impl IError for BreakOutOfLoop {
    fn get_msg(&self) -> String {
        "'break' is only allowed inside a loop".to_string()
    }
}

pub struct UndeclaredVar {
    pub name: &'static str,
}

impl IError for UndeclaredVar {
    fn get_msg(&self) -> String {
        format!("undeclared variable '{}'", self.name)
    }
}

pub struct RefInStatic {
    pub field: &'static str,
    pub method: &'static str,
}

impl IError for RefInStatic {
    fn get_msg(&self) -> String {
        format!("can not reference a non-static field '{}' from static method '{}'", self.field, self.method)
    }
}

pub struct BadFieldAccess {
    pub name: &'static str,
    pub owner_t: String,
}

impl IError for BadFieldAccess {
    fn get_msg(&self) -> String {
        format!("cannot access field '{}' from '{}'", self.name, self.owner_t)
    }
}

pub struct PrivateFieldAccess {
    pub name: &'static str,
    pub owner_t: String,
}

impl IError for PrivateFieldAccess {
    fn get_msg(&self) -> String {
        format!("field '{}' of '{}' not accessible here", self.name, self.owner_t)
    }
}

pub struct NoSuchField {
    pub name: &'static str,
    pub owner_t: String,
}

impl IError for NoSuchField {
    fn get_msg(&self) -> String {
        format!("field '{}' not found in '{}'", self.name, self.owner_t)
    }
}

pub struct LengthWithArgument {
    pub count: i32,
}

impl IError for LengthWithArgument {
    fn get_msg(&self) -> String {
        format!("function 'length' expects 0 argument(s) but {} given", self.count)
    }
}

pub struct BadLength;

impl IError for BadLength {
    fn get_msg(&self) -> String {
        "'length' can only be applied to arrays".to_string()
    }
}

// want a method, found only a field
pub struct NotMethod {
    pub name: &'static str,
    pub owner_t: String,
}

impl IError for NotMethod {
    fn get_msg(&self) -> String {
        format!("'{}' is not a method in class '{}'", self.name, self.owner_t)
    }
}

pub struct WrongArgc {
    pub name: &'static str,
    pub expect: i32,
    pub actual: i32,
}

impl IError for WrongArgc {
    fn get_msg(&self) -> String {
        format!("function '{}' expects {} argument(s) but {} given", self.name, self.expect, self.actual)
    }
}

pub struct WrongArgType {
    pub loc: i32,
    pub arg_t: String,
    pub param_t: String,
}

impl IError for WrongArgType {
    fn get_msg(&self) -> String {
        format!("incompatible argument {}: {} given, {} expected", self.loc, self.arg_t, self.param_t)
    }
}