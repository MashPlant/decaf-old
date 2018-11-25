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

  unsafe fn check_call(&mut self, call_: &mut Call, symbol: Option<Symbol>) {
    let owner_t = match &call_.owner {
      Some(owner) => owner.get_type().clone(),
      None => (*self.current_class).get_object_type(),
    };
    match symbol {
      Some(symbol) => {
        match symbol {
          Symbol::Method(method) => {
            let method = &*method;
            call_.method = method;
            call_.type_ = method.ret_t.sem.clone();
            match &call_.owner {
              Some(_) => {
                if owner_t.is_class() && !method.static_ {
                  // call a instance method through class reference
                  issue!(self, call_.loc, BadFieldAccess { name: call_.name, owner_t: owner_t.to_string() });
                }
                if method.static_ {
                  call_.owner = None;
                }
              }
              None => {
                match ((*self.current_method).static_, method.static_) {
                  (true, false) => issue!(self, call_.loc, RefInStatic { field: method.name, method: (*self.current_method).name }),
                  (false, false) => call_.owner = Some(Box::new(Expr::This(This {
                    loc: call_.loc,
                    type_: (*self.current_class).get_object_type(),
                  }))),
                  _ => {}
                }
              }
            };
            for expr in &mut call_.args { self.visit_expr(expr); }
            let this_offset = if method.static_ { 0 } else { 1 };
            let argc = call_.args.len();
            if argc != method.params.len() - this_offset {
              issue!(self, call_.loc, WrongArgc { name: call_.name, expect: method.params.len() as i32, actual: argc as i32 });
            } else {
              for i in this_offset..argc + this_offset {
                let arg_t = call_.args[i - this_offset].get_type();
                if arg_t != &ERROR && !arg_t.extends(&method.params[i].type_.sem) {
                  issue!(self, call_.args[i].get_loc(), WrongArgType {
                      loc: i as i32,
                      arg_t: arg_t.to_string(),
                      param_t: method.params[i].type_.sem.to_string()
                  });
                }
              }
            }
          }
          _ => {
            issue!(self, call_.loc, NotMethod { name: call_.name, owner_t: owner_t.to_string() });
            call_.type_ = ERROR;
          }
        }
      }
      None => {
        issue!(self, call_.loc, NoSuchField { name: call_.name, owner_t: owner_t.to_string() });
        call_.type_ = ERROR;
      }
    };
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

  fn visit_break(&mut self, break_: &mut Break) {
    if self.loop_counter == 0 { issue!(self, break_.loc, BreakOutOfLoop); }
  }

  fn visit_assign(&mut self, assign: &mut Assign) {
    self.visit_expr(&mut assign.dst);
    self.visit_expr(&mut assign.src);
    let dst_type = assign.dst.get_type();
    let src_type = assign.src.get_type();
    // error check is contained in extends
    if dst_type.is_method() || !src_type.extends(dst_type) {
      issue!(self, assign.loc, IncompatibleBinary{left_t:dst_type.to_string(), opt:"=", right_t:src_type.to_string() })
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
          unary.type_ = ERROR;
        }
      }
      Operator::Not => {
        if !opr.error_or(&BOOL) {
          issue!(self, unary.loc, IncompatibleUnary { opt: "!", type_: opr.to_string() });
          unary.type_ = ERROR;
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

  fn visit_call(&mut self, call_: &mut Call) {
    let call_ptr = call_ as *mut Call;
    match unsafe { &mut (*call_ptr).owner } {
      Some(owner) => {
        self.current_id_used_for_ref = true;
        self.visit_expr(owner);
        let owner_t = owner.get_type();
        if owner_t == &ERROR {
          call_.type_ = ERROR;
          return;
        }
        // check array length call
        // quite a dirty implementation
        if call_.name == "length" {
          if owner_t.is_array() {
            if !call_.args.is_empty() {
              issue!(self, call_.loc, LengthWithArgument { count: call_.args.len() as i32 });
            }
            call_.type_ = INT;
          } else if !owner_t.is_object() {
            issue!(self, call_.loc, BadLength);
            call_.type_ = ERROR;
          }
        }
        if !owner_t.is_object() {
          issue!(self, call_.loc, BadFieldAccess{name: call_.name, owner_t: owner_t.to_string() });
          call_.type_ = ERROR;
          return;
        }
        let symbol = owner_t.get_class().lookup(call_.name);
        unsafe { self.check_call(call_, symbol); }
      }
      None => unsafe {
        let symbol = (*self.current_class).lookup(call_.name);
        self.check_call(call_, symbol);
      }
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
      let owner_ptr = &mut id.owner as *mut _; // work with borrow check
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
                None => {
                  issue!(self, id.loc, NoSuchField { name: id.name, owner_t: owner_t.to_string() });
                  id.type_ = ERROR;
                }
              }
            }
            SemanticType::Error => id.type_ = ERROR,
            _ => {
              issue!(self, id.loc, BadFieldAccess{name: id.name, owner_t: owner_t.to_string() });
              id.type_ = ERROR;
            }
          }
        }
        None => {
          match self.scopes.lookup_before(id.name, id.loc) {
            Some(symbol) => {
              match symbol {
                Symbol::Class(class) => {
                  if !self.current_id_used_for_ref {
                    issue!(self, id.loc, UndeclaredVar { name: id.name });
                    id.type_ = ERROR;
                  } else { id.type_ = SemanticType::Class(class); }
                }
                Symbol::Method(method) => id.type_ = SemanticType::Method(method),
                Symbol::Var(var) => {
                  id.type_ = var.get_type().clone();
                  if var.get_scope().is_class() && (*self.current_method).static_ {
                    issue!(self, id.loc, RefInStatic {
                        field: id.name,
                        method: (*self.current_method).name
                    });
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
            None => {
              issue!(self, id.loc, UndeclaredVar { name: id.name });
              id.type_ = ERROR;
            }
          }
          self.current_id_used_for_ref = false;
        }
      }
    }
  }
}