use super::ast::{ClassDef, MethodDef};
use super::util::*;
use std::default::Default as D;

// the struct SemanticType in ast.rs is syntactic type(so it have field `loc`)
#[derive(Debug)]
pub enum SemanticType {
    Error,
    Var,
    Null,
    // int, string, bool, void
    Basic(&'static str),
    // user defined class
    Class(&'static str, *const ClassDef),
    // type [][]...
    Array(Box<SemanticType>),
    // refer to a method, only possible in semantic analysis
    Method(*const MethodDef),
}

impl Clone for SemanticType {
    fn clone(&self) -> Self {
        match &self {
            SemanticType::Error => SemanticType::Error,
            SemanticType::Var => SemanticType::Var,
            SemanticType::Null => SemanticType::Null,
            SemanticType::Basic(name) => SemanticType::Basic(name),
            SemanticType::Class(name, class) => SemanticType::Class(name, *class),
            SemanticType::Array(elem) => SemanticType::Array(elem.clone()),
            SemanticType::Method(method) => SemanticType::Method(*method),
        }
    }
}

pub const ERROR: SemanticType = SemanticType::Error;
pub const VAR: SemanticType = SemanticType::Var;
pub const NULL: SemanticType = SemanticType::Null;
pub const INT: SemanticType = SemanticType::Basic("int");
pub const BOOL: SemanticType = SemanticType::Basic("bool");
pub const VOID: SemanticType = SemanticType::Basic("void");
pub const STRING: SemanticType = SemanticType::Basic("string");

impl D for SemanticType {
    fn default() -> Self {
        ERROR
    }
}

impl ToString for SemanticType {
    fn to_string(&self) -> String {
        match self {
            SemanticType::Error => "error".to_string(),
            SemanticType::Var => "var".to_string(),
            SemanticType::Null => "null".to_string(),
            SemanticType::Basic(name) => name.to_string(),
            SemanticType::Class(name, _) => "class : ".to_string() + name,
            SemanticType::Array(elem) => elem.to_string() + "[]",
        }
    }
}

impl SemanticType {
    // a relationship of is-subclass-of
    pub fn extends(&self, rhs: &SemanticType) -> bool {
        match (self, rhs) {
            (SemanticType::Error, _) => true,
            (_, SemanticType::Error) => true,
            (SemanticType::Basic(name1), SemanticType::Basic(name2)) => name1 == name2,
            (SemanticType::Class(_, class1), SemanticType::Class(_, class2)) => {
                let mut class1 = *class1;
                let class2 = *class2;
                while !class1.is_null() {
                    if class1 == class2 {
                        return true;
                    }
                    class1 = unsafe { (*class1).parent_ref };
                }
                false
            }
            (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
            _ => false,
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            SemanticType::Error => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        if let SemanticType::Basic(name) = self {
            return name == &"void";
        }
        false
    }

    pub fn print_ast(&self, printer: &mut IndentPrinter) {
        match self {
            SemanticType::Var => printer.print("var"),
            SemanticType::Basic(name) => printer.print(&(name.to_string() + "type")),
            SemanticType::Class(name, _) => {
                printer.print("classtype");
                printer.print(name);
            }
            SemanticType::Array(name) => {
                printer.print("arrtype");
                name.print_ast(printer);
            }
            _ => unreachable!()
        }
    }
}

impl PartialEq for SemanticType {
    fn eq(&self, other: &SemanticType) -> bool {
        // in correct usage,  SemanticType::Var & SemanticType::Null won't be compared here
        match (self, other) {
            (SemanticType::Error, SemanticType::Error) => true,
            (SemanticType::Basic(name1), SemanticType::Basic(name2)) => name1 == name2,
            (SemanticType::Class(name1, _), SemanticType::Class(name2, _)) => name1 == name2,
            (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
            _ => false,
        }
    }
}

impl Eq for SemanticType {}