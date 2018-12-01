use super::ast::*;
use super::types::SemanticType;
use super::loc::*;
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
    unsafe {
      match self.kind {
        ScopeKind::Class(class) => &*class,
        _ => panic!("call get_class on non-class scope"),
      }
    }
  }

  pub fn sorted(&self) -> Vec<Symbol> {
    let mut symbols: Vec<Symbol> = self.symbols.iter().map(|(_, symbol)| *symbol).collect();
    symbols.sort_by_key(|x| x.get_loc());
    symbols
  }

  pub fn is_local(&self) -> bool {
    match self.kind {
      ScopeKind::Local(_) => true,
      _ => false,
    }
  }

  pub fn is_parameter(&self) -> bool {
    match self.kind {
      ScopeKind::Parameter(_) => true,
      _ => false,
    }
  }

  pub fn is_class(&self) -> bool {
    match self.kind {
      ScopeKind::Class(_) => true,
      _ => false,
    }
  }
}

impl D for ScopeKind {
  fn default() -> Self {
    ScopeKind::InvalidDefaultState
  }
}

#[derive(Debug, Copy, Clone)]
pub enum Var {
  VarDef(*const VarDef),
  VarAssign(*const VarAssign),
}

impl D for Var {
  fn default() -> Self {
    Var::VarDef(ptr::null())
  }
}

impl Var {
  pub fn get_loc(&self) -> Loc {
    unsafe {
      match self {
        Var::VarDef(var_def) => (**var_def).loc,
        Var::VarAssign(var_assign) => (**var_assign).finish_loc,
      }
    }
  }

  pub fn get_type(&self) -> &SemanticType {
    unsafe {
      match self {
        Var::VarDef(var_def) => &(**var_def).type_.sem,
        Var::VarAssign(var_assign) => &(**var_assign).type_,
      }
    }
  }

  pub fn get_name(&self) -> &'static str {
    unsafe {
      match self {
        Var::VarDef(var_def) => (**var_def).name,
        Var::VarAssign(var_assign) => (**var_assign).name,
      }
    }
  }

  pub fn is_param(&self) -> bool {
    unsafe {
      match self {
        Var::VarDef(var_def) => (*(**var_def).scope).is_parameter(),
        Var::VarAssign(_) => false,
      }
    }
  }

  pub fn get_scope(&self) -> &Scope {
    unsafe {
      match self {
        Var::VarDef(var_def) => &*(**var_def).scope,
        Var::VarAssign(var_assign) => &*(**var_assign).scope,
      }
    }
  }
}

// refer to a node in ast
#[derive(Debug, Copy, Clone)]
pub enum Symbol {
  Class(*mut ClassDef),
  Method(*mut MethodDef),
  Var(Var),
}

impl fmt::Display for Symbol {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    unsafe {
      match self {
        Symbol::Class(class) => {
          let class = &**class;
          write!(f, "{} -> class {}", class.loc, class.name);
          if (*class).p_ptr.is_null() { Ok(()) } else {
            write!(f, " : {}", (*class.p_ptr).name)
          }
        }
        Symbol::Method(method) => {
          let method = &**method;
          write!(f, "{} -> {}function {} : {}", method.loc, if method.static_ { "static " } else { "" },
                 method.name, SemanticType::Method(method))
        }
        Symbol::Var(var) => {
          write!(f, "{} -> variable {}{} : {}", var.get_loc(), if var.is_param() { "@" } else { "" },
                 var.get_name(), var.get_type())
        }
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
      Symbol::Class(class) => unsafe { &mut **class },
      _ => panic!("call as_class on non-class symbol"),
    }
  }

  pub fn as_method(&self) -> &mut MethodDef {
    match self {
      Symbol::Method(method) => unsafe { &mut **method },
      _ => panic!("call as_method on non-method symbol"),
    }
  }

  pub fn get_name(&self) -> &'static str {
    unsafe {
      match self {
        Symbol::Class(class) => (**class).name,
        Symbol::Method(method) => (**method).name,
        Symbol::Var(var) => var.get_name(),
      }
    }
  }

  pub fn get_loc(&self) -> Loc {
    unsafe {
      match self {
        Symbol::Class(class) => (**class).loc,
        Symbol::Method(method) => (**method).loc,
        Symbol::Var(var) => var.get_loc(),
      }
    }
  }

  // for a class symbol, will return the type of its instance
  pub fn get_type(&self) -> SemanticType {
    match self {
      Symbol::Class(class) => SemanticType::Object(*class),
      Symbol::Method(method) => SemanticType::Method(*method),
      Symbol::Var(var) => var.get_type().clone(),
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
    unsafe {
      if recursive {
        for scope in self.scopes.iter().rev() {
          if let Some(symbol) = (**scope).get(name) {
            return Some((*symbol, *scope as *const _));
          }
        }
        None
      } else {
        (**self.scopes.last().unwrap()).get(name)
          .map(|symbol| (*symbol, *self.scopes.last().unwrap() as *const _))
      }
    }
  }

  pub fn lookup_before(&self, name: &'static str, loc: Loc) -> Option<Symbol> {
    unsafe {
      for scope in self.scopes.iter().rev() {
        if let Some(symbol) = (**scope).get(name) {
          if (**scope).is_local() && symbol.get_loc() > loc {
            continue;
          }
          return Some(*symbol);
        }
      }
      None
    }
  }

  pub fn declare(&mut self, symbol: Symbol) {
    unsafe {
      (**self.scopes.last().unwrap()).insert(symbol.get_name(), symbol);
    }
  }

  pub fn open(&mut self, scope: &mut Scope) {
    unsafe {
      match scope.kind {
        ScopeKind::Global => self.global_scope = scope,
        ScopeKind::Class(class) => {
          if !(*class).p_ptr.is_null() {
            self.open(&mut (*(*class).p_ptr).scope);
          }
        }
        _ => {}
      }
      self.scopes.push(scope);
    }
  }

  pub fn close(&mut self) {
    unsafe {
      let scope = self.scopes.pop().unwrap();
      if let ScopeKind::Class(_) = (*scope).kind {
        // all scopes in the stack except the bottom are parent of the class
        for _ in 0..self.scopes.len() - 1 { self.scopes.pop(); }
      }
    }
  }

  pub fn cur_scope(&self) -> &mut Scope {
    unsafe { &mut **self.scopes.last().unwrap() }
  }

  pub fn lookup_class(&self, name: &'static str) -> Option<Symbol> {
    unsafe {
      (*self.global_scope).get(name).map(|class| *class)
    }
  }
}