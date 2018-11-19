use super::ast::*;
use super::loc::*;
use super::errors::*;
use super::config::*;
use super::symbol::*;
use std::default::Default as D;
use std::ptr;

macro_rules! issue {
    ($rec:expr, $loc: expr, $err: expr) => {
        $rec.errors.push(Error::new($loc, $err));
    };
}

pub struct SymbolBuilder {
    errors: Vec<Error>,
    scopes: ScopeStack,
}

unsafe fn calc_order(class_def: *mut ClassDef) -> i32 {
    if class_def.is_null() { return -1; }
    let class_def = &mut *class_def;
    if class_def.order == 0 {
        class_def.order = calc_order(class_def.parent_ref) + 1;
    }
    class_def.order
}

impl SymbolBuilder {
    pub fn new() -> SymbolBuilder {
        SymbolBuilder {
            errors: Vec::new(),
            scopes: ScopeStack {
                global_scope: ptr::null_mut(),
                scopes: Vec::new(),
            },
        }
    }

    pub fn build(mut self, mut program: Program) -> Result<Program, Vec<Error>> {
        self.visit_program(&mut program);
        if self.errors.is_empty() {
            Ok(program)
        } else {
            Err(self.errors)
        }
    }
}

impl SymbolBuilder {
    unsafe fn check_override(&mut self, class_def: &mut ClassDef) {
        if class_def.checked || class_def.parent_ref.is_null() {
            return;
        }
        let parent = &mut *class_def.parent_ref;
        self.check_override(parent);
        let self_scope = &mut class_def.scope;
        self.scopes.open(&mut parent.scope);
        // remove all conflicting fields
        self_scope.retain(|name, symbol| {
            match self.scopes.lookup(name, true) {
                Some((parent_symbol, _)) if !parent_symbol.is_class() =>
                    if (parent_symbol.is_var() && symbol.is_method()) ||
                        (parent_symbol.is_method() && symbol.is_var()) {
                        issue!(self, symbol.get_loc(), ConflictDeclaration { earlier: parent_symbol.get_loc(), name });
                        false
                    } else if parent_symbol.is_method() {
                        // here symbol.is_method() must also be true
                        let parent_symbol = parent_symbol.as_method();
                        let symbol = symbol.as_method();
                        if parent_symbol.static_ || symbol.static_ {
                            issue!(self, symbol.loc, ConflictDeclaration { earlier: parent_symbol.loc, name });
                            false
                        } else if !symbol.return_type.extends(&parent_symbol.return_type) {
                            issue!(self, symbol.loc, BadOverride { method_name: name, parent_name: parent.name });
                            false
                        } else {
                            true
                        }
                    } else if parent_symbol.is_var() {
                        issue!(self, symbol.get_loc(), OverrideVar { name });
                        false
                    } else {
                        true
                    }
                _ => true,
            }
        });
        self.scopes.close();
        class_def.checked = true;
    }

    unsafe fn check_main(&mut self, class_def: *const ClassDef) -> bool {
        if class_def.is_null() {
            return false;
        }
        let class_def = &*class_def;
        match class_def.scope.get(MAIN_METHOD) {
            Some(main) if main.is_method() => {
                let main = main.as_method();
                main.static_ && main.return_type.data == TypeData::Basic("void") && main.parameters.is_empty()
            }
            _ => false,
        }
    }
}

impl Visitor for SymbolBuilder {
    fn visit_program(&mut self, program: &mut Program) {
        unsafe {
            self.scopes.open(&mut program.scope);
            for class_def in &mut program.classes {
                if let Some(earlier) = self.scopes.lookup_class(class_def.name) {
                    issue!(self, class_def.loc, ConflictDeclaration {
                        earlier: earlier.get_loc(),
                        name: class_def.name,
                    });
                } else {
                    self.scopes.declare(Symbol::Class(class_def));
                }
            }
            for class_def in &mut program.classes {
                if let Some(parent) = class_def.parent {
                    if let Some(parent_ref) = self.scopes.lookup_class(parent) {
                        let parent_ref = parent_ref.as_class();
                        class_def.parent_ref = parent_ref;
                        if calc_order(class_def) <= calc_order(parent_ref) {
                            issue!(self, class_def.loc, CyclicInheritance);
                            class_def.parent_ref = ptr::null_mut();
                        } else if parent_ref.sealed {
                            issue!(self, class_def.loc, SealedInheritance);
                            class_def.parent_ref = ptr::null_mut();
                        }
                    } else {
                        issue!(self, class_def.loc, ClassNotFound { name: parent });
                    }
                }
            }
            for class_def in &mut program.classes {
                self.visit_class_def(class_def);
                if class_def.name == MAIN_CLASS {
                    program.main = class_def;
                }
            }
            for class_def in &mut program.classes {
                self.check_override(class_def);
            }
            if !self.check_main(program.main) {
                issue!(self, NO_LOC, NoMainClass);
            }
        }
    }

    fn visit_class_def(&mut self, class_def: &mut ClassDef) {
        class_def.scope = Scope { symbols: D::default(), kind: ScopeKind::Class(class_def) };
        self.scopes.open(&mut class_def.scope);
        for field_def in &mut class_def.fields {
            match field_def {
                FieldDef::MethodDef(method_def) => self.visit_method_def(method_def),
                FieldDef::VarDef(var_def) => self.visit_var_def(var_def),
            };
        }
        self.scopes.close();
    }

    fn visit_method_def(&mut self, method_def: &mut MethodDef) {
        self.visit_type(&mut method_def.return_type);
        if let Some((earlier, _)) = self.scopes.lookup(method_def.name, false) {
            issue!(self, method_def.loc, ConflictDeclaration {
                earlier: earlier.get_loc(),
                name: method_def.name,
            });
        } else {
            self.scopes.declare(Symbol::Method(method_def as *mut _));
        }
        if !method_def.static_ {
            let class = self.scopes.current_scope().get_class();
            method_def.parameters.insert(0, VarDef {
                loc: method_def.loc,
                name: "this",
                type_: Type { loc: method_def.loc, data: TypeData::Class(class.name, class) },
                is_parameter: true,
            });
        }
        method_def.scope = Scope { symbols: D::default(), kind: ScopeKind::Parameter(method_def) };
        self.scopes.open(&mut method_def.scope);
        for var_def in &mut method_def.parameters {
            self.visit_var_def(var_def);
        }
        method_def.body.is_method = true;
        self.visit_block(&mut method_def.body);
        self.scopes.close();
    }

    fn visit_var_def(&mut self, var_def: &mut VarDef) {
        unsafe {
            self.visit_type(&mut var_def.type_);
            if var_def.type_.is_void() {
                issue!(self, var_def.loc, VoidVar { name: var_def.name });
                return;
            }
            if {
                if let Some((symbol, scope)) = self.scopes.lookup(var_def.name, true) {
                    if {
                        let current = self.scopes.current_scope();
                        current as *const _ == scope || ((*scope).is_parameter() && match current.kind {
                            ScopeKind::Local(block) => (*block).is_method,
                            _ => false,
                        })
                    } {
                        issue!(self, var_def.loc, ConflictDeclaration { earlier: symbol.get_loc(),name: var_def.name, });
                        false
                    } else { true }
                } else { true }
            } {
                self.scopes.declare(Symbol::Var(var_def));
                var_def.is_parameter = self.scopes.current_scope().is_parameter();
            }
        }
    }

    fn visit_block(&mut self, block: &mut Block) {
        block.scope = Scope { symbols: D::default(), kind: ScopeKind::Local(block) };
        self.scopes.open(&mut block.scope);
        for statement in &mut block.statements {
            self.visit_statement(statement);
        }
        self.scopes.close();
    }

    fn visit_while(&mut self, while_: &mut While) {
        self.visit_statement(&mut while_.body);
    }

    fn visit_for(&mut self, for_: &mut For) {
        self.visit_statement(&mut for_.body);
    }

    fn visit_if(&mut self, if_: &mut If) {
        self.visit_statement(&mut if_.on_true);
        if let Some(on_false) = &mut if_.on_false {
            self.visit_statement(on_false);
        }
    }

    fn visit_foreach(&mut self, _foreach: &mut Foreach) {
        unimplemented!()
    }

    fn visit_guarded(&mut self, guarded: &mut Guarded) {
        for (_, statement) in &mut guarded.guarded {
            self.visit_statement(statement);
        }
    }

    fn visit_type(&mut self, type_: &mut Type) {
        let mut is_error = false; // work around with borrow check
        match &mut type_.data {
            TypeData::Class(name, ref mut class) => {
                if let Some(class_symbol) = self.scopes.lookup_class(name) {
                    *class = class_symbol.as_class();
                } else {
                    is_error = true;
                    issue!(self, type_.loc, ClassNotFound { name });
                }
            }
            TypeData::Array(elem_type) => {
                if elem_type.is_error() {
                    is_error = true;
                } else if elem_type.is_void() {
                    is_error = true;
                    issue!(self, type_.loc, VoidArrayElement);
                }
            }
            _ => {}
        }
        if is_error {
            type_.data = TypeData::Error;
        }
    }
}