use super::ast::*;
use super::types::*;
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
  if class_def.order < 0 {
    class_def.order = 0;
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
      self.errors.sort_by_key(|x| x.loc);
      Err(self.errors)
    }
  }

  fn visit_semantic_type(&mut self, type_: &mut SemanticType, loc: Loc) {
    if match type_ { // work around with borrow check
      SemanticType::Object(name, ref mut class) =>
        if let Some(class_symbol) = self.scopes.lookup_class(name) {
          *class = class_symbol.as_class();
          false
        } else {
          issue!(self, loc, ClassNotFound { name });
          true
        }
      SemanticType::Array(elem_type) => {
        self.visit_semantic_type(elem_type, loc);
        if elem_type.as_ref() == &ERROR {
          true
        } else if elem_type.as_ref() == &VOID {
          issue!(self, loc, VoidArrayElement);
          true
        } else { false }
      }
      _ => false,
    } { *type_ = ERROR; }
  }
}

impl SymbolBuilder {
  unsafe fn check_override(&mut self, class_def: &mut ClassDef) {
    if class_def.checked || class_def.parent_ref.is_null() { return; }
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
            } else if !symbol.ret_t.extends(&parent_symbol.ret_t)
              || symbol.params.len() != parent_symbol.params.len()
              || {
              let mut unfit = false;
              // start from 1, skip this
              for i in 1..symbol.params.len() {
                if !parent_symbol.params[i].type_.extends(&symbol.params[i].type_) {
                  unfit = true;
                  break;
                }
              }
              unfit
            } {
              issue!(self, symbol.loc, BadOverride { method: name, parent: parent.name });
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

  // check whether a declaration is valid
  // if valid, return true, but it is not declared yet
  unsafe fn check_var_declaration(&mut self, name: &'static str, loc: Loc) -> bool {
    if let Some((symbol, scope)) = self.scopes.lookup(name, true) {
      if {
        let cur = self.scopes.cur_scope();
        cur as *const _ == scope || ((*scope).is_parameter() && match cur.kind {
          ScopeKind::Local(block) => (*block).is_method,
          _ => false,
        })
      } {
        issue!(self, loc, ConflictDeclaration { earlier: symbol.get_loc(), name, });
        false
      } else { true }
    } else { true }
  }

  unsafe fn check_main(&mut self, class_def: *const ClassDef) -> bool {
    if class_def.is_null() { return false; }
    let class_def = &*class_def;
    match class_def.scope.get(MAIN_METHOD) {
      Some(main) if main.is_method() => {
        let main = main.as_method();
        main.static_ && main.ret_t.sem == VOID && main.params.is_empty()
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
        class_def.scope = Scope { symbols: D::default(), kind: ScopeKind::Class(class_def) };
      }

      for class_def in &mut program.classes {
        self.visit_class_def(class_def);
        if class_def.name == MAIN_CLASS {
          program.main = class_def;
        }
      }

      for class_def in &mut program.classes { self.check_override(class_def); }

      if !self.check_main(program.main) { issue!(self, NO_LOC, NoMainClass); }
    }
  }

  fn visit_class_def(&mut self, class_def: &mut ClassDef) {
    self.scopes.open(&mut class_def.scope);
    for field_def in &mut class_def.fields { self.visit_field_def(field_def) }
    self.scopes.close();
  }

  fn visit_method_def(&mut self, method_def: &mut MethodDef) {
    self.visit_type(&mut method_def.ret_t);
    if let Some((earlier, _)) = self.scopes.lookup(method_def.name, false) {
      issue!(self, method_def.loc, ConflictDeclaration {
                earlier: earlier.get_loc(),
                name: method_def.name,
            });
    } else {
      self.scopes.declare(Symbol::Method(method_def as *mut _));
    }
    if !method_def.static_ {
      let class = self.scopes.cur_scope().get_class();
      method_def.params.insert(0, VarDef {
        loc: method_def.loc,
        name: "this",
        type_: Type { loc: method_def.loc, sem: SemanticType::Object(class.name, class) },
        scope: &method_def.scope,
      });
    }
    method_def.scope = Scope { symbols: D::default(), kind: ScopeKind::Parameter(method_def) };
    self.scopes.open(&mut method_def.scope);
    for var_def in &mut method_def.params {
      self.visit_var_def(var_def);
    }
    method_def.body.is_method = true;
    self.visit_block(&mut method_def.body);
    self.scopes.close();
  }

  fn visit_var_def(&mut self, var_def: &mut VarDef) {
    unsafe {
      self.visit_type(&mut var_def.type_);
      if var_def.type_.sem == VOID {
        issue!(self, var_def.loc, VoidVar { name: var_def.name });
        return;
      }
      if self.check_var_declaration(var_def.name, var_def.loc) {
        var_def.scope = self.scopes.cur_scope() as *const _;
        self.scopes.declare(Symbol::Var(Var::VarDef(var_def)));
      }
    }
  }

  fn visit_var_assign(&mut self, var_assign: &mut VarAssign) {
    unsafe {
      if self.check_var_declaration(var_assign.name, var_assign.loc) {
        var_assign.scope = self.scopes.cur_scope() as *const _;
        self.scopes.declare(Symbol::Var(Var::VarAssign(var_assign)));
      }
    }
  }

  fn visit_block(&mut self, block: &mut Block) {
    block.scope = Scope { symbols: D::default(), kind: ScopeKind::Local(block) };
    self.scopes.open(&mut block.scope);
    for stmt in &mut block.stmts { self.visit_stmt(stmt); }
    self.scopes.close();
  }

  fn visit_while(&mut self, while_: &mut While) {
    self.visit_block(&mut while_.body);
  }

  fn visit_for(&mut self, for_: &mut For) {
    let block = &mut for_.body;
    block.scope = Scope { symbols: D::default(), kind: ScopeKind::Local(block) };
    self.scopes.open(&mut block.scope);
    if let Simple::VarAssign(var_assign) = &mut for_.init { self.visit_var_assign(var_assign); }
    for stmt in &mut block.stmts { self.visit_stmt(stmt); }
    self.scopes.close();
  }

  fn visit_if(&mut self, if_: &mut If) {
    self.visit_block(&mut if_.on_true);
    if let Some(on_false) = &mut if_.on_false { self.visit_block(on_false); }
  }

  fn visit_foreach(&mut self, _foreach: &mut Foreach) {
    unimplemented!()
  }

  fn visit_guarded(&mut self, guarded: &mut Guarded) {
    for (_, stmt) in &mut guarded.guarded { self.visit_block(stmt); }
  }

  fn visit_type(&mut self, type_: &mut Type) {
    self.visit_semantic_type(&mut type_.sem, type_.loc);
  }
}