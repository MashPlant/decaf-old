extern crate backend;

use backend::*;
use backend::jvm::class::{ACC_PUBLIC, ACC_STATIC, ACC_FINAL};
use backend::jvm::writer::*;
use super::ast::*;
use super::types::*;
use super::symbol::*;

use std::ptr;

macro_rules! handle {
  ($t: expr, $int_bool: expr, $object: expr) => {
    match $t {
      SemanticType::Basic(name) => match *name {
        "int" | "bool" => $int_bool,
        "string" => $object,
        _ => unreachable!(),
      }
      SemanticType::Object(_) => $object,
      SemanticType::Array(_) => $object,
      _ => unreachable!(),
    }
  };
  ($t: expr, $int: expr, $bool: expr, $object: expr) => {
    match $t {
      SemanticType::Basic(name) => match *name {
        "int" => $int,
        "bool" => $bool,
        "string" => $object,
        _ => unreachable!(),
      }
      SemanticType::Object(_) => $object,
      SemanticType::Array(_) => $object,
      _ => unreachable!(),
    }
  };
}

macro_rules! cmp {
  ($self_: expr, $cond: ident) => { {
    let before_else = $self_.new_label();
    let after_else = $self_.new_label();
    $self_.method().$cond(before_else);
    $self_.method().bool_const(false);
    $self_.method().goto(after_else);
    $self_.method().label(before_else);
    $self_.method().bool_const(true);
    $self_.method().label(after_else);
  } };
}

pub struct JvmCodeGen {
  class_builder: *mut ClassBuilder,
  method_builder: *mut MethodBuilder,
  main: *const ClassDef,
  break_stack: Vec<u16>,
  label: u16,
  stack_index: u8,
}

trait ToJavaType {
  fn to_java(&self) -> JavaType;

  fn to_java_with_dim(&self) -> (JavaType, u8);
}

// need to first judge whether it is string
// which is regarded ad basic type in decaf
impl ToJavaType for SemanticType {
  fn to_java(&self) -> JavaType {
    match self {
      SemanticType::Basic(name) => match *name {
        "int" => JavaType::Int,
        "void" => JavaType::Void,
        "bool" => JavaType::Boolean,
        "string" => JavaType::Class("java/lang/String"),
        _ => unreachable!(),
      },
      SemanticType::Object(class) => JavaType::Class(unsafe { (**class).name }),
      SemanticType::Array(elem) => JavaType::Array(Box::new(elem.to_java())),
      _ => unreachable!(),
    }
  }

  fn to_java_with_dim(&self) -> (JavaType, u8) {
    match self {
      SemanticType::Array(elem) => {
        let ret = elem.to_java_with_dim();
        (ret.0, ret.1 + 1)
      }
      _ => (self.to_java(), 1)
    }
  }
}

impl JvmCodeGen {
  pub fn new() -> JvmCodeGen {
    JvmCodeGen {
      class_builder: ptr::null_mut(),
      method_builder: ptr::null_mut(),
      main: ptr::null(),
      break_stack: Vec::new(),
      label: 0,
      stack_index: 0,
    }
  }

  pub fn gen(mut self, mut program: Program) {
    self.program(&mut program);
  }

  fn class(&self) -> &mut ClassBuilder {
    unsafe { &mut *self.class_builder }
  }

  fn method(&self) -> &mut MethodBuilder {
    unsafe { &mut *self.method_builder }
  }

  fn store_to_stack(&self, t: &SemanticType, index: u8) {
    handle!(t, self.method().i_store(index), self.method().a_store(index));
  }

  fn load_from_stack(&self, t: &SemanticType, index: u8) {
    handle!(t, self.method().i_load(index), self.method().a_load(index));
  }

  fn new_label(&mut self) -> u16 {
    let ret = self.label;
    self.label += 1;
    ret
  }
}

impl Visitor for JvmCodeGen {
  fn program(&mut self, program: &mut Program) {
    self.main = program.main;
    for class_def in &mut program.class {
      self.class_def(class_def);
    }
  }

  fn class_def(&mut self, class_def: &mut ClassDef) {
    let parent = if let Some(parent) = class_def.parent { parent } else { "java/lang/Object" };
    let mut class_builder =
      ClassBuilder::new(ACC_PUBLIC | if class_def.sealed { ACC_FINAL } else { 0 }
                        , class_def.name, parent);
    self.class_builder = &mut class_builder;

    {
      // generate constructor
      let mut constructor = MethodBuilder::new(&mut class_builder, ACC_PUBLIC, "<init>", &[], &JavaType::Void);
      constructor.a_load(0);
      constructor.invoke_special(parent, "<init>", &[], &JavaType::Void);
      constructor.return_();
      constructor.done(1);
    }

    for field_def in &mut class_def.field {
      self.field_def(field_def);
    }
    class_builder.done().write_to_file(&(class_def.name.to_owned() + ".class"));
    self.class_builder = ptr::null_mut();
  }

  fn method_def(&mut self, method_def: &mut MethodDef) {
    if method_def.class == self.main && method_def.name == "main" {
      method_def.param.insert(0, VarDef {
        loc: method_def.loc,
        name: "args",
        type_: Type { loc: method_def.loc, sem: SemanticType::Array(Box::new(SemanticType::Basic("string"))) },
        scope: &method_def.scope,
        index: 0,
      });
    }
    let argument_types: Vec<JavaType> = method_def.param.iter().map(|var_def| var_def.type_.to_java()).collect();
    let return_type = method_def.ret_t.to_java();
    // in type check, a virtual this is added to the param list
    // but jvm doesn't need it, so take the slice from 1 to end
    let mut method_builder = MethodBuilder::new(self.class(),
                                                ACC_PUBLIC | if method_def.static_ { ACC_STATIC } else { 0 },
                                                method_def.name,
                                                &argument_types[if method_def.static_ { 0 } else { 1 }..],
                                                &return_type);
    self.method_builder = &mut method_builder;
    self.label = 0;
    self.stack_index = 0;
    // this is counted here
    for var_def in &mut method_def.param {
      self.var_def(var_def);
    }
    self.block(&mut method_def.body);
    if &method_def.ret_t.sem == &VOID {
      method_builder.return_();
    }
    method_builder.done(self.stack_index as u16);
    self.method_builder = ptr::null_mut();
  }

  fn var_def(&mut self, var_def: &mut VarDef) {
    match unsafe { (*var_def.scope).kind } {
      ScopeKind::Local(_) | ScopeKind::Parameter(_) => {
        var_def.index = self.stack_index;
        self.stack_index += 1;
      }
      ScopeKind::Class(_) => self.class().define_field(ACC_PUBLIC, var_def.name, &var_def.type_.to_java()),
      _ => unreachable!(),
    }
  }

  fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt { self.stmt(stmt); }
  }

  fn while_(&mut self, while_: &mut While) {
    let before_cond = self.new_label();
    let after_body = self.new_label();
    self.break_stack.push(after_body);
    self.method().label(before_cond);
    self.expr(&mut while_.cond);
    self.method().if_eq(after_body);
    self.block(&mut while_.body);
    self.method().goto(before_cond);
    self.method().label(after_body);
    self.break_stack.pop();
  }

  fn for_(&mut self, for_: &mut For) {
    let before_cond = self.new_label();
    let after_body = self.new_label();
    self.break_stack.push(after_body);
    self.simple(&mut for_.init);
    self.method().label(before_cond);
    self.expr(&mut for_.cond);
    self.method().if_eq(after_body);
    self.block(&mut for_.body);
    self.simple(&mut for_.update);
    self.method().goto(before_cond);
    self.method().label(after_body);
    self.break_stack.pop();
  }

  fn if_(&mut self, if_: &mut If) {
    let before_else = self.new_label();
    let after_else = self.new_label();
    self.expr(&mut if_.cond);
    self.method().if_eq(before_else); // if_eq jump to before_else if stack_top == 0
    self.block(&mut if_.on_true);
    self.method().goto(after_else);
    self.method().label(before_else);
    if let Some(on_false) = &mut if_.on_false { self.block(on_false); }
    self.method().label(after_else);
  }

  fn break_(&mut self, _break: &mut Break) {
    self.method().goto(*self.break_stack.last().unwrap());
  }

  fn return_(&mut self, return_: &mut Return) {
    if let Some(expr) = &mut return_.expr {
      self.expr(expr);
      handle!(expr.get_type(), self.method().i_return(), self.method().a_return());
    } else {
      self.method().return_();
    }
  }

  fn new_class(&mut self, new_class: &mut NewClass) {
    self.method().new_(new_class.name);
    self.method().dup();
    self.method().invoke_special(new_class.name, "<init>", &[], &JavaType::Void);
  }

  fn new_array(&mut self, new_array: &mut NewArray) {
    self.expr(&mut new_array.len);
    // new_array.elem_t is not set during type check, it may still be Named
    match &new_array.type_ {
      SemanticType::Array(elem_t) => handle!(elem_t.as_ref(), self.method().new_int_array(), self.method().new_bool_array(),
                                            self.method().a_new_array(&elem_t.to_java().to_string())),
      _ => unreachable!(),
    }
  }

  fn assign(&mut self, assign: &mut Assign) {
    unsafe {
      match &mut assign.dst {
        Expr::Indexed(indexed) => indexed.for_assign = true,
        Expr::Identifier(identifier) => identifier.for_assign = true,
        _ => unreachable!(),
      }
      self.expr(&mut assign.dst);
      self.expr(&mut assign.src);
      match &assign.dst {
        Expr::Identifier(identifier) => match identifier.symbol {
          Var::VarDef(var_def) => {
            let var_def = &*var_def;
            match (*var_def.scope).kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => self.store_to_stack(&var_def.type_, var_def.index),
              ScopeKind::Class(class) => self.method().put_field((*class).name, var_def.name, &var_def.type_.to_java()),
              _ => unreachable!(),
            }
          }
          Var::VarAssign(var_assign) => self.store_to_stack(&(*var_assign).type_, (*var_assign).index),
        }
        Expr::Indexed(indexed) => handle!(&indexed.type_, self.method().i_a_store(), self.method().b_a_store(), self.method().a_a_store()),
        _ => unreachable!(),
      }
    }
  }

  fn const_(&mut self, const_: &mut Const) {
    match const_ {
      Const::IntConst(int_const) => self.method().int_const(int_const.value),
      Const::BoolConst(bool_const) => self.method().bool_const(bool_const.value),
      Const::StringConst(string_const) => self.method().string_const(&string_const.value),
      Const::Null(_) => self.method().a_const_null(),
      _ => unimplemented!(),
    }
  }

  fn unary(&mut self, unary: &mut Unary) {
    match unary.op {
      Operator::Neg => {
        self.expr(&mut unary.r);
        self.method().i_neg();
      }
      Operator::Not => {
        let true_label = self.new_label();
        let out_label = self.new_label();
        self.expr(&mut unary.r);
        self.method().if_eq(true_label);
        self.method().bool_const(false);
        self.method().goto(out_label);
        self.method().label(true_label);
        self.method().bool_const(true);
        self.method().label(out_label);
      }
      _ => unreachable!()
    }
  }

  fn binary(&mut self, binary: &mut Binary) {
    use super::ast::Operator::*;
    match binary.op {
      And => {
        let out_label = self.new_label();
        let false_label = self.new_label();
        self.expr(&mut binary.l);
        self.method().if_eq(false_label);
        self.expr(&mut binary.r);
        self.method().if_eq(false_label);
        self.method().bool_const(true);
        self.method().goto(out_label);
        self.method().label(false_label);
        self.method().bool_const(false);
        self.method().label(out_label);
      }
      Or => {
        let out_label = self.new_label();
        let true_label = self.new_label();
        self.expr(&mut binary.l);
        self.method().if_ne(true_label);
        self.expr(&mut binary.r);
        self.method().if_ne(true_label);
        self.method().bool_const(false);
        self.method().goto(out_label);
        self.method().label(true_label);
        self.method().bool_const(true);
        self.method().label(out_label);
      }
      _ => {
        self.expr(&mut binary.l);
        self.expr(&mut binary.r);
        match binary.op {
          Add => self.method().i_add(),
          Sub => self.method().i_sub(),
          Mul => self.method().i_mul(),
          Div => self.method().i_div(),
          Mod => self.method().i_rem(),
          Le => cmp!(self, if_i_cmp_le),
          Lt => cmp!(self, if_i_cmp_lt),
          Ge => cmp!(self, if_i_cmp_ge),
          Gt => cmp!(self, if_i_cmp_gt),
          Eq => match binary.l.get_type() {
            SemanticType::Null | SemanticType::Object(_) => cmp!(self, if_a_cmp_eq),
            _ => cmp!(self, if_i_cmp_eq),
          }
          Ne => match binary.l.get_type() {
            SemanticType::Null | SemanticType::Object(_) => cmp!(self, if_a_cmp_ne),
            _ => cmp!(self, if_i_cmp_ne),
          }
          _ => unreachable!(),
        }
      }
    }
  }

  fn call(&mut self, call: &mut Call) {
    unsafe {
      let method = &*call.method;
      if !method.static_ {
        self.expr(if let Some(owner) = &mut call.owner { owner } else { unreachable!() });
      }
      for arg in &mut call.arg {
        self.expr(arg);
      }
      let argument_types: Vec<JavaType> = method.param.iter().map(|var_def| var_def.type_.to_java()).collect();
      let return_type = method.ret_t.to_java();
      if method.static_ {
        self.method().invoke_static((*method.class).name, method.name, &argument_types, &return_type);
      } else {
        self.method().invoke_virtual((*method.class).name, method.name, &argument_types[1..], &return_type);
      }
    }
  }

  fn print(&mut self, print: &mut Print) {
    for print in &mut print.print {
      self.method().get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
      self.expr(print);
      self.method().invoke_virtual("java/io/PrintStream", "print", &[print.get_type().to_java()], &JavaType::Void);
    }
  }

  fn this(&mut self, _this: &mut This) {
    self.method().a_load(0);
  }

  fn indexed(&mut self, indexed: &mut Indexed) {
    self.expr(&mut indexed.arr);
    self.expr(&mut indexed.idx);
    if !indexed.for_assign {
      handle!(&indexed.type_, self.method().i_a_load(), self.method().b_a_load(), self.method().a_a_load());
    }
  }

  fn identifier(&mut self, identifier: &mut Identifier) {
    unsafe {
      if !identifier.for_assign {
        match identifier.symbol {
          Var::VarDef(var_def) => {
            let var_def = &*var_def;
            match (*var_def.scope).kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => self.load_from_stack(&var_def.type_, var_def.index),
              ScopeKind::Class(class) => {
                self.expr(if let Some(owner) = &mut identifier.owner { owner } else { unreachable!() });
                self.method().get_field((*class).name, var_def.name, &var_def.type_.to_java())
              }
              _ => unreachable!(),
            }
          }
          Var::VarAssign(var_assign) => self.load_from_stack(&(*var_assign).type_, (*var_assign).index),
        }
      } else {
        if let Some(owner) = &mut identifier.owner { self.expr(owner); }
      }
    }
  }
}