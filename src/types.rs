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
    // a class object
    Object(&'static str, *const ClassDef),
    // type [][]...
    Array(Box<SemanticType>),
    // refer to a method, only possible in semantic analysis
    Method(*const MethodDef),
    // a class, e.g., the type of `Main` in Main.f()
    Class(*const ClassDef),
}

impl Clone for SemanticType {
    fn clone(&self) -> Self {
        match &self {
            SemanticType::Error => SemanticType::Error,
            SemanticType::Var => SemanticType::Var,
            SemanticType::Null => SemanticType::Null,
            SemanticType::Basic(name) => SemanticType::Basic(name),
            SemanticType::Object(name, class) => SemanticType::Object(name, *class),
            SemanticType::Array(elem) => SemanticType::Array(elem.clone()),
            SemanticType::Method(method) => SemanticType::Method(*method),
            SemanticType::Class(class) => SemanticType::Class(*class),
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
            SemanticType::Object(name, _) => "class : ".to_string() + name,
            SemanticType::Array(elem) => elem.to_string() + "[]",
            SemanticType::Method(method) => {
                // TODO remove the duplicate code
                let method = unsafe { &**method };
                let mut s = method.loc.to_string() + " -> " + if method.static_ { "static " } else { "" } + "function "
                    + method.name + " : ";
                for parameter in &method.parameters {
                    s += &(parameter.type_.to_string() + "->");
                }
                s + &method.return_type.to_string()
            }
            _ => unreachable!()
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
            (SemanticType::Object(_, class1), SemanticType::Object(_, class2)) => {
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
        self == &ERROR
    }

    pub fn is_void(&self) -> bool {
        self == &VOID
    }

    pub fn is_object(&self) -> bool {
        match self {
            SemanticType::Object(_, _) => true,
            _ => false,
        }
    }

    pub fn is_method(&self) -> bool {
        match self {
            SemanticType::Method(_) => true,
            _ => false,
        }
    }

    pub fn print_ast(&self, printer: &mut IndentPrinter) {
        match self {
            SemanticType::Var => printer.print("var"),
            SemanticType::Basic(name) => printer.print(&(name.to_string() + "type")),
            SemanticType::Object(name, _) => {
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
            (SemanticType::Object(_, class1), SemanticType::Object(_, class2)) => class1 == class2,
            (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
            _ => false,
        }
    }
}

impl Eq for SemanticType {}