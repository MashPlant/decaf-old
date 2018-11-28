use super::class::*;
use super::types::*;
use super::writer::*;
use super::class::Instruction::*;

use std::collections::HashMap;
use std::string::ToString;

pub struct ClassBuilder {
  access_flags: u16,
  this_class_index: u16,
  super_class_index: u16,
  constants: Vec<Constant>,
  constant_cache: HashMap<Constant, u16>,
  fields: Vec<Field>,
  methods: Vec<Method>,
}

impl ClassBuilder {
  pub fn new(access_flags: u16, this_class: &str, super_class: &str) -> ClassBuilder {
    let mut builder = ClassBuilder {
      access_flags,
      this_class_index: 0,
      super_class_index: 0,
      constants: Vec::new(),
      constant_cache: HashMap::new(),
      fields: Vec::new(),
      methods: Vec::new(),
    };
    builder.this_class_index = builder.define_class(this_class);
    builder.super_class_index = builder.define_class(super_class);
    builder
  }

  fn push_constant(&mut self, constant: Constant) -> u16 {
    if let Some(index) = self.constant_cache.get(&constant) {
      return *index;
    }
    self.constants.push(constant.clone());
    let ret = self.constants.len() as u16; // 1 indexed
    self.constant_cache.insert(constant, ret);
    ret
  }

  fn define_utf8(&mut self, string: &str) -> u16 {
    self.push_constant(Constant::Utf8(string.to_owned()))
  }

  // only be used to define this_class & super_class
  fn define_class(&mut self, class: &str) -> u16 {
    let name_index = self.define_utf8(class);
    self.push_constant(Constant::Class { name_index })
  }

  fn define_string(&mut self, value: &str) -> u16 {
    let string_index = self.define_utf8(value);
    self.push_constant(Constant::String { string_index })
  }

  fn define_field_ref(&mut self, class: &str, name: &str, field_type: &JavaType) -> u16 {
    let class_index = self.define_class(class);
    let name_and_type_index = self.define_name_and_type(name, &field_type.to_string());
    self.push_constant(Constant::FieldRef { class_index, name_and_type_index })
  }

  fn define_method_ref(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) -> u16 {
    let class_index = self.define_class(class);
    let descriptor = make_method_type(argument_types, return_type);
    let name_and_type_index = self.define_name_and_type(name, &descriptor);
    self.push_constant(Constant::MethodRef { class_index, name_and_type_index })
  }

  fn define_name_and_type(&mut self, name: &str, descriptor: &str) -> u16 {
    let name_index = self.define_utf8(name);
    let descriptor_index = self.define_utf8(&descriptor);
    self.push_constant(Constant::NameAndType { name_index, descriptor_index })
  }

  fn define_int(&mut self, value: i32) -> u16 {
    self.push_constant(Constant::Integer { bytes: value as u32 })
  }

  pub fn done(self) -> Class {
    Class {
      constant_pool: self.constants,
      access_flags: self.access_flags,
      this_class: self.this_class_index,
      super_class: self.super_class_index,
      fields: self.fields,
      methods: self.methods,
    }
  }
}

pub struct MethodBuilder<'a> {
  class_builder: &'a mut ClassBuilder,
  access_flags: u16,
  name_index: u16,
  descriptor_index: u16,
  //  instructions: Vec<(u16, DelayedInstruction)>,
  code: Vec<u8>,
  // map label to the index of code with the label
  labels: HashMap<u16, u16>,
  // map index of code to label, index points to the high byte of code need to be filled with the label
  fills: Vec<(u16, u16)>,
  cur_stack: u16,
  max_stack: u16,
}

impl<'a> MethodBuilder<'a> {
  pub fn new(class_builder: &'a mut ClassBuilder,
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
      code: Vec::new(),
      labels: HashMap::new(),
      fills: Vec::new(),
      cur_stack: 0,
      max_stack: 0,
    }
  }

  fn inc_stack(&mut self) {
    self.cur_stack += 1;
    if self.cur_stack > self.max_stack {
      self.max_stack = self.cur_stack
    }
  }

  fn dec_stack(&mut self) {
    self.cur_stack -= 1;
  }

  fn dec_stack_n(&mut self, n: u16) {
    self.cur_stack -= n;
  }

  pub fn int_const(&mut self, value: i32) {
    match value {
      -128...127 => self.b_i_push(value as i8),
      -32768...32767 => self.s_i_push(value as i16),
      x => self.ldc(self.class_builder.define_int(x) as u8),
    };
    self.inc_stack();
  }

  pub fn bool_const(&mut self, value: bool) {
    if value { self.i_const_1(); } else { self.i_const_1(); }
  }

  pub fn string_const(&mut self, value: &str) {
    self.ldc(self.class_builder.define_string(value) as u8);
    self.inc_stack();
  }

  pub fn a_const_null(&mut self) {
    self.push_code(AConstNull);
    self.inc_stack();
  }

  pub fn i_const_0(&mut self) {
    self.push_code(IConst0);
    self.inc_stack();
  }

  pub fn i_const_1(&mut self) {
    self.push_code(IConst1);
    self.inc_stack();
  }

  // [-128, 127]
  pub fn b_i_push(&mut self, value: i8) {
    self.push_code(BIPush(value as u8));
    self.inc_stack();
  }

  // [-32768, 32767] \ [-128, 127]
  pub fn s_i_push(&mut self, value: i16) {
    self.push_code(SIPush(value as u16));
    self.inc_stack();
  }

  pub fn ldc(&mut self, index: u8) {
    self.push_code(Ldc(index));
    self.inc_stack();
  }

  pub fn i_load(&mut self, stack_index: u8) {
    self.push_code(ILoad(stack_index));
    self.inc_stack();
  }

  pub fn a_load(&mut self, stack_index: u8) {
    self.push_code(ALoad(stack_index));
    self.inc_stack();
  }

  pub fn i_a_load(&mut self) {
    self.push_code(IALoad);
    self.dec_stack();
  }

  pub fn a_a_load(&mut self) {
    self.push_code(AALoad);
    self.dec_stack();
  }

  pub fn b_a_load(&mut self) {
    self.push_code(BALoad);
    self.dec_stack();
  }

  pub fn i_a_store(&mut self) {
    self.push_code(IAStore);
    self.dec_stack();
  }

  pub fn a_a_store(&mut self) {
    self.push_code(AAStore);
    self.dec_stack();
  }

  pub fn b_a_store(&mut self) {
    self.push_code(BAStore);
    self.dec_stack();
  }

  pub fn i_add(&mut self) {
    self.push_code(IAdd);
    self.dec_stack();
  }

  pub fn i_sub(&mut self) {
    self.push_code(ISub);
    self.dec_stack();
  }

  pub fn i_mul(&mut self) {
    self.push_code(IMul);
    self.dec_stack();
  }

  pub fn i_div(&mut self) {
    self.push_code(IDiv);
    self.dec_stack();
  }

  pub fn i_rem(&mut self) {
    self.push_code(IRem);
    self.dec_stack();
  }

  pub fn i_neg(&mut self) {
    self.push_code(INeg);
  }

  pub fn if_eq(&mut self, label: u16) {
    self.delay_code(label, IfEq(0));
    self.dec_stack();
  }

  pub fn if_ne(&mut self, label: u16) {
    self.delay_code(label, IfNe(0));
    self.dec_stack();
  }

  pub fn if_lt(&mut self, label: u16) {
    self.delay_code(label, IfLt(0));
    self.dec_stack();
  }

  pub fn if_ge(&mut self, label: u16) {
    self.delay_code(label, IfGe(0));
    self.dec_stack();
  }

  pub fn if_gt(&mut self, label: u16) {
    self.delay_code(label, IfGt(0));
    self.dec_stack();
  }

  pub fn if_le(&mut self, label: u16) {
    self.delay_code(label, IfLe(0));
    self.dec_stack();
  }

  pub fn if_i_cmp_eq(&mut self, label: u16) {
    self.delay_code(label, IfICmpEq(0));
    self.dec_stack_n(2);
  }

  pub fn if_i_cmp_ne(&mut self, label: u16) {
    self.delay_code(label, IfICmpNe(0));
    self.dec_stack_n(2);
  }

  pub fn if_i_cmp_lt(&mut self, label: u16) {
    self.delay_code(label, IfICmpLt(0));
    self.dec_stack_n(2);
  }

  pub fn if_i_cmp_ge(&mut self, label: u16) {
    self.delay_code(label, IfICmpGe(0));
    self.dec_stack_n(2);
  }

  pub fn if_i_cmp_gt(&mut self, label: u16) {
    self.delay_code(label, IfICmpGt(0));
    self.dec_stack_n(2);
  }

  pub fn if_i_cmp_le(&mut self, label: u16) {
    self.delay_code(label, IfICmpLe(0));
    self.dec_stack_n(2);
  }

  pub fn if_a_cmp_eq(&mut self, label: u16) {
    self.delay_code(label, IfACmpEq(0));
    self.dec_stack_n(2);
  }

  pub fn if_a_cmp_ne(&mut self, label: u16) {
    self.delay_code(label, IfACmpNe(0));
    self.dec_stack_n(2);
  }

  pub fn goto(&mut self, label: u16) {
    self.delay_code(label, Goto(0));
  }

  pub fn i_return(&mut self) {
    self.push_code(IReturn);
    self.dec_stack();
  }

  pub fn a_return(&mut self) {
    self.push_code(AReturn);
    self.dec_stack();
  }

  pub fn return_(&mut self) {
    self.push_code(Return);
  }

  pub fn get_static(&mut self, class: &str, name: &str, field_type: &JavaType) {
    let index = self.class_builder.define_field_ref(class, name, field_type);
    self.push_code(GetStatic(index));
    self.inc_stack();
  }

  pub fn get_field(&mut self, class: &str, name: &str, field_type: &JavaType) {
    let index = self.class_builder.define_field_ref(class, name, field_type);
    self.push_code(GetField(index));
  }

  pub fn put_field(&mut self, class: &str, name: &str, field_type: &JavaType) {
    let index = self.class_builder.define_field_ref(class, name, field_type);
    self.push_code(PutField(index));
    self.dec_stack();
  }

  pub fn invoke_virtual(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) {
    let index = self.class_builder.define_method_ref(class, name, argument_types, return_type);
    self.push_code(InvokeVirtual(index));
    self.dec_stack_n(argument_types.len() as u16 + 1);
    if *return_type != JavaType::Void { self.inc_stack(); }
  }

  pub fn invoke_special(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) {
    let index = self.class_builder.define_method_ref(class, name, argument_types, return_type);
    self.push_code(InvokeSpecial(index));
    self.dec_stack_n(argument_types.len() as u16 + 1);
    if *return_type != JavaType::Void { self.inc_stack(); }
  }

  pub fn invoke_static(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) {
    let index = self.class_builder.define_method_ref(class, name, argument_types, return_type);
    self.push_code(InvokeStatic(index));
    self.dec_stack_n(argument_types.len() as u16);
    if *return_type != JavaType::Void { self.inc_stack(); }
  }

  pub fn new_(&mut self, class: &str) {}

  pub fn new_array(&mut self, class: &str) {}

  pub fn a_new_array(&mut self, class: &str) {}

  pub fn array_length(&mut self) {
    self.push_code(ArrayLength);
  }

  pub fn label(&mut self, label: u16) {
    self.labels.insert(label, self.code.len() as u16);
  }

  fn push_code(&mut self, instruction: Instruction) {
    instruction.write_to(&mut self.code);
  }

  fn delay_code(&mut self, label: u16, instruction: Instruction) {
    instruction.write_to(&mut self.code);
    self.fills.push((self.code.len() as u16 - 2, label));
  }

  pub fn done(mut self) {
    if self.cur_stack != 0 {
      println!("Warning: stack depth at the end of a method should be 0, but is {} instead", self.cur_stack);
    }

    for (index, label) in self.fills {
      let label = *self.labels.get(&label).unwrap() - index + 1;
      self.code[index as usize] = (label >> 8) as u8;
      self.code[index as usize + 1] = label as u8;
    }

    let attribute_name_index = self.class_builder.define_utf8("Code");
    let code = Code {
      attribute_name_index,
      max_stack: self.max_stack,
      max_locals: 1,
      code: self.code,
    };

    self.class_builder.methods.push(Method {
      access_flags: self.access_flags,
      name_index: self.name_index,
      descriptor_index: self.descriptor_index,
      code,
    });
  }
}