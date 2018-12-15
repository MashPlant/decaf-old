use super::ast::*;
use super::types::*;
use super::errors::*;
use super::symbol::*;
use super::util::*;
use std::ptr;

macro_rules! issue {
  ($rec:expr, $loc: expr, $err: expr) => {
      $rec.errors.push(Error::new($loc, $err));
  };
}

pub struct TypeChecker {
  errors: Vec<Error>,
  scopes: ScopeStack,
  loop_cnt: i32,
  cur_method: *const MethodDef,
  cur_class: *const ClassDef,
  cur_id_used_for_ref: bool,
}

impl TypeChecker {
  pub fn new() -> TypeChecker {
    TypeChecker {
      errors: Vec::new(),
      scopes: ScopeStack {
        global_scope: ptr::null_mut(),
        scopes: Vec::new(),
      },
      loop_cnt: 0,
      cur_method: ptr::null(),
      cur_class: ptr::null(),
      cur_id_used_for_ref: false,
    }
  }

  pub fn check(mut self, mut program: Program) -> Result<Program, Vec<Error>> {
    self.scopes.open(&mut program.scope);
    for class_def in &mut program.class { self.class_def(class_def); }
    self.scopes.close();
    if self.errors.is_empty() {
      Ok(program)
    } else {
      self.errors.sort_by_key(|x| x.loc);
      Err(self.errors)
    }
  }

  fn check_bool(&mut self, expr: &mut Expr) {
    self.expr(expr);
    let t = expr.get_type();
    if !t.error_or(&BOOL) {
      issue!(self, expr.get_loc(), TestNotBool{});
    }
  }

  fn check_call(&mut self, call: &mut Call, symbol: Option<Symbol>) {
    let owner_t = match &call.owner {
      Some(owner) => owner.get_type().clone(),
      None => self.cur_class.get().get_object_type(),
    };
    match symbol {
      Some(symbol) => {
        match symbol {
          Symbol::Method(method) => {
            let method = method.get();
            call.method = method;
            call.type_ = method.ret_t.sem.clone();
            match &call.owner {
              Some(_) => {
                if owner_t.is_class() && !method.static_ {
                  // call a instance method through class reference
                  issue!(self, call.loc, BadFieldAccess { name: call.name, owner_t: owner_t.to_string() });
                }
                if method.static_ {
                  call.owner = None;
                }
              }
              None => {
                match (self.cur_method.get().static_, method.static_) {
                  (true, false) => issue!(self, call.loc, RefInStatic { field: method.name, method: self.cur_method.get().name }),
                  (false, false) => call.owner = Some(Box::new(Expr::This(This {
                    loc: call.loc,
                    type_: self.cur_class.get().get_object_type(),
                  }))),
                  _ => {}
                }
              }
            };
            for expr in &mut call.arg { self.expr(expr); }
            let this_offset = if method.static_ { 0 } else { 1 };
            let argc = call.arg.len();
            if argc != method.param.len() - this_offset {
              issue!(self, call.loc, WrongArgc { name: call.name, expect: method.param.len() as i32, actual: argc as i32 });
            } else {
              for i in this_offset..argc + this_offset {
                let arg_t = call.arg[i - this_offset].get_type();
                if !arg_t.extends(&method.param[i].type_.sem) {
                  issue!(self, call.arg[i].get_loc(), WrongArgType {
                    loc: (i + 1 - this_offset) as i32,
                    arg_t: arg_t.to_string(),
                    param_t: method.param[i].type_.sem.to_string()
                  });
                }
              }
            }
          }
          _ => issue!(self, call.loc, NotMethod { name: call.name, owner_t: owner_t.to_string() }),
        }
      }
      None => issue!(self, call.loc, NoSuchField { name: call.name, owner_t: owner_t.to_string() }),
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
            let expr_t = expr.get_type();
            if !expr_t.extends(expect) {
              issue!(self, return_.loc, WrongReturnType { ret_t: expr_t.to_string(), expect_t: expect.to_string() });
            }
          }
          None => {
            if expect != &VOID {
              issue!(self, return_.loc, WrongReturnType { ret_t: "void".to_owned(), expect_t: expect.to_string() });
            }
          }
        }
      }
      Print(print) => for (i, expr) in print.print.iter_mut().enumerate() {
        self.expr(expr);
        let expr_t = expr.get_type();
        if expr_t != &ERROR && expr_t != &BOOL && expr_t != &INT && expr_t != &STRING {
          issue!(self, expr.get_loc(), BadPrintArg { loc: i as i32 + 1, type_: expr_t.to_string() });
        }
      },
      Break(break_) => if self.loop_cnt == 0 { issue!(self, break_.loc, BreakOutOfLoop{}); },
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
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        let dst_t = assign.dst.get_type();
        let src_t = assign.src.get_type();
        // error check is contained in extends
        if dst_t.is_method() || !src_t.extends(dst_t) {
          issue!(self, assign.loc, IncompatibleBinary{l_t: dst_t.to_string(), op: "=", r_t: src_t.to_string() })
        }
      }
      Simple::VarAssign(var_assign) => if let Some(src) = &mut var_assign.src { // if it doesn't have an src, it is an old var_def, and thus needn't handling
        self.expr(src);
        let src_t = src.get_type();
        if var_assign.type_.sem == VAR {
          var_assign.type_.sem = src_t.clone();
        } else if !src_t.extends(&var_assign.type_.sem) {
          issue!(self, var_assign.loc, IncompatibleBinary{l_t: var_assign.type_.sem.to_string(), op: "=", r_t: src_t.to_string() })
        }
      }
      Simple::Expr(expr) => self.expr(expr),
      _ => {}
    }
  }

  fn expr(&mut self, expr: &mut Expr) {
    use self::Expr::*;
    match expr {
      Identifier(identifier) => self.identifier(identifier),
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
        let (arr_t, idx_t) = (indexed.arr.get_type(), indexed.idx.get_type());
        match &arr_t {
          SemanticType::Array(elem) => indexed.type_ = *elem.clone(),
          SemanticType::Error => {}
          _ => issue!(self, indexed.arr.get_loc(), NotArray{}),
        }
        if !idx_t.error_or(&INT) {
          issue!(self, indexed.loc, ArrayIndexNotInt{});
        }
      }
      Call(call) => self.call(call),
      Unary(unary) => self.unary(unary),
      Binary(binary) => self.binary(binary),
      This(this) => if self.cur_method.get().static_ {
        issue!(self, this.loc, ThisInStatic{});
      } else {
        this.type_ = self.cur_class.get().get_object_type();
      },
      NewClass(new_class) => match self.scopes.lookup_class(new_class.name) {
        Some(class) => new_class.type_ = class.get_type(),
        None => issue!(self, new_class.loc, NoSuchClass { name: new_class.name }),
      },
      NewArray(new_array) => {
        let elem_t = &mut new_array.elem_t;
        new_array.type_ = SemanticType::Array(Box::new(elem_t.sem.clone()));
        self.semantic_type(&mut new_array.type_, elem_t.loc);
        self.expr(&mut new_array.len);
        let len_t = new_array.len.get_type();
        if !len_t.error_or(&INT) { issue!(self, new_array.len.get_loc(), BadNewArrayLen{}); }
      }
      TypeTest(type_test) => {
        self.expr(&mut type_test.expr);
        let expr_t = type_test.expr.get_type();
        if expr_t != &ERROR && !expr_t.is_object() {
          issue!(self, type_test.loc, NotObject { type_: expr_t.to_string() });
        }
        if self.scopes.lookup_class(type_test.name).is_none() {
          issue!(self, type_test.loc, NoSuchClass { name: type_test.name });
        }
      }
      TypeCast(type_cast) => {
        self.expr(&mut type_cast.expr);
        let expr_t = type_cast.expr.get_type();
        if expr_t != &ERROR && !expr_t.is_object() {
          issue!(self, type_cast.loc, NotObject { type_: expr_t.to_string() });
        }
        // doesn't need to set type to error because it originally was
        match self.scopes.lookup_class(type_cast.name) {
          Some(class) => type_cast.type_ = class.get_type(),
          None => issue!(self, type_cast.loc, NoSuchClass { name: type_cast.name }),
        }
      }
      Default(default) => self.default(default),
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
    let src_t = s_copy.src.get_type();
    match self.scopes.lookup_before(s_copy.dst, s_copy.loc) {
      Some(symbol) => {
        let dst_t = symbol.get_type();
        if &dst_t != &ERROR && !dst_t.is_object() {
          issue!(self, s_copy.dst_loc, SCopyNotClass { which: "dst", type_: dst_t.to_string() });
        };
        if !dst_t.is_object() {
          if src_t != &ERROR && !src_t.is_object() {
            issue!(self, s_copy.src.get_loc(), SCopyNotClass { which: "src", type_: src_t.to_string() });
          };
        } else if !src_t.error_or(&dst_t) {
          issue!(self, s_copy.loc, SCopyMismatch { dst_t: dst_t.to_string(), src_t: src_t.to_string() });
        }
        if let Symbol::Var(var) = symbol { s_copy.dst_sym = var; }
      }
      None => {
        issue!(self, s_copy.dst_loc, UndeclaredVar { name: s_copy.dst });
        if src_t != &ERROR && !src_t.is_object() {
          issue!(self, s_copy.src.get_loc(), SCopyNotClass { which: "src", type_: src_t.to_string() });
        };
      }
    }
  }

  fn foreach(&mut self, foreach: &mut Foreach) {
    // arr is visited before scope open, cond is after
    self.expr(&mut foreach.arr);
    let arr_t = foreach.arr.get_type();
    match arr_t {
      SemanticType::Array(elem) => match foreach.def.type_.sem {
        ref mut t if t == &VAR => {
          *t = *elem.clone();
          foreach.body.scope.kind = ScopeKind::Local(&mut foreach.body);
          foreach.def.scope = &foreach.body.scope;
        }
        ref t if !elem.extends(t) => issue!(self, foreach.def.loc, ForeachMismatch{ elem_t: elem.to_string(), def_t: t.to_string() }),
        _ => {}
      }
      SemanticType::Error => if &foreach.def.type_.sem == &VAR {
        // if declared type is not 'var', retain it; otherwise set it to error
        foreach.def.type_.sem = ERROR;
      }
      _ => {
        issue!(self, foreach.arr.get_loc(), BadArrayOp{});
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

  fn unary(&mut self, unary: &mut Unary) {
    use super::ast::Operator::*;
    self.expr(&mut unary.r);
    let r = unary.r.get_type();
    match unary.op {
      Neg => {
        if !r.error_or(&INT) {
          issue!(self, unary.loc, IncompatibleUnary { op: "-", r_t: r.to_string() });
        }
        unary.type_ = INT;
      }
      Not => {
        if !r.error_or(&BOOL) {
          issue!(self, unary.loc, IncompatibleUnary { op: "!", r_t: r.to_string() });
        }
        unary.type_ = BOOL;
      }
      PreInc | PreDec | PostInc | PostDec => {
        let op = unary.op.to_str();
        if !unary.r.is_lvalue() {
          issue!(self, unary.loc, NotLValue { op });
        }
        if !r.error_or(&INT) {
          issue!(self, unary.loc, IncompatibleUnary { op, r_t: r.to_string() });
        }
        unary.type_ = INT;
      }
      _ => unreachable!(),
    }
  }

  fn binary(&mut self, binary: &mut Binary) {
    use super::ast::Operator::*;
    self.expr(&mut binary.l);
    self.expr(&mut binary.r);
    let (l, r) = (&mut binary.l, &mut binary.r);
    let (l_t, r_t) = (l.get_type(), r.get_type());
    match binary.op {
      Repeat => {
        if !r_t.error_or(&INT) { issue!(self, r.get_loc(), ArrayRepeatNotInt{}); }
        // l_t cannot be void here
        if l_t != &ERROR {
          binary.type_ = SemanticType::Array(Box::new(l_t.clone()));
        }
      }
      Concat => {
        if l_t != &ERROR && !l_t.is_array() { issue!(self, l.get_loc(), BadArrayOp{}); }
        if r_t != &ERROR && !r_t.is_array() { issue!(self, r.get_loc(), BadArrayOp{}); }
        if l_t.is_array() && r_t.is_array() {
          if l_t != r_t {
            issue!(self, binary.loc, ConcatMismatch { l_t: l_t.to_string(), r_t: r_t.to_string() });
          }
          binary.type_ = l_t.clone();
        }
      }
      _ => {
        if l_t == &ERROR || r_t == &ERROR {
          return binary.type_ = match binary.op {
            Add | Sub | Mul | Div | Mod | BAnd | BOr | BXor | Shl | Shr => l_t.clone(),
            _ => BOOL,
          };
        }
        if !match binary.op {
          Add | Sub | Mul | Div | Mod | BAnd | BOr | BXor | Shl | Shr => {
            binary.type_ = l_t.clone();
            l_t == &INT && r_t == &INT
          }
          Lt | Le | Gt | Ge => {
            binary.type_ = BOOL;
            l_t == &INT && r_t == &INT
          }
          Eq | Ne => {
            binary.type_ = BOOL;
            l_t.extends(r_t) || r_t.extends(l_t)
          }
          And | Or => {
            binary.type_ = BOOL;
            l_t == &BOOL && r_t == &BOOL
          }
          _ => unreachable!(),
        } {
          issue!(self, binary.loc, IncompatibleBinary {
            l_t: l_t.to_string(), op: binary.op.to_str(), r_t: r_t.to_string(),
          });
        }
      }
    }
  }

  fn call(&mut self, call: &mut Call) {
    let call_ptr = call as *mut Call;
    match { &mut call_ptr.get().owner } {
      Some(owner) => {
        self.cur_id_used_for_ref = true;
        self.expr(owner);
        let owner_t = owner.get_type();
        if owner_t == &ERROR { return; }
        // check array length call, quite a dirty implementation
        if call.name == "length" {
          if owner_t.is_array() {
            if !call.arg.is_empty() {
              issue!(self, call.loc, LengthWithArgument { count: call.arg.len() as i32 });
            }
            call.type_ = INT;
            call.is_arr_len = true;
            return;
          } else if !owner_t.is_object() && !owner_t.is_class() {
            issue!(self, call.loc, BadLength{});
            return;
          }
        }
        if !owner_t.is_object() && !owner_t.is_class() {
          issue!(self, call.loc, BadFieldAccess{name: call.name, owner_t: owner_t.to_string() });
        } else {
          let symbol = owner_t.get_class().lookup(call.name);
          self.check_call(call, symbol);
        }
      }
      None => {
        let symbol = self.cur_class.get().lookup(call.name);
        self.check_call(call, symbol);
      }
    }
  }

  fn identifier(&mut self, id: &mut Identifier) {
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
        let owner_t = owner.get_type();
        match owner_t {
          SemanticType::Object(class) => {
            let class = class.get();
            // lookup through inheritance chain
            match class.lookup(id.name) {
              Some(symbol) => {
                match symbol {
                  Symbol::Var(var) => {
                    id.type_ = var.get_type().clone();
                    id.symbol = var;
                    if !self.cur_class.get().extends(class) {
                      issue!(self, id.loc, PrivateFieldAccess { name: id.name, owner_t: owner_t.to_string() });
                    }
                  }
                  _ => id.type_ = symbol.get_type(),
                }
              }
              None => issue!(self, id.loc, NoSuchField { name: id.name, owner_t: owner_t.to_string() }),
            }
          }
          SemanticType::Error => {}
          _ => issue!(self, id.loc, BadFieldAccess{name: id.name, owner_t: owner_t.to_string() }),
        }
      }
      None => {
        match self.scopes.lookup_before(id.name, id.loc) {
          Some(symbol) => {
            match symbol {
              Symbol::Class(class) => {
                if !self.cur_id_used_for_ref {
                  issue!(self, id.loc, UndeclaredVar { name: id.name });
                } else { id.type_ = SemanticType::Class(class); }
              }
              Symbol::Method(method) => id.type_ = SemanticType::Method(method),
              Symbol::Var(var) => {
                id.type_ = var.get_type().clone();
                id.symbol = var;
                if var.get_scope().is_class() {
                  if self.cur_method.get().static_ {
                    issue!(self, id.loc, RefInStatic { field: id.name, method: self.cur_method.get().name });
                  } else {
                    // add a virtual `this`, it doesn't need visit
                    *owner_ptr.get() = Some(Box::new(Expr::This(This {
                      loc: id.loc,
                      type_: SemanticType::Object(self.cur_class),
                    })));
                  }
                }
              }
            }
          }
          None => issue!(self, id.loc, UndeclaredVar { name: id.name }),
        }
        self.cur_id_used_for_ref = false;
      }
    }
  }

  fn default(&mut self, default: &mut Default) {
    self.expr(&mut default.arr);
    self.expr(&mut default.idx);
    self.expr(&mut default.dft);
    let (arr_t, idx_t, dft_t) =
      (default.arr.get_type(), default.idx.get_type(), default.dft.get_type());
    match arr_t {
      SemanticType::Array(elem) => {
        default.type_ = *elem.clone();
        if dft_t != &ERROR && dft_t != elem.as_ref() {
          issue!(self, default.loc, DefaultMismatch { elem_t: elem.to_string(), dft_t: dft_t.to_string() });
        }
      }
      SemanticType::Error => {}
      _ => {
        issue!(self, default.arr.get_loc(), BadArrayOp{});
        default.type_ = dft_t.clone();
      }
    }
    if !idx_t.error_or(&INT) {
      issue!(self, default.idx.get_loc(), ArrayIndexNotInt{});
    }
  }
}