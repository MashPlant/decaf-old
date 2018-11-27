use super::class::*;
use super::types::*;

use std::collections::HashMap;

pub struct ClassBuilder {
  access_flags: u16,
  this_class_index: u16,
  super_class_index: u16,
  constants: Vec<Constant>,
  methods: Vec<Method>,
}

impl ClassBuilder {
  pub fn new(access_flags: u16, this_class: &str, super_class: &str) -> ClassBuilder {
    let mut builder = ClassBuilder {
      access_flags,
      this_class_index: 0,
      super_class_index: 0,
      constants: Vec::new(),
      methods: Vec::new(),
    };
    builder.this_class_index = builder.define_class(this_class);
    builder.super_class_index = builder.define_class(super_class);
    builder
  }

  fn push_constant(&mut self, constant: Constant) -> u16 {
    self.constants.push(constant);
    self.constants.len() as u16
  }

  fn define_utf8(&mut self, string: &str) -> u16 {
    self.push_constant(Constant::Utf8(string.to_owned()))
  }

  fn define_class(&mut self, class: &str) -> u16 {
    let name_index = self.define_utf8(class);
    self.push_constant(Constant::Class { name_index })
  }

  fn define_string(&mut self, value: &str) -> u16 {
    let string_index = self.define_utf8(value);
    self.push_constant(Constant::String { string_index })
  }
}

#[derive(Debug)]
pub enum IntermediateInstruction<'a> {
  Ready(Instruction),
  Waiting(&'a str, Instruction),
}

pub struct MethodBuilder<'a> {
  class_builder: &'a mut ClassBuilder,
  access_flags: u16,
  name_index: u16,
  descriptor_index: u16,
  instructions: Vec<(u16, IntermediateInstruction<'a>)>,
  labels: HashMap<String, u16>,
  cur_stack_depth: u16,
  max_stack_depth: u16,
}

impl<'a> MethodBuilder<'a> {
  fn new(class_builder: &'a mut ClassBuilder,
             access_flags: u16, name: &str,
             argument_types: &[JavaType],
             return_type: &JavaType) -> MethodBuilder<'a> {
    let name_index = class_builder.define_utf8(name);
    let descriptor = make_method_type(argument_types, return_type);
    let descriptor_index = class_builder.define_utf8(&descriptor);
    MethodBuilder {
      class_builder,
      access_flags,
      name_index,
      descriptor_index,
      instructions: Vec::new(),
      labels: HashMap::new(),
      cur_stack_depth: 0,
      max_stack_depth: 0,
    }
  }
}