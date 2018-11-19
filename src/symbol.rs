use super::ast::*;
use super::loc::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

// ast node owns the scope
#[derive(Debug)]
pub struct Scope {
    pub symbols: HashMap<&'static str, Symbol>,
    pub kind: ScopeKind,
}

#[derive(Debug, Copy, Clone)]
pub enum ScopeKind {
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
    pub fn is_local(&self) -> bool {
        match self.kind {
            ScopeKind::Local(_) => true,
            _ => false,
        }
    }
}

// refer to a node in ast
#[derive(Debug, Copy, Clone)]
pub enum Symbol {
    Class(*mut ClassDef),
    Method(*mut MethodDef),
    Var(*mut VarDef),
}

impl Symbol {
    pub fn as_class(&self) -> &mut ClassDef {
        unsafe {
            match self {
                Symbol::Class(class) => &mut **class,
                _ => panic!("call as_class on non-class symbol"),
            }
        }
    }

    pub fn get_name(&self) -> &'static str {
        unsafe {
            match self {
                Symbol::Class(class) => (**class).name,
                Symbol::Method(method) => (**method).name,
                Symbol::Var(var) => (**var).name,
            }
        }
    }

    pub fn get_loc(&self) -> Loc {
        unsafe {
            match self {
                Symbol::Class(class) => (**class).loc,
                Symbol::Method(method) => (**method).loc,
                Symbol::Var(var) => (**var).loc,
            }
        }
    }
}

pub struct ScopeStack {
    pub global_scope: *mut Scope,
    pub scopes: Vec<*mut Scope>,
}

impl ScopeStack {
    pub fn lookup(&self, name: &'static str, recursive: bool) -> Option<Symbol> {
        unsafe {
            if recursive {
                for scope in self.scopes.iter().rev() {
                    if let Some(symbol) = (**scope).get(name) {
                        return Some(*symbol);
                    }
                }
                None
            } else {
                (**self.scopes.last().unwrap()).get(name).map(|symbol| *symbol)
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
                    if !(*class).parent_ref.is_null() {
                        self.open(&mut (*(*class).parent_ref).scope);
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
                // all scopes in the stack are parent of the class
                self.scopes.clear();
            }
        }
    }

    pub fn current_scope(&self) -> &mut Scope {
        unsafe { &mut **self.scopes.last().unwrap() }
    }

    pub fn lookup_class(&self, name: &'static str) -> Option<Symbol> {
        unsafe {
            (*self.global_scope).get(name).map(|class| *class)
        }
    }
}