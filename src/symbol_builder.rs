use super::ast::*;
use super::types::*;
use super::loc::*;
use super::errors::*;
use super::config::*;
use super::symbol::*;
use super::util::*;
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

fn calc_order(class_def: *mut ClassDef) -> i32 {
  if class_def.is_null() { -1 } else {
    let class_def = class_def.get();
    if class_def.order < 0 {
      class_def.order = 0;
      class_def.order = calc_order(class_def.p_ptr) + 1;
    }
    class_def.order
  }
}

impl SymbolBuilder {
  pub fn build(mut program: Program) -> Result<Program, Vec<Error>> {
    let mut builder = SymbolBuilder {
      errors: Vec::new(),
      scopes: ScopeStack {
        global_scope: ptr::null_mut(),
        scopes: Vec::new(),
      },
    };
    builder.program(&mut program);
    if builder.errors.is_empty() {
      Ok(program)
    } else {
      builder.errors.sort_by_key(|x| x.loc);
      Err(builder.errors)
    }
  }
}

impl SymbolBuilder {
  fn check_override(&mut self, class_def: &mut ClassDef) {
    if class_def.checked || class_def.p_ptr.is_null() { return; }
    let parent = class_def.p_ptr.get();
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
            } else if !symbol.ret_t.assignable_to(&parent_symbol.ret_t)
              || symbol.param.len() != parent_symbol.param.len()
              || {
              let mut unfit = false;
              // start from 1, skip this
              for i in 1..symbol.param.len() {
                if !parent_symbol.param[i].type_.assignable_to(&symbol.param[i].type_) {
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
  fn check_var_declaration(&mut self, name: &'static str, loc: Loc) -> bool {
    if let Some((symbol, scope)) = self.scopes.lookup(name, true) {
      if {
        let cur = self.scopes.cur_scope();
        cur as *const _ == scope || (scope.get().is_parameter() && match cur.kind {
          ScopeKind::Local(block) => block.get().is_method,
          _ => false,
        })
      } {
        issue!(self, loc, ConflictDeclaration { earlier: symbol.get_loc(), name, });
        false
      } else { true }
    } else { true }
  }

  fn check_main(&mut self, class_def: *const ClassDef) -> bool {
    if class_def.is_null() { return false; }
    let class_def = class_def.get();
    match class_def.scope.get(MAIN_METHOD) {
      Some(main) if main.is_method() => {
        let main = main.as_method();
        main.static_ && main.ret_t.sem == VOID && main.param.is_empty()
      }
      _ => false,
    }
  }
}

impl SemanticTypeVisitor for SymbolBuilder {
  fn push_error(&mut self, error: Error) {
    self.errors.push(error)
  }

  fn lookup_class(&self, name: &'static str) -> Option<Symbol> {
    self.scopes.lookup_class(name)
  }
}

impl SymbolBuilder {
  fn program(&mut self, program: &mut Program) {
    self.scopes.open(&mut program.scope);
    for class_def in &mut program.class {
      if let Some(earlier) = self.scopes.lookup_class(class_def.name) {
        issue!(self, class_def.loc, ConflictDeclaration { earlier: earlier.get_loc(), name: class_def.name });
      } else {
        self.scopes.declare(Symbol::Class(class_def));
      }
    }

    for class_def in &mut program.class {
      if let Some(parent) = class_def.parent {
        if let Some(parent_ref) = self.scopes.lookup_class(parent) {
          let parent_ref = parent_ref.as_class();
          class_def.p_ptr = parent_ref;
          if calc_order(class_def) <= calc_order(parent_ref) {
            issue!(self, class_def.loc, CyclicInheritance{});
            class_def.p_ptr = ptr::null_mut();
          } else if parent_ref.sealed {
            issue!(self, class_def.loc, SealedInheritance{});
            class_def.p_ptr = ptr::null_mut();
          }
        } else {
          issue!(self, class_def.loc, NoSuchClass { name: parent });
        }
      }
    }

    for class_def in &mut program.class {
      class_def.scope = Scope { symbols: D::default(), kind: ScopeKind::Class(class_def) };
    }

    for class_def in &mut program.class {
      self.class_def(class_def);
      if class_def.name == MAIN_CLASS {
        program.main = class_def;
      }
    }

    for class_def in &mut program.class { self.check_override(class_def); }

    if !self.check_main(program.main) { issue!(self, NO_LOC, NoMainClass{}); }
  }

  fn class_def(&mut self, class_def: &mut ClassDef) {
    self.scopes.open(&mut class_def.scope);
    for field_def in &mut class_def.field {
      match field_def {
        FieldDef::MethodDef(method_def) => self.method_def(method_def),
        FieldDef::VarDef(var_def) => self.var_def(var_def),
      };
    }
    self.scopes.close();
  }

  fn method_def(&mut self, method_def: &mut MethodDef) {
    self.type_(&mut method_def.ret_t);
    if let Some((earlier, _)) = self.scopes.lookup(method_def.name, false) {
      issue!(self, method_def.loc, ConflictDeclaration { earlier: earlier.get_loc(), name: method_def.name });
    } else {
      self.scopes.declare(Symbol::Method(method_def as *mut _));
    }
    if !method_def.static_ {
      let class = self.scopes.cur_scope().get_class();
      method_def.param.insert(0, VarDef {
        loc: method_def.loc,
        name: "this",
        type_: Type { loc: method_def.loc, sem: SemanticType::Object(class) },
        src: None,
        finish_loc: method_def.loc,
        scope: &method_def.scope,
        jvm_index: 0, // 'this' is at 0
        offset: -1,
        llvm_val: ptr::null_mut(),
      });
    }
    method_def.scope = Scope { symbols: D::default(), kind: ScopeKind::Parameter(method_def) };
    self.scopes.open(&mut method_def.scope);
    for var_def in &mut method_def.param {
      self.var_def(var_def);
    }
    method_def.body.is_method = true;
    self.block(&mut method_def.body);
    self.scopes.close();
  }

  fn stmt(&mut self, stmt: &mut Stmt) {
    match stmt {
      Stmt::Simple(simple) => if let Simple::VarDef(var_def) = simple { self.var_def(var_def); }
      Stmt::If(if_) => {
        self.block(&mut if_.on_true);
        if let Some(on_false) = &mut if_.on_false { self.block(on_false); }
      }
      Stmt::While(while_) => self.block(&mut while_.body),
      Stmt::For(for_) => {
        let block = &mut for_.body;
        block.scope = Scope { symbols: D::default(), kind: ScopeKind::Local(block) };
        self.scopes.open(&mut block.scope);
        if let Simple::VarDef(var_def) = &mut for_.init { self.var_def(var_def); }
        for stmt in &mut block.stmt { self.stmt(stmt); }
        self.scopes.close();
      }
      Stmt::Foreach(foreach) => {
        foreach.body.scope.kind = ScopeKind::Local(&mut foreach.body);
        self.scopes.open(&mut foreach.body.scope);
        // reuse the code of var def, which can handle var correctly
        self.var_def(&mut foreach.def);
        for stmt in &mut foreach.body.stmt { self.stmt(stmt); }
        self.scopes.close();
      }
      Stmt::Guarded(guarded) => for (_, stmt) in &mut guarded.guarded { self.block(stmt); },
      Stmt::Block(block) => self.block(block),
      _ => {}
    };
  }

  fn var_def(&mut self, var_def: &mut VarDef) {
    self.type_(&mut var_def.type_);
    if var_def.type_.sem == VOID {
      issue!(self, var_def.loc, VoidVar { name: var_def.name });
      return;
    }
    if self.check_var_declaration(var_def.name, var_def.loc) {
      var_def.scope = self.scopes.cur_scope() as *const _;
      self.scopes.declare(Symbol::Var(var_def));
    }
  }

  fn block(&mut self, block: &mut Block) {
    block.scope = Scope { symbols: D::default(), kind: ScopeKind::Local(block) };
    self.scopes.open(&mut block.scope);
    for stmt in &mut block.stmt { self.stmt(stmt); }
    self.scopes.close();
  }

  fn type_(&mut self, type_: &mut Type) {
    self.semantic_type(&mut type_.sem, type_.loc);
  }
}