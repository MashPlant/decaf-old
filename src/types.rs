use super::ast::{ClassDef, MethodDef};
use super::print::*;
use super::errors::*;
use super::loc::*;
use super::symbol::Symbol;

use std::default::Default as D;
use std::fmt;

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

impl fmt::Display for SemanticType {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      SemanticType::Error => write!(f, "error"),
      SemanticType::Var => write!(f, "var"),
      SemanticType::Null => write!(f, "null"),
      SemanticType::Basic(name) => write!(f, "{}", name),
      SemanticType::Object(name, _) => write!(f, "class : {}", name),
      SemanticType::Class(class) => write!(f, "class : {}", unsafe { &**class }.name),
      SemanticType::Array(elem) => write!(f, "{}[]", elem),
      SemanticType::Method(method) => {
        let method = unsafe { &**method };
        for parameter in &method.params {
          write!(f, "{}->", parameter.type_.sem);
        }
        write!(f, "{}", method.ret_t.sem)
      }
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
      (SemanticType::Object(_, class1), SemanticType::Object(_, class2)) => unsafe { (&**class1).extends(*class2) },
      (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
      (SemanticType::Null, SemanticType::Object(_, _)) => true,
      _ => false,
    }
  }

  pub fn is_class(&self) -> bool {
    match self {
      SemanticType::Class(_) => true,
      _ => false,
    }
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

  pub fn is_array(&self) -> bool {
    match self {
      SemanticType::Array(_) => true,
      _ => false,
    }
  }

  pub fn get_class(&self) -> &ClassDef {
    unsafe {
      match self {
        SemanticType::Object(_, class) => &**class,
        SemanticType::Class(class) => &**class,
        _ => panic!("call get_class on non-class & non-object type"),
      }
    }
  }

  pub fn error_or(&self, require: &SemanticType) -> bool {
    self == &ERROR || self == require
  }

  pub fn error_or_array(&self) -> bool {
    match self {
      SemanticType::Error | SemanticType::Array(_) => true,
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
    // in correct usage, SemanticType::Null won't be compared here
    match (self, other) {
      (SemanticType::Var, SemanticType::Var) => true,
      (SemanticType::Error, SemanticType::Error) => true,
      (SemanticType::Basic(name1), SemanticType::Basic(name2)) => name1 == name2,
      (SemanticType::Object(_, class1), SemanticType::Object(_, class2)) => class1 == class2,
      (SemanticType::Array(elem1), SemanticType::Array(elem2)) => elem1 == elem2,
      _ => false,
    }
  }
}

impl Eq for SemanticType {}

pub trait SemanticTypeVisitor {
  fn push_error(&mut self, error: Error);

  fn lookup_class(&self, name: &'static str) -> Option<Symbol>;

  fn visit_semantic_type(&mut self, type_: &mut SemanticType, loc: Loc) {
    if match type_ { // work around with borrow check
      SemanticType::Object(name, ref mut class) =>
        if let Some(class_symbol) = self.lookup_class(name) {
          *class = class_symbol.as_class();
          false
        } else {
          self.push_error(Error::new(loc, NoSuchClass { name }));
          true
        }
      SemanticType::Array(elem_type) => {
        self.visit_semantic_type(elem_type, loc);
        if elem_type.as_ref() == &ERROR {
          true
        } else if elem_type.as_ref() == &VOID {
          self.push_error(Error::new(loc, VoidArrayElement));
          true
        } else { false }
      }
      _ => false,
    } { *type_ = ERROR; }
  }
}