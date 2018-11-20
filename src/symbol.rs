use super::ast::*;
use super::loc::*;
use super::types::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::string::ToString;
use std::default::Default as D;

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

impl ToString for Symbol {
    fn to_string(&self) -> String {
        unsafe {
            match self {
                Symbol::Class(class) => {
                    let class = &**class;
                    let s = format!("{} -> class {}", class.loc, class.name);
                    if (*class).parent_ref.is_null() { s } else { s + " : " + (*class.parent_ref).name }
                }
                Symbol::Method(method) => {
                    let method = &**method;
                    let mut s = method.loc.to_string() + " -> " + if method.static_ { "static " } else { "" } + "function "
                        + method.name + " : ";
                    for parameter in &method.parameters {
                        s += &(parameter.type_.to_string() + "->");
                    }
                    s + &method.return_type.to_string()
                }
                Symbol::Var(var) => {
                    let var = &**var;
                    var.loc.to_string() + " -> variable " + if var.is_parameter { "@" } else { "" } + var.name
                        + " : " + &var.type_.to_string()
                }
            }
        }
    }
}

impl Symbol {
    pub fn is_class(&self) -> bool {
        match self {
            Symbol::Class(_) => true,
            _ => false,
        }
    }

    pub fn is_method(&self) -> bool {
        match self {
            Symbol::Method(_) => true,
            _ => false,
        }
    }

    pub fn is_var(&self) -> bool {
        match self {
            Symbol::Var(_) => true,
            _ => false,
        }
    }

    pub fn as_class(&self) -> &mut ClassDef {
        unsafe {
            match self {
                Symbol::Class(class) => &mut **class,
                _ => panic!("call as_class on non-class symbol"),
            }
        }
    }

    pub fn as_method(&self) -> &mut MethodDef {
        unsafe {
            match self {
                Symbol::Method(method) => &mut **method,
                _ => panic!("call as_method on non-method symbol"),
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

    pub fn get_type(&self) -> SemanticType {
        unsafe {
            match self {
                Symbol::Class(class) => SemanticType::Class((**class).name, *class),
                Symbol::Method(method) => SemanticType::Method(*method),
                Symbol::Var(var) => (**var).type_.sem.clone(),
            }
        }
    }
}

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
                // all scopes in the stack except the bottom are parent of the class
                for _ in 0..self.scopes.len() - 1 { self.scopes.pop(); }
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