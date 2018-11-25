use super::ast::*;
use super::types::*;
use super::errors::*;
use super::symbol::*;
use std::ptr;

macro_rules! issue {
  ($rec:expr, $loc: expr, $err: expr) => {
      $rec.errors.push(Error::new($loc, $err));
  };
}

pub struct TypeChecker {
  errors: Vec<Error>,
  scopes: ScopeStack,
  loop_counter: i32,
  current_method: *const MethodDef,
  current_class: *const ClassDef,
  current_id_used_for_ref: bool,
}

impl TypeChecker {
  pub fn new() -> TypeChecker {
    TypeChecker {
      errors: Vec::new(),
      scopes: ScopeStack {
        global_scope: ptr::null_mut(),
        scopes: Vec::new(),
      },
      loop_counter: 0,
      current_method: ptr::null(),
      current_class: ptr::null(),
      current_id_used_for_ref: false,
    }
  }

  pub fn check(mut self, mut program: Program) -> Result<Program, Vec<Error>> {
    self.visit_program(&mut program);
    if self.errors.is_empty() {
      Ok(program)
    } else {
      self.errors.sort_by_key(|x| x.loc);
      Err(self.errors)
    }
  }

  fn check_bool(&mut self, expr: &mut Expr) {
    self.visit_expr(expr);
    let t = expr.get_type();
    if !t.error_or(&BOOL) {
      issue!(self, expr.get_loc(), TestNotBool);
    }
  }

  unsafe fn check_call(&mut self, call: &mut Call, symbol: Option<Symbol>) {
    let owner_t = match &call.owner {
      Some(owner) => owner.get_type().clone(),
      None => (*self.current_class).get_object_type(),
    };
    match symbol {
      Some(symbol) => {
        match symbol {
          Symbol::Method(method) => {
            let method = &*method;
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
                match ((*self.current_method).static_, method.static_) {
                  (true, false) => issue!(self, call.loc, RefInStatic { field: method.name, method: (*self.current_method).name }),
                  (false, false) => call.owner = Some(Box::new(Expr::This(This {
                    loc: call.loc,
                    type_: (*self.current_class).get_object_type(),
                  }))),
                  _ => {}
                }
              }
            };
            for expr in &mut call.args { self.visit_expr(expr); }
            let this_offset = if method.static_ { 0 } else { 1 };
            let argc = call.args.len();
            if argc != method.params.len() - this_offset {
              issue!(self, call.loc, WrongArgc { name: call.name, expect: method.params.len() as i32, actual: argc as i32 });
            } else {
              for i in this_offset..argc + this_offset {
                let arg_t = call.args[i - this_offset].get_type();
                if !arg_t.extends(&method.params[i].type_.sem) {
                  issue!(self, call.args[i].get_loc(), WrongArgType {
                    loc: (i + 1 - this_offset) as i32,
                    arg_t: arg_t.to_string(),
                    param_t: method.params[i].type_.sem.to_string()
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

impl Visitor for TypeChecker {
  fn visit_program(&mut self, program: &mut Program) {
    self.scopes.open(&mut program.scope);
    for class_def in &mut program.classes { self.visit_class_def(class_def); }
    self.scopes.close();
  }

  fn visit_class_def(&mut self, class_def: &mut ClassDef) {
    self.current_class = class_def;
    self.scopes.open(&mut class_def.scope);
    for field_def in &mut class_def.fields { self.visit_field_def(field_def) }
    self.scopes.close();
  }

  fn visit_method_def(&mut self, method_def: &mut MethodDef) {
    self.current_method = method_def;
    self.scopes.open(&mut method_def.scope);
    self.visit_block(&mut method_def.body);
    self.scopes.close();
  }

  fn visit_var_assign(&mut self, var_assign: &mut VarAssign) {
    self.visit_expr(&mut var_assign.src);
    var_assign.type_ = var_assign.src.get_type().clone();
  }

  fn visit_block(&mut self, block: &mut Block) {
    self.scopes.open(&mut block.scope);
    for stmt in &mut block.stmts { self.visit_stmt(stmt); }
    self.scopes.close();
  }

  fn visit_while(&mut self, while_: &mut While) {
    self.check_bool(&mut while_.cond);
    self.loop_counter += 1;
    self.visit_block(&mut while_.body);
    self.loop_counter -= 1;
  }

  fn visit_for(&mut self, for_: &mut For) {
    let block = &mut for_.body;
    self.scopes.open(&mut block.scope);
    self.visit_simple(&mut for_.init);
    self.check_bool(&mut for_.cond);
    self.visit_simple(&mut for_.update);
    for stmt in &mut block.stmts { self.visit_stmt(stmt); }
    self.scopes.close();
  }

  fn visit_if(&mut self, if_: &mut If) {
    self.check_bool(&mut if_.cond);
    self.visit_block(&mut if_.on_true);
    if let Some(on_false) = &mut if_.on_false { self.visit_block(on_false); }
  }

  fn visit_break(&mut self, break_: &mut Break) {
    if self.loop_counter == 0 { issue!(self, break_.loc, BreakOutOfLoop); }
  }

  fn visit_return(&mut self, return_: &mut Return) {
    unsafe {
      let expect = &(*self.current_method).ret_t.sem;
      match &mut return_.expr {
        Some(expr) => {
          self.visit_expr(expr);
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
  }

  fn visit_s_copy(&mut self, s_copy: &mut SCopy) {
    self.visit_expr(&mut s_copy.src);
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
        } else if src_t != &ERROR && &dst_t != src_t {
          issue!(self, s_copy.loc, SCopyMismatch { dst_t: dst_t.to_string(), src_t: src_t.to_string() });
        }
      }
      None => {
        issue!(self, s_copy.dst_loc, UndeclaredVar { name: s_copy.dst });
        if src_t != &ERROR && !src_t.is_object() {
          issue!(self, s_copy.src.get_loc(), SCopyNotClass { which: "src", type_: src_t.to_string() });
        };
      }
    }
  }

  fn visit_new_class(&mut self, new_class: &mut NewClass) {
    match self.scopes.lookup_class(new_class.name) {
      Some(class) => new_class.type_ = class.get_type(),
      None => issue!(self, new_class.loc, NoSuchClass { name: new_class.name }),
    }
  }

  fn visit_new_array(&mut self, new_array: &mut NewArray) {
    let elem_t = &mut new_array.elem_type;
    new_array.type_ = SemanticType::Array(Box::new(elem_t.sem.clone()));
    self.visit_semantic_type(&mut new_array.type_, elem_t.loc);
    self.visit_expr(&mut new_array.len);
    let len_t = new_array.len.get_type();
    if !len_t.error_or(&INT) {
      issue!(self, new_array.len.get_loc(), BadNewArrayLen);
    }
  }

  fn visit_assign(&mut self, assign: &mut Assign) {
    self.visit_expr(&mut assign.dst);
    self.visit_expr(&mut assign.src);
    let dst_t = assign.dst.get_type();
    let src_t = assign.src.get_type();
    // error check is contained in extends
    if dst_t.is_method() || !src_t.extends(dst_t) {
      issue!(self, assign.loc, IncompatibleBinary{left_t: dst_t.to_string(), opt: "=", right_t: src_t.to_string() })
    }
  }

  fn visit_unary(&mut self, unary: &mut Unary) {
    self.visit_expr(&mut unary.opr);
    let opr = unary.opr.get_type();
    match unary.opt {
      Operator::Neg => {
        if opr.error_or(&INT) {
          unary.type_ = INT;
        } else {
          issue!(self, unary.loc, IncompatibleUnary { opt: "-", type_: opr.to_string() });
        }
      }
      Operator::Not => {
        if !opr.error_or(&BOOL) {
          issue!(self, unary.loc, IncompatibleUnary { opt: "!", type_: opr.to_string() });
        }
        // no matter error or not, set type to bool
        unary.type_ = BOOL;
      }
      _ => unreachable!(),
    }
  }

  fn visit_binary(&mut self, binary: &mut Binary) {
    self.visit_expr(&mut binary.left);
    self.visit_expr(&mut binary.right);
    let (left, right) = (&*binary.left, &*binary.right);
    let (left_t, right_t) = (left.get_type(), right.get_type());
    if left_t == &ERROR || right_t == &ERROR {
      match binary.opt {
        Operator::Add | Operator::Sub | Operator::Mul | Operator::Div | Operator::Mod => binary.type_ = left_t.clone(),
        Operator::Repeat | Operator::Concat => unimplemented!(),
        _ => binary.type_ = BOOL,
      }
      return;
    }
    if !match binary.opt {
      Operator::Add | Operator::Sub | Operator::Mul | Operator::Div | Operator::Mod => {
        binary.type_ = left_t.clone();
        left_t == &INT && right_t == &INT
      }
      Operator::Lt | Operator::Le | Operator::Gt | Operator::Ge => {
        binary.type_ = BOOL;
        left_t == &INT && right_t == &INT
      }
      Operator::Eq | Operator::Ne => {
        binary.type_ = BOOL;
        left_t == right_t
      }
      Operator::And | Operator::Or => {
        binary.type_ = BOOL;
        left_t == &BOOL && right_t == &BOOL
      }
      Operator::Repeat | Operator::Concat => unimplemented!(),
      _ => unreachable!(),
    } {
      issue!(self, binary.loc, IncompatibleBinary {
          left_t: left_t.to_string(),
          opt: binary.opt.to_str(),
          right_t: right_t.to_string(),
      });
    }
  }

  fn visit_call(&mut self, call: &mut Call) {
    let call_ptr = call as *mut Call;
    match unsafe { &mut (*call_ptr).owner } {
      Some(owner) => {
        self.current_id_used_for_ref = true;
        self.visit_expr(owner);
        let owner_t = owner.get_type();
        if owner_t == &ERROR { return; }
        // check array length call, quite a dirty implementation
        if call.name == "length" {
          if owner_t.is_array() {
            if !call.args.is_empty() {
              issue!(self, call.loc, LengthWithArgument { count: call.args.len() as i32 });
            }
            call.type_ = INT;
            return;
          } else if !owner_t.is_object() && !owner_t.is_class() {
            issue!(self, call.loc, BadLength);
            return;
          }
        }
        if !owner_t.is_object() && !owner_t.is_class() {
          issue!(self, call.loc, BadFieldAccess{name: call.name, owner_t: owner_t.to_string() });
        } else {
          let symbol = owner_t.get_class().lookup(call.name);
          unsafe { self.check_call(call, symbol); }
        }
      }
      None => unsafe {
        let symbol = (*self.current_class).lookup(call.name);
        self.check_call(call, symbol);
      }
    }
  }

  fn visit_print(&mut self, print: &mut Print) {
    for (i, expr) in print.print.iter_mut().enumerate() {
      self.visit_expr(expr);
      let expr_t = expr.get_type();
      if expr_t != &ERROR && expr_t != &BOOL && expr_t != &INT && expr_t != &STRING {
        issue!(self, expr.get_loc(), BadPrintArg { loc: i as i32 + 1, type_: expr_t.to_string() });
      }
    }
  }

  fn visit_this(&mut self, this: &mut This) {
    unsafe {
      if (*self.current_method).static_ {
        issue!(self, this.loc, ThisInStatic);
      } else {
        this.type_ = (*self.current_class).get_object_type();
      }
    }
  }

  fn visit_type_cast(&mut self, type_cast: &mut TypeCast) {
    self.visit_expr(&mut type_cast.expr);
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

  fn visit_type_test(&mut self, type_test: &mut TypeTest) {
    self.visit_expr(&mut type_test.expr);
    let expr_t = type_test.expr.get_type();
    if expr_t != &ERROR && !expr_t.is_object() {
      issue!(self, type_test.loc, NotObject { type_: expr_t.to_string() });
    }
    if self.scopes.lookup_class(type_test.name).is_none() {
      issue!(self, type_test.loc, NoSuchClass { name: type_test.name });
    }
  }

  fn visit_indexed(&mut self, indexed: &mut Indexed) {
    self.visit_expr(&mut indexed.array);
    self.visit_expr(&mut indexed.index);
    let (arr_t, idx_t) = (indexed.array.get_type(), indexed.index.get_type());
    match &arr_t {
      SemanticType::Array(elem) => indexed.type_ = *elem.clone(),
      SemanticType::Error => {}
      _ => issue!(self, indexed.array.get_loc(), NotArray),
    }
    if idx_t != &ERROR && idx_t != &INT {
      issue!(self, indexed.loc, BadArrayIndex);
    }
  }

  fn visit_identifier(&mut self, id: &mut Identifier) {
    // not found(no owner) or sole ClassName => UndeclaredVar
    // refer to field in static function => RefInStatic
    // <not object>.a (Main.a, 1.a, func.a) => BadFieldAssess
    // access a field that doesn't belong to self & parent => PrivateFieldAccess
    // given owner but not found object.a => NoSuchField

    // actually a ClassName in the looking-up process is bound to occur an error
    // wither UndeclaredVar or BadFieldAssess

    unsafe {
      let owner_ptr = &mut id.owner as *mut _; // workaround with borrow check
      match &mut id.owner {
        Some(owner) => {
          self.current_id_used_for_ref = true;
          self.visit_expr(owner);
          let owner_t = owner.get_type();
          match owner_t {
            SemanticType::Object(_, class) => {
              let class = &**class;
              // lookup through inheritance chain
              match class.lookup(id.name) {
                Some(symbol) => {
                  match symbol {
                    Symbol::Var(var) => {
                      id.type_ = var.get_type().clone();
                      if !(*self.current_class).extends(class) {
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
                  if !self.current_id_used_for_ref {
                    issue!(self, id.loc, UndeclaredVar { name: id.name });
                  } else { id.type_ = SemanticType::Class(class); }
                }
                Symbol::Method(method) => id.type_ = SemanticType::Method(method),
                Symbol::Var(var) => {
                  id.type_ = var.get_type().clone();
                  if var.get_scope().is_class() {
                    if (*self.current_method).static_ {
                      issue!(self, id.loc, RefInStatic { field: id.name, method: (*self.current_method).name });
                    } else {
                      // add a virtual `this`, it doesn't need visit
                      *owner_ptr = Some(Box::new(Expr::This(This {
                        loc: id.loc,
                        type_: SemanticType::Object((*self.current_class).name, self.current_class),
                      })));
                    }
                  }
                }
              }
            }
            None => issue!(self, id.loc, UndeclaredVar { name: id.name }),
          }
          self.current_id_used_for_ref = false;
        }
      }
    }
  }
}