use super::ast::*;
use super::types::SemanticType;
use super::loc::*;
use super::util::*;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::default::Default as D;
use std::ptr;

// ast node owns the scope
#[derive(Debug, Default)]
pub struct Scope {
  pub symbols: HashMap<&'static str, Symbol>,
  pub kind: ScopeKind,
}

#[derive(Debug, Copy, Clone)]
pub enum ScopeKind {
  // only for Default::default()
  InvalidDefaultState,
  Local(*mut Block),
  Class(*mut ClassDef),
  Global,
  Parameter(*mut MethodDef),
}

impl Deref for Scope {
  type Target = HashMap<&'static str, Symbol>;

  fn deref(&self) -> &HashMap<&'static str, Symbol> {
    &self.symbols
  }
}

impl DerefMut for Scope {
  fn deref_mut(&mut self) -> &mut HashMap<&'static str, Symbol> {
    &mut self.symbols
  }
}

impl Scope {
  pub fn get_class(&self) -> &ClassDef {
    match self.kind {
      ScopeKind::Class(class) => class.get(),
      _ => panic!("call get_class on non-class scope"),
    }
  }

  pub fn sorted(&self) -> Vec<Symbol> {
    let mut symbols: Vec<Symbol> = self.symbols.iter().map(|(_, symbol)| *symbol).collect();
    symbols.sort_by_key(|x| x.get_loc());
    symbols
  }

  pub fn is_local(&self) -> bool {
    if let ScopeKind::Local(_) = self.kind { true } else { false }
  }

  pub fn is_parameter(&self) -> bool {
    if let ScopeKind::Parameter(_) = self.kind { true } else { false }
  }

  pub fn is_class(&self) -> bool {
    if let ScopeKind::Class(_) = self.kind { true } else { false }
  }
}

impl D for ScopeKind {
  fn default() -> Self {
    ScopeKind::InvalidDefaultState
  }
}

// refer to a node in ast
#[derive(Debug, Copy, Clone)]
pub enum Symbol {
  Class(*mut ClassDef),
  Method(*mut MethodDef),
  Var(*mut VarDef),
}

impl fmt::Display for Symbol {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      Symbol::Class(class) => {
        let class = class.get();
        write!(f, "{} -> class {}", class.loc, class.name)?;
        if (*class).p_ptr.is_null() { Ok(()) } else {
          write!(f, " : {}", class.p_ptr.get().name)
        }
      }
      Symbol::Method(method) => {
        let method = method.get();
        write!(f, "{} -> {}function {} : {}", method.loc, if method.static_ { "static " } else { "" },
               method.name, SemanticType::Method(method))
      }
      Symbol::Var(var) => {
        let var = var.get();
        write!(f, "{} -> variable {}{} : {}", var.loc, if var.scope.get().is_parameter() { "@" } else { "" },
               var.name, var.type_.sem)
      }
    }
  }
}

impl Symbol {
  pub fn is_class(&self) -> bool {
    if let Symbol::Class(_) = self { true } else { false }
  }

  pub fn is_method(&self) -> bool {
    if let Symbol::Method(_) = self { true } else { false }
  }

  pub fn is_var(&self) -> bool {
    if let Symbol::Var(_) = self { true } else { false }
  }

  pub fn as_class(&self) -> &mut ClassDef {
    match self {
      Symbol::Class(class) => class.get(),
      _ => panic!("call as_class on non-class symbol"),
    }
  }

  pub fn as_method(&self) -> &mut MethodDef {
    match self {
      Symbol::Method(method) => method.get(),
      _ => panic!("call as_method on non-method symbol"),
    }
  }

  pub fn get_name(&self) -> &'static str {
    match self {
      Symbol::Class(class) => class.get().name,
      Symbol::Method(method) => method.get().name,
      Symbol::Var(var) => var.get().name,
    }
  }

  pub fn get_loc(&self) -> Loc {
    match self {
      Symbol::Class(class) => class.get().loc,
      Symbol::Method(method) => method.get().loc,
      // use finish loc here because 'get_loc' is used in 'lookup_before'
      Symbol::Var(var) => var.get().finish_loc,
    }
  }

  // for a class symbol, will return the type of its instance
  pub fn get_type(&self) -> SemanticType {
    match self {
      Symbol::Class(class) => SemanticType::Object(*class),
      Symbol::Method(method) => SemanticType::Method(*method),
      Symbol::Var(var) => var.get().type_.sem.clone(),
    }
  }
}

#[derive(Debug)]
pub struct ScopeStack {
  pub global_scope: *mut Scope,
  pub scopes: Vec<*mut Scope>,
}

impl ScopeStack {
  pub fn lookup(&self, name: &'static str, recursive: bool) -> Option<(Symbol, *const Scope)> {
    if recursive {
      for scope in self.scopes.iter().rev() {
        if let Some(symbol) = scope.get().get(name) {
          return Some((*symbol, *scope as *const _));
        }
      }
      None
    } else {
      self.scopes.last().unwrap().get().get(name)
        .map(|symbol| (*symbol, *self.scopes.last().unwrap() as *const _))
    }
  }

  pub fn lookup_before(&self, name: &'static str, loc: Loc) -> Option<Symbol> {
    for scope in self.scopes.iter().rev() {
      if let Some(symbol) = scope.get().get(name) {
        if scope.get().is_local() && symbol.get_loc() > loc {
          continue;
        }
        return Some(*symbol);
      }
    }
    None
  }

  pub fn declare(&mut self, symbol: Symbol) {
    self.scopes.last().unwrap().get().insert(symbol.get_name(), symbol);
  }

  pub fn open(&mut self, scope: &mut Scope) {
    match scope.kind {
      ScopeKind::Global => self.global_scope = scope,
      ScopeKind::Class(class) => {
        if !class.get().p_ptr.is_null() {
          self.open(&mut class.get().p_ptr.get().scope);
        }
      }
      _ => {}
    }
    self.scopes.push(scope);
  }

  pub fn close(&mut self) {
    let scope = self.scopes.pop().unwrap();
    if let ScopeKind::Class(_) = scope.get().kind {
      // all scopes in the stack except the bottom are parent of the class
      for _ in 0..self.scopes.len() - 1 { self.scopes.pop(); }
    }
  }

  pub fn cur_scope(&self) -> &mut Scope {
    self.scopes.last().unwrap().get()
  }

  pub fn lookup_class(&self, name: &'static str) -> Option<Symbol> {
    self.global_scope.get().get(name).map(|class| *class)
  }
}