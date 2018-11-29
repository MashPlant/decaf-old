extern crate backend;

use backend::*;
use backend::jvm::class::{ACC_PUBLIC, ACC_STATIC, ACC_FINAL};
use backend::jvm::writer::*;
use super::ast::*;
use super::types::*;
use super::symbol::*;

use std::ptr;

pub struct JvmCodeGen {
  class_builder: *mut ClassBuilder,
  method_builder: *mut MethodBuilder,
  main: *const ClassDef,
  stack_index: u8,
}

trait ToJavaType {
  fn to_java_type(&self) -> JavaType;
}

// need to first judge whether it is string
// which is regarded ad basic type in decaf
impl ToJavaType for SemanticType {
  fn to_java_type(&self) -> JavaType {
    match self {
      SemanticType::Basic(name) => match *name {
        "int" => JavaType::Int,
        "void" => JavaType::Void,
        "bool" => JavaType::Boolean,
        "string" => JavaType::Class("java/lang/String"),
        _ => unreachable!(),
      },
      SemanticType::Object(class) => JavaType::Class(unsafe { (**class).name }),
      SemanticType::Array(elem) => JavaType::Array(Box::new(elem.to_java_type())),
      _ => unreachable!(),
    }
  }
}

impl JvmCodeGen {
  pub fn new() -> JvmCodeGen {
    JvmCodeGen {
      class_builder: ptr::null_mut(),
      method_builder: ptr::null_mut(),
      main: ptr::null(),
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
    match t {
      SemanticType::Basic(name) => match *name {
        "int" | "bool" => self.method().i_store(index),
        "string" => self.method().a_store(index),
        _ => unreachable!(),
      },
      SemanticType::Object(_) => self.method().a_store(index),
      _ => unreachable!(),
    }
  }

  fn load_from_stack(&self, t: &SemanticType, index: u8) {
    match t {
      SemanticType::Basic(name) => match *name {
        "int" | "bool" => self.method().i_load(index),
        "string" => self.method().a_load(index),
        _ => unreachable!(),
      },
      SemanticType::Object(_) => self.method().a_load(index),
      _ => unreachable!(),
    }
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
    let mut class_builder =
      ClassBuilder::new(ACC_PUBLIC | if class_def.sealed { ACC_FINAL } else { 0 }
                        , class_def.name
                        , if let Some(parent) = class_def.parent { parent } else { "java/lang/Object" });
    self.class_builder = &mut class_builder;
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
    let access_flags = ACC_PUBLIC | if method_def.static_ { ACC_STATIC } else { 0 };
    let argument_types: Vec<JavaType> = method_def.param.iter().map(|var_def| var_def.type_.to_java_type()).collect();
    let return_type = method_def.ret_t.to_java_type();
    let mut method_builder = MethodBuilder::new(self.class(),
                                                access_flags, method_def.name, &argument_types, &return_type);
    self.method_builder = &mut method_builder;
    self.stack_index = method_def.param.len() as u8;
    self.block(&mut method_def.body);
    // TODO: reject the code with non-void return type which doesn't return in some branches
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
      ScopeKind::Class(_) => self.class().define_field(ACC_PUBLIC, var_def.name, &var_def.type_.to_java_type()),
      _ => unreachable!(),
    }
  }

  fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt { self.stmt(stmt); }
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
              ScopeKind::Class(class) => self.method().put_field((*class).name, var_def.name, &var_def.type_.to_java_type()),
              _ => unreachable!(),
            }
          }
          Var::VarAssign(var_assign) => self.store_to_stack(&(*var_assign).type_, (*var_assign).index),
        }
        Expr::Indexed(indexed) => {
          match &indexed.type_ {
            SemanticType::Basic(name) => match *name {
              "int" => self.method().i_a_store(),
              "bool" => self.method().b_a_store(),
              "string" => self.method().a_a_store(),
              _ => unreachable!(),
            },
            SemanticType::Object(_) => self.method().a_a_store(),
            _ => unreachable!(),
          }
        }
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

  fn binary(&mut self, binary: &mut Binary) {
    use super::ast::Operator::*;
    match binary.op {
      Add | Sub | Mul | Div | Mod => {
        self.expr(&mut binary.l);
        self.expr(&mut binary.r);
        match binary.op {
          Add => self.method().i_add(),
          Sub => self.method().i_sub(),
          Mul => self.method().i_mul(),
          Div => self.method().i_div(),
          Mod => self.method().i_rem(),
          _ => unreachable!(),
        }
      }
      _ => unimplemented!(),
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
      let argument_types: Vec<JavaType> = method.param.iter().map(|var_def| var_def.type_.to_java_type()).collect();
      let return_type = method.ret_t.to_java_type();
      if method.static_ {
        self.method().invoke_static((*method.class).name, method.name, &argument_types, &return_type);
      } else {
        self.method().invoke_virtual((*method.class).name, method.name, &argument_types, &return_type);
      }
    }
  }

  fn print(&mut self, print: &mut Print) {
    for print in &mut print.print {
      self.method().get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
      self.expr(print);
      self.method().invoke_virtual("java/io/PrintStream", "println", &[print.get_type().to_java_type()], &JavaType::Void);
    }
  }

  fn indexed(&mut self, indexed: &mut Indexed) {
    self.expr(&mut indexed.arr);
    self.expr(&mut indexed.idx);
    if !indexed.for_assign {
      match &indexed.type_ {
        SemanticType::Basic(name) => match *name {
          "int" => self.method().i_a_load(),
          "bool" => self.method().b_a_load(),
          "string" => self.method().a_a_load(),
          _ => unreachable!(),
        },
        SemanticType::Object(_) => self.method().a_a_load(),
        _ => unreachable!(),
      }
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
              ScopeKind::Class(class) => self.method().get_field((*class).name, var_def.name, &var_def.type_.to_java_type()),
              _ => unreachable!(),
            }
          }
          Var::VarAssign(var_assign) => self.load_from_stack(&(*var_assign).type_, (*var_assign).index),
        }
      }
    }
  }
}