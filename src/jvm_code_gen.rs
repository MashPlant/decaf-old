extern crate backend;

use backend::*;
use backend::jvm::class::{ACC_PUBLIC, ACC_STATIC, ACC_FINAL};
use backend::jvm::writer::*;
use super::ast::*;
use super::types::*;
use super::symbol::*;

use std::ptr;

struct JvmCodeGen {
  class_builder: *mut ClassBuilder,
  method_builder: *mut MethodBuilder,
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
  fn new() -> JvmCodeGen {
    JvmCodeGen {
      class_builder: ptr::null_mut(),
      method_builder: ptr::null_mut(),
      stack_index: 0,
    }
  }

  fn class(&self) -> &mut ClassBuilder {
    unsafe { &mut *self.class_builder }
  }

  fn method(&self) -> &mut MethodBuilder {
    unsafe { &mut *self.method_builder }
  }
}

impl Visitor for JvmCodeGen {
  fn program(&mut self, program: &mut Program) {
    for class_def in &mut program.class {
      self.class_def(class_def);
    }
  }

  fn class_def(&mut self, class_def: &mut ClassDef) {
    let mut class_builder = ClassBuilder::new(ACC_PUBLIC, class_def.name,
                                              if let Some(parent) = class_def.parent { parent } else { "java/lang/Object" });
    self.class_builder = &mut class_builder;
    for field_def in &mut class_def.field {
      self.field_def(field_def);
    }
    class_builder.done().write_to_file(&(class_def.name.to_owned() + ".class"));
    self.class_builder = ptr::null_mut();
  }

  fn method_def(&mut self, method_def: &mut MethodDef) {
    let access_flags = ACC_PUBLIC | if method_def.static_ { ACC_STATIC } else { 0 };
    let argument_types: Vec<JavaType> = method_def.param.iter().map(|var_def| var_def.type_.to_java_type()).collect();
    let return_type = method_def.ret_t.to_java_type();
    let mut method_builder = MethodBuilder::new(self.class(),
                                                access_flags, method_def.name, &argument_types, &return_type);
    self.method_builder = &mut method_builder;
    self.stack_index = if method_def.static_ { 0 } else { 1 };
    self.block(&mut method_def.body);
    self.method_builder = ptr::null_mut();
  }

  fn var_def(&mut self, var_def: &mut VarDef) {
    match unsafe { (*var_def.scope).kind } {
      ScopeKind::Local(_) | ScopeKind::Parameter(_) => var_def.index = self.stack_index + 1,
      ScopeKind::Class(_) => self.class().define_field(ACC_PUBLIC, var_def.name, &var_def.type_.to_java_type()),
      _ => unreachable!(),
    }
  }

  fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt { self.stmt(stmt); }
  }


  fn assign(&mut self, assign: &mut Assign) {
    if let Expr::Indexed(indexed) = &mut assign.dst {
      indexed.for_assign = true;
    }
    self.expr(&mut assign.dst);
    self.expr(&mut assign.src);
    match &assign.dst {
      Expr::Identifier(identifier) => {

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
}