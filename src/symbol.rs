use super::ast::*;
use super::loc::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

// ast node owns the scope
pub struct Scope {
    pub symbols: HashMap<&'static str, Symbol>,
    pub kind: ScopeKind,
}

#[derive(Copy, Clone)]
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
        if let ScopeKind::Local(_) = self.kind {
            true
        }
        false
    }
}

// refer to a node in ast
#[derive(Copy, Clone)]
pub enum Symbol {
    Class(*mut ClassDef),
    Method(*mut MethodDef),
    Var(*mut VarDef),
}

impl Symbol {
    pub fn get_name(&self) -> &'static str {
        unsafe {
            match self {
                Symbol::Class(class) => (*class).name,
                Symbol::Method(method) => (*method).name,
                Symbol::Var(var) => (*var).name,
            }
        }
    }
}

pub struct ScopeStack {
    pub global_scope: *mut Scope,
    pub scopes: Vec<*mut Scope>,
}

impl ScopeStack {
    pub fn lookup(&self, name: &'static str, recursive: bool) -> Option<&Symbol> {
        if recursive {
            for scope in &self.scopes.rev() {
                if let Some(symbol) = scope.get(name) {
                    return Some(symbol);
                }
            }
            None
        } else {
            self.scopes.last().unwrap().get(name)
        }
    }

    pub fn lookup_before(&self, name: &'static str, loc: Loc) -> Option<&Symbol> {
        for scope in &self.scopes.rev() {
            if let Some(symbol) = scope.get(name) {
                if scope.is_local() && symbol.loc > loc {
                    continue;
                }
                return Some(symbol);
            }
        }
        None
    }

    pub fn declare(&mut self, symbol: Symbol) {
        self.scopes.last().unwrap().insert(symbol.get_name(), symbol);
    }

    pub fn open(&mut self, scope: &mut Scope) {
        unsafe {
            match scope.kind {
                ScopeKind::Global(global) => self.global_scope = scope,
                ScopeKind::Class(class) => {
                    if !class.parent_ref.is_null() {
                        self.open((*class.parent_ref).scope);
                    }
                }
                _ => {}
            }
            self.scopes.push(scope);
        }
    }

    pub fn close(&mut self) {
        let scope = self.scopes.pop().unwrap();
        if let ScopeKind::Class(_) = scope {
            // all scopes in the stack are parent of the class
            self.scopes.clear();
        }
    }

    pub fn current_scope(&mut self) -> &mut Scope {
        unsafe { &mut **self.scopes.last().unwrap() }
    }

    pub fn lookup_class(&self, name: &'static str) -> Option<&Symbol> {
        self.global_scope.get(name)
    }
}