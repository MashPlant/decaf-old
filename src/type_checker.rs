use super::ast::*;
use super::types::*;
use super::errors::*;
use super::symbol::*;
use super::util::*;
use super::loc::*;

use std::ptr;

pub struct TypeChecker {
  errors: Vec<Error>,
  scopes: ScopeStack,
  loop_cnt: i32,
  cur_method: *const MethodDef,
  cur_class: *const ClassDef,
  cur_id_used_for_ref: bool,
}

impl TypeChecker {
  pub fn check(mut program: Program) -> Result<Program, Vec<Error>> {
    let mut checker = TypeChecker {
      errors: Vec::new(),
      scopes: ScopeStack {
        global_scope: ptr::null_mut(),
        scopes: Vec::new(),
      },
      loop_cnt: 0,
      cur_method: ptr::null(),
      cur_class: ptr::null(),
      cur_id_used_for_ref: false,
    };
    checker.scopes.open(&mut program.scope);
    for class_def in &mut program.class { checker.class_def(class_def); }
    checker.scopes.close();
    if checker.errors.is_empty() {
      Ok(program)
    } else {
      checker.errors.sort_by_key(|x| x.loc);
      Err(checker.errors)
    }
  }

  fn issue<E: IError + 'static>(&mut self, loc: Loc, error: E) {
    self.errors.push(Error::new(loc, error));
  }

  fn check_bool(&mut self, expr: &mut Expr) {
    self.expr(expr);
    if !expr.type_.error_or(&BOOL) {
      self.issue(expr.loc, TestNotBool {});
    }
  }

  fn check_call(&mut self, call: &mut Call, symbol: Option<Symbol>, expr_loc: Loc, expr_type: &mut SemanticType) {
    let owner_t = match &call.owner {
      Some(owner) => owner.type_.clone(),
      None => self.cur_class.get().get_object_type(),
    };
    match symbol {
      Some(symbol) => {
        match symbol {
          Symbol::Method(method) => {
            let method = method.get();
            call.method = method;
            *expr_type = method.ret_t.sem.clone();
            match &call.owner {
              Some(_) => {
                if owner_t.is_class() && !method.static_ {
                  // call a instance method through class reference
                  self.issue(expr_loc, BadFieldAccess { name: call.name, owner_t: owner_t.to_string() });
                }
                if method.static_ { call.owner = None; }
              }
              None => {
                match (self.cur_method.get().static_, method.static_) {
                  (true, false) => {
                    let cur_method = self.cur_method.get().name;
                    self.issue(expr_loc, RefInStatic { field: method.name, method: cur_method });
                  }
                  (false, false) => call.owner = Some(Box::new(Expr::with_type(expr_loc, self.cur_class.get().get_object_type(), ExprData::This))),
                  _ => {}
                }
              }
            };
            for expr in &mut call.arg { self.expr(expr); }
            let this_offset = if method.static_ { 0 } else { 1 };
            let argc = call.arg.len();
            if argc != method.param.len() - this_offset {
              self.issue(expr_loc, WrongArgc { name: call.name, expect: method.param.len() as i32, actual: argc as i32 });
            } else {
              for i in this_offset..argc + this_offset {
                let arg_t = &call.arg[i - this_offset].type_;
                if !arg_t.assignable_to(&method.param[i].type_.sem) {
                  self.issue(call.arg[i - this_offset].loc, WrongArgType {
                    loc: (i + 1 - this_offset) as i32,
                    arg_t: arg_t.to_string(),
                    param_t: method.param[i].type_.sem.to_string(),
                  });
                }
              }
            }
          }
          _ => self.issue(expr_loc, NotMethod { name: call.name, owner_t: owner_t.to_string() }),
        }
      }
      None => self.issue(expr_loc, NoSuchField { name: call.name, owner_t: owner_t.to_string() }),
    };
  }
}

impl SemanticTypeVisitor for TypeChecker {
  fn push_error(&mut self, error: Error) {
    self.errors.push(error)
  }

  fn lookup_class(&self, name: &'static str) -> Option<Symbol> {
    self.scopes.lookup_class(name)
  }
}

impl TypeChecker {
  fn class_def(&mut self, class_def: &mut ClassDef) {
    self.cur_class = class_def;
    self.scopes.open(&mut class_def.scope);
    for field_def in &mut class_def.field {
      if let FieldDef::MethodDef(method_def) = field_def {
        self.cur_method = method_def;
        method_def.class = self.cur_class;
        self.scopes.open(&mut method_def.scope);
        self.block(&mut method_def.body);
        self.scopes.close();
      };
    }
    self.scopes.close();
  }

  fn stmt(&mut self, stmt: &mut Stmt) {
    use super::ast::Stmt::*;
    match stmt {
      Simple(simple) => self.simple(simple),
      If(if_) => {
        self.check_bool(&mut if_.cond);
        self.block(&mut if_.on_true);
        if let Some(on_false) = &mut if_.on_false { self.block(on_false); }
      }
      While(while_) => {
        self.check_bool(&mut while_.cond);
        self.loop_cnt += 1;
        self.block(&mut while_.body);
        self.loop_cnt -= 1;
      }
      For(for_) => {
        self.scopes.open(&mut for_.body.scope);
        self.simple(&mut for_.init);
        self.check_bool(&mut for_.cond);
        self.simple(&mut for_.update);
        for stmt in &mut for_.body.stmt { self.stmt(stmt); }
        self.scopes.close();
      }
      Return(return_) => {
        let expect = &self.cur_method.get().ret_t.sem;
        match &mut return_.expr {
          Some(expr) => {
            self.expr(expr);
            if !expr.type_.assignable_to(expect) {
              self.issue(return_.loc, WrongReturnType { ret_t: expr.type_.to_string(), expect_t: expect.to_string() });
            }
          }
          None => {
            if expect != &VOID {
              self.issue(return_.loc, WrongReturnType { ret_t: "void".to_owned(), expect_t: expect.to_string() });
            }
          }
        }
      }
      Print(print) => for (i, expr) in print.print.iter_mut().enumerate() {
        self.expr(expr);
        if expr.type_ != ERROR && expr.type_ != BOOL && expr.type_ != INT && expr.type_ != STRING {
          self.issue(expr.loc, BadPrintArg { loc: i as i32 + 1, type_: expr.type_.to_string() });
        }
      },
      Break(break_) => if self.loop_cnt == 0 { self.issue(break_.loc, BreakOutOfLoop {}); },
      SCopy(s_copy) => self.s_copy(s_copy),
      Foreach(foreach) => self.foreach(foreach),
      Guarded(guarded) => for (e, b) in &mut guarded.guarded {
        self.check_bool(e);
        self.block(b);
      },
      Block(block) => self.block(block),
    };
  }

  fn simple(&mut self, simple: &mut Simple) {
    match simple {
      Simple::Assign(assign) => {
        let Assign { dst, src, loc: _ } = assign;
        self.expr(dst);
        self.expr(src);
        // error check is contained in extends
        if dst.type_.is_method() || !src.type_.assignable_to(&dst.type_) {
          self.issue(assign.loc, IncompatibleBinary { l_t: dst.type_.to_string(), op: "=", r_t: src.type_.to_string() })
        }
      }
      Simple::VarDef(var_def) => if let Some(src) = &mut var_def.src {
        self.expr(src);
        if var_def.type_.sem == VAR {
          var_def.type_.sem = src.type_.clone();
        } else if !src.type_.assignable_to(&var_def.type_) {
          self.issue(var_def.loc, IncompatibleBinary { l_t: var_def.type_.sem.to_string(), op: "=", r_t: src.type_.to_string() })
        }
      }
      Simple::Expr(expr) => self.expr(expr),
      _ => {}
    }
  }

  fn expr(&mut self, expr: &mut Expr) {
    use self::ExprData::*;
    match &mut expr.data {
      Id(id) => self.id(id, expr.loc, &mut expr.type_),
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
        match &indexed.arr.type_ {
          SemanticType::Array(elem) => expr.type_ = *elem.clone(),
          SemanticType::Error => {}
          _ => self.issue(indexed.arr.loc, NotArray {}),
        }

        if !indexed.idx.type_.error_or(&INT) {
          self.issue(expr.loc, ArrayIndexNotInt {});
        }
      }
      Call(call) => self.call(call, expr.loc, &mut expr.type_),
      Unary(unary) => self.unary(unary, expr.loc, &mut expr.type_),
      Binary(binary) => self.binary(binary, expr.loc, &mut expr.type_),
      This => if self.cur_method.get().static_ {
        self.issue(expr.loc, ThisInStatic {});
      } else {
        expr.type_ = self.cur_class.get().get_object_type();
      }
      NewClass { name } => match self.scopes.lookup_class(name) {
        Some(class) => expr.type_ = class.get_type(),
        None => self.issue(expr.loc, NoSuchClass { name }),
      },
      NewArray { elem_t, len } => {
        expr.type_ = SemanticType::Array(Box::new(elem_t.sem.clone()));
        self.semantic_type(&mut expr.type_, elem_t.loc);
        self.expr(len);
        if !len.type_.error_or(&INT) { self.issue(len.loc, BadNewArrayLen {}); }
      }
      TypeTest { expr: src, name } => {
        self.expr(src);
        if src.type_ != ERROR && !src.type_.is_object() {
          self.issue(expr.loc, NotObject { type_: src.type_.to_string() });
        }
        if self.scopes.lookup_class(name).is_none() {
          self.issue(expr.loc, NoSuchClass { name });
        }
      }
      TypeCast { name, expr: src } => {
        self.expr(src);
        if src.type_ != ERROR && !src.type_.is_object() {
          self.issue(expr.loc, NotObject { type_: src.type_.to_string() });
        }
        // doesn't need to set type to error because it originally was
        match self.scopes.lookup_class(name) {
          Some(class) => src.type_ = class.get_type(),
          None => self.issue(expr.loc, NoSuchClass { name }),
        }
      }
      Default(default) => self.default(default, expr.loc, &mut expr.type_),
      _ => {}
    };
  }

  fn block(&mut self, block: &mut Block) {
    self.scopes.open(&mut block.scope);
    for stmt in &mut block.stmt { self.stmt(stmt); }
    self.scopes.close();
  }

  fn s_copy(&mut self, s_copy: &mut SCopy) {
    self.expr(&mut s_copy.src);
    let src_t = &s_copy.src.type_;
    match self.scopes.lookup_before(s_copy.dst, s_copy.loc) {
      Some(symbol) => {
        let dst_t = symbol.get_type();
        if &dst_t != &ERROR && !dst_t.is_object() {
          self.issue(s_copy.dst_loc, SCopyNotClass { which: "dst", type_: dst_t.to_string() });
        };
        if !dst_t.is_object() {
          if src_t != &ERROR && !src_t.is_object() {
            self.issue(s_copy.src.loc, SCopyNotClass { which: "src", type_: src_t.to_string() });
          };
        } else if !src_t.error_or(&dst_t) {
          self.issue(s_copy.loc, SCopyMismatch { dst_t: dst_t.to_string(), src_t: src_t.to_string() });
        }
        if let Symbol::Var(var) = symbol { s_copy.dst_sym = var; }
      }
      None => {
        self.issue(s_copy.dst_loc, UndeclaredVar { name: s_copy.dst });
        if src_t != &ERROR && !src_t.is_object() {
          self.issue(s_copy.src.loc, SCopyNotClass { which: "src", type_: src_t.to_string() });
        };
      }
    }
  }

  fn foreach(&mut self, foreach: &mut Foreach) {
    // arr is visited before scope open, cond is after
    self.expr(&mut foreach.arr);
    match &foreach.arr.type_ {
      SemanticType::Array(elem) => match foreach.def.type_.sem {
        ref mut t if t == &VAR => {
          *t = *elem.clone();
          foreach.body.scope.kind = ScopeKind::Local(&mut foreach.body);
          foreach.def.scope = &foreach.body.scope;
        }
        ref t if !elem.assignable_to(t) => self.issue(foreach.def.loc, ForeachMismatch { elem_t: elem.to_string(), def_t: t.to_string() }),
        _ => {}
      }
      SemanticType::Error => if &foreach.def.type_.sem == &VAR {
        // if declared type is not 'var', retain it; otherwise set it to error
        foreach.def.type_.sem = ERROR;
      }
      _ => {
        self.issue(foreach.arr.loc, BadArrayOp {});
        if &foreach.def.type_.sem == &VAR {
          foreach.def.type_.sem = ERROR;
        }
      }
    };
    // first open the scope, where the loop variable is defined
    self.loop_cnt += 1;
    self.scopes.open(&mut foreach.body.scope);
    if let Some(cond) = &mut foreach.cond { self.check_bool(cond); }
    for stmt in &mut foreach.body.stmt { self.stmt(stmt); }
    self.scopes.close();
    self.loop_cnt -= 1;
  }

  fn unary(&mut self, unary: &mut Unary, expr_loc: Loc, expr_type: &mut SemanticType) {
    use super::ast::Operator::*;
    self.expr(&mut unary.r);
    let r = &unary.r.type_;
    match unary.op {
      Neg => {
        if !r.error_or(&INT) {
          self.issue(expr_loc, IncompatibleUnary { op: "-", r_t: r.to_string() });
        }
        *expr_type = INT;
      }
      Not => {
        if !r.error_or(&BOOL) {
          self.issue(expr_loc, IncompatibleUnary { op: "!", r_t: r.to_string() });
        }
        *expr_type = BOOL;
      }
      PreInc | PreDec | PostInc | PostDec => {
        let op = unary.op.to_str();
        if !unary.r.is_lvalue() {
          self.issue(expr_loc, NotLValue { op });
        }
        if !r.error_or(&INT) {
          self.issue(expr_loc, IncompatibleUnary { op, r_t: r.to_string() });
        }
        *expr_type = INT;
      }
      _ => unreachable!(),
    }
  }

  fn binary(&mut self, binary: &mut Binary, expr_loc: Loc, expr_type: &mut SemanticType) {
    use super::ast::Operator::*;
    self.expr(&mut binary.l);
    self.expr(&mut binary.r);
    let (l, r) = (&mut binary.l, &mut binary.r);
    let (l_t, r_t) = (&l.type_, &r.type_);
    match binary.op {
      Repeat => {
        if !r_t.error_or(&INT) { self.issue(r.loc, ArrayRepeatNotInt {}); }
        // l_t cannot be void here
        if l_t != &ERROR {
          *expr_type = SemanticType::Array(Box::new(l_t.clone()));
        }
      }
      Concat => {
        if l_t != &ERROR && !l_t.is_array() { self.issue(l.loc, BadArrayOp {}); }
        if r_t != &ERROR && !r_t.is_array() { self.issue(r.loc, BadArrayOp {}); }
        if l_t.is_array() && r_t.is_array() {
          if l_t != r_t {
            self.issue(expr_loc, ConcatMismatch { l_t: l_t.to_string(), r_t: r_t.to_string() });
          }
          *expr_type = l_t.clone();
        }
      }
      _ => {
        if l_t == &ERROR || r_t == &ERROR {
          return *expr_type = match binary.op {
            Add | Sub | Mul | Div | Mod | BAnd | BOr | BXor | Shl | Shr => l_t.clone(),
            _ => BOOL,
          };
        }
        if !match binary.op {
          Add | Sub | Mul | Div | Mod | BAnd | BOr | BXor | Shl | Shr => {
            *expr_type = l_t.clone();
            l_t == &INT && r_t == &INT
          }
          Lt | Le | Gt | Ge => {
            *expr_type = BOOL;
            l_t == &INT && r_t == &INT
          }
          Eq | Ne => {
            *expr_type = BOOL;
            l_t.assignable_to(r_t) || r_t.assignable_to(l_t)
          }
          And | Or => {
            *expr_type = BOOL;
            l_t == &BOOL && r_t == &BOOL
          }
          _ => unreachable!(),
        } {
          self.issue(expr_loc, IncompatibleBinary {
            l_t: l_t.to_string(),
            op: binary.op.to_str(),
            r_t: r_t.to_string(),
          });
        }
      }
    }
  }

  fn call(&mut self, call: &mut Call, expr_loc: Loc, expr_type: &mut SemanticType) {
    let call_ptr = call as *mut Call;
    match { &mut call_ptr.get().owner } {
      Some(owner) => {
        self.cur_id_used_for_ref = true;
        self.expr(owner);
        let owner_t = &owner.type_;
        if owner_t == &ERROR { return; }
        // check array length call, quite a dirty implementation
        if call.name == "length" {
          if owner_t.is_array() {
            if !call.arg.is_empty() {
              self.issue(expr_loc, LengthWithArgument { count: call.arg.len() as i32 });
            }
            *expr_type = INT;
            call.is_arr_len = true;
            return;
          } else if !owner_t.is_object() && !owner_t.is_class() {
            self.issue(expr_loc, BadLength {});
            return;
          }
        }
        if !owner_t.is_object() && !owner_t.is_class() {
          self.issue(expr_loc, BadFieldAccess { name: call.name, owner_t: owner_t.to_string() });
        } else {
          let symbol = owner_t.get_class().lookup(call.name);
          self.check_call(call, symbol, expr_loc, expr_type);
        }
      }
      None => {
        let symbol = self.cur_class.get().lookup(call.name);
        self.check_call(call, symbol, expr_loc, expr_type);
      }
    }
  }

  fn id(&mut self, id: &mut Id, expr_loc: Loc, expr_type: &mut SemanticType) {
    // not found(no owner) or sole ClassName => UndeclaredVar
    // refer to field in static function => RefInStatic
    // <not object>.a (Main.a, 1.a, func.a) => BadFieldAssess
    // access a field that doesn't belong to self & parent => PrivateFieldAccess
    // given owner but not found object.a => NoSuchField

    // actually a ClassName in the looking-up process is bound to occur an error(UndeclaredVar/BadFieldAssess)

    let owner_ptr = &mut id.owner as *mut Option<Box<Expr>>; // workaround with borrow check
    match &mut id.owner {
      Some(owner) => {
        self.cur_id_used_for_ref = true;
        self.expr(owner);
        let owner_t = &owner.type_;
        match owner_t {
          SemanticType::Object(class) => {
            let class = class.get();
            // lookup through inheritance chain
            match class.lookup(id.name) {
              Some(symbol) => {
                match symbol {
                  Symbol::Var(var_def) => {
                    *expr_type = var_def.get().type_.clone();
                    id.symbol = var_def;
                    if !self.cur_class.get().extends(class) {
                      self.issue(expr_loc, PrivateFieldAccess { name: id.name, owner_t: owner_t.to_string() });
                    }
                  }
                  _ => *expr_type = symbol.get_type(),
                }
              }
              None => self.issue(expr_loc, NoSuchField { name: id.name, owner_t: owner_t.to_string() }),
            }
          }
          SemanticType::Error => {}
          _ => self.issue(expr_loc, BadFieldAccess { name: id.name, owner_t: owner_t.to_string() }),
        }
      }
      None => {
        match self.scopes.lookup_before(id.name, expr_loc) {
          Some(symbol) => {
            match symbol {
              Symbol::Class(class) => {
                if !self.cur_id_used_for_ref {
                  self.issue(expr_loc, UndeclaredVar { name: id.name });
                } else { *expr_type = SemanticType::Class(class); }
              }
              Symbol::Method(method) => *expr_type = SemanticType::Method(method),
              Symbol::Var(var) => {
                let var = var.get();
                *expr_type = var.type_.clone();
                id.symbol = var;
                if var.scope.get().is_class() {
                  if self.cur_method.get().static_ {
                    let method = self.cur_method.get().name;
                    self.issue(expr_loc, RefInStatic { field: id.name, method });
                  } else {
                    // add a virtual `this`, it doesn't need visit
                    *owner_ptr.get() = Some(Box::new(Expr::with_type(expr_loc, SemanticType::Object(self.cur_class), ExprData::This)));
                  }
                }
              }
            }
          }
          None => self.issue(expr_loc, UndeclaredVar { name: id.name }),
        }
        self.cur_id_used_for_ref = false;
      }
    }
  }

  fn default(&mut self, default: &mut Default, expr_loc: Loc, expr_type: &mut SemanticType) {
    let Default { arr, idx, dft } = default;
    (self.expr(arr), self.expr(idx), self.expr(dft));
    match &arr.type_ {
      SemanticType::Array(elem) => {
        *expr_type = *elem.clone();
        if dft.type_ != ERROR && &dft.type_ != elem.as_ref() {
          self.issue(expr_loc, DefaultMismatch { elem_t: elem.to_string(), dft_t: dft.type_.to_string() });
        }
      }
      SemanticType::Error => {}
      _ => {
        self.issue(arr.loc, BadArrayOp {});
        *expr_type = dft.type_.clone();
      }
    }
    if !idx.type_.error_or(&INT) {
      self.issue(idx.loc, ArrayIndexNotInt {});
    }
  }
}