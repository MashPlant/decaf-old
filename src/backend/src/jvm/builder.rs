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

  pub fn define_field(&mut self, access_flags: u16, name: &str, field_type: &JavaType) {
    let name_index = self.define_utf8(name);
    let descriptor = field_type.to_string();
    let descriptor_index = self.define_utf8(&descriptor);
    self.fields.push(Field { access_flags, name_index, descriptor_index });
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

pub struct MethodBuilder {
  class_builder: *mut ClassBuilder,
  access_flags: u16,
  name_index: u16,
  descriptor_index: u16,
  code: Vec<u8>,
  // map label to the index of code with the label
  labels: HashMap<u16, u16>,
  // map index of code to label, index points to the high byte of code need to be filled with the label
  fills: Vec<(u16, u16)>,
  cur_stack: u16,
  max_stack: u16,
  // only for debug
  instructions: Vec<Instruction>,
}

impl MethodBuilder {
  pub fn new(class_builder: &mut ClassBuilder,
             access_flags: u16, name: &str,
             argument_types: &[JavaType],
             return_type: &JavaType) -> MethodBuilder {
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
      instructions: Vec::new(),
    }
  }

  fn builder(&self) -> &mut ClassBuilder {
    unsafe { &mut *self.class_builder }
  }

  // some wrapper function for convenience
  pub fn int_const(&mut self, value: i32) {
    match value {
      -1 => self.push_code(IConstM1),
      0 => self.push_code(IConst0),
      1 => self.push_code(IConst1),
      2 => self.push_code(IConst2),
      3 => self.push_code(IConst3),
      4 => self.push_code(IConst4),
      5 => self.push_code(IConst5),
      -128...127 => self.push_code(BIPush(value as u8)),
      -32768...32767 => self.push_code(SIPush(value as u16)),
      _ => {
        let index = self.builder().define_int(value);
        self.ldc(index);
      }
    };
    self.inc_stack();
  }

  pub fn bool_const(&mut self, value: bool) {
    self.push_code(if value { IConst1 } else { IConst0 });
    self.inc_stack();
  }

  pub fn string_const(&mut self, value: &str) {
    let index = self.builder().define_string(value);
    self.ldc(index);
    self.inc_stack();
  }

  pub fn new_bool_array(&mut self) {
    self.new_array(4);
  }

  pub fn new_int_array(&mut self) {
    self.new_array(10);
  }

  pub fn label(&mut self, label: u16) {
    self.labels.insert(label, self.code.len() as u16);
  }

  // some instructions are not implemented(generate by wrapper function)
  // some instructions are merged

  pub fn a_const_null(&mut self) {
    self.push_code(AConstNull);
    self.inc_stack();
  }

  // stack is not inc-ed!!!
  fn ldc(&mut self, index: u16) {
    match index {
      0...255 => self.push_code(Ldc(index as u8)),
      256...65535 => self.push_code(LdcW(index)),
      _ => unreachable!(),
    };
  }

  pub fn i_load(&mut self, stack_index: u8) {
    self.push_code(match stack_index {
      0 => ILoad0,
      1 => ILoad1,
      2 => ILoad2,
      3 => ILoad3,
      _ => ILoad(stack_index),
    });
    self.inc_stack();
  }

  pub fn a_load(&mut self, stack_index: u8) {
    self.push_code(match stack_index {
      0 => ALoad0,
      1 => ALoad1,
      2 => ALoad2,
      3 => ALoad3,
      _ => ALoad(stack_index),
    });
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

  pub fn i_store(&mut self, stack_index: u8) {
    self.push_code(match stack_index {
      0 => IStore0,
      1 => IStore1,
      2 => IStore2,
      3 => IStore3,
      _ => IStore(stack_index),
    });
    self.dec_stack();
  }

  pub fn a_store(&mut self, stack_index: u8) {
    self.push_code(match stack_index {
      0 => AStore0,
      1 => AStore1,
      2 => AStore2,
      3 => AStore3,
      _ => AStore(stack_index),
    });
    self.dec_stack();
  }

  pub fn i_a_store(&mut self) {
    self.push_code(IAStore);
    self.dec_stack_n(3);
  }

  pub fn a_a_store(&mut self) {
    self.push_code(AAStore);
    self.dec_stack_n(3);
  }

  pub fn b_a_store(&mut self) {
    self.push_code(BAStore);
    self.dec_stack_n(3);
  }

  pub fn dup(&mut self) {
    self.push_code(Dup);
    self.inc_stack();
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
    let index = self.builder().define_field_ref(class, name, field_type);
    self.push_code(GetStatic(index));
    self.inc_stack();
  }

  pub fn get_field(&mut self, class: &str, name: &str, field_type: &JavaType) {
    let index = self.builder().define_field_ref(class, name, field_type);
    self.push_code(GetField(index));
  }

  pub fn put_field(&mut self, class: &str, name: &str, field_type: &JavaType) {
    let index = self.builder().define_field_ref(class, name, field_type);
    self.push_code(PutField(index));
    self.dec_stack();
  }

  pub fn invoke_virtual(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) {
    let index = self.builder().define_method_ref(class, name, argument_types, return_type);
    self.push_code(InvokeVirtual(index));
    self.dec_stack_n(argument_types.len() as u16 + 1);
    if *return_type != JavaType::Void { self.inc_stack(); }
  }

  pub fn invoke_special(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) {
    let index = self.builder().define_method_ref(class, name, argument_types, return_type);
    self.push_code(InvokeSpecial(index));
    self.dec_stack_n(argument_types.len() as u16 + 1);
    if *return_type != JavaType::Void { self.inc_stack(); }
  }

  pub fn invoke_static(&mut self, class: &str, name: &str, argument_types: &[JavaType], return_type: &JavaType) {
    let index = self.builder().define_method_ref(class, name, argument_types, return_type);
    self.push_code(InvokeStatic(index));
    self.dec_stack_n(argument_types.len() as u16);
    if *return_type != JavaType::Void { self.inc_stack(); }
  }

  pub fn new_(&mut self, class: &str) {
    let index = self.builder().define_class(class);
    self.push_code(New(index));
    self.inc_stack();
  }

  // a_type can only be int(10) / bool(4) in decaf
  pub fn new_array(&mut self, a_type: u8) {
    self.push_code(NewArray(a_type));
  }

  pub fn a_new_array(&mut self, class: &str) {
    let index = self.builder().define_class(class);
    self.push_code(ANewArray(index));
  }

  pub fn array_length(&mut self) {
    self.push_code(ArrayLength);
  }

  pub fn check_cast(&mut self, class: &str) {
    let index = self.builder().define_class(class);
    self.push_code(CheckCast(index));
  }

  pub fn instance_of(&mut self, class: &str) {
    let index = self.builder().define_class(class);
    self.push_code(InstanceOf(index));
  }

  // class is a array object, instead of the element object
  // e.g.: multi_a_new_array("[[I", 2) means create a 2-dim int array
  // this take 2 int from stack, and put 1 array ref to the stack
  pub fn multi_a_new_array(&mut self, class: &str, dim: u8) {
    let index = self.builder().define_class(class);
    self.push_code(MultiANewArray(index, dim));
    self.dec_stack_n(dim as u16 - 1);
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

  fn push_code(&mut self, instruction: Instruction) {
    instruction.write_to(&mut self.code);
    self.instructions.push(instruction);
  }

  fn delay_code(&mut self, label: u16, instruction: Instruction) {
    instruction.write_to(&mut self.code);
    self.fills.push((self.code.len() as u16 - 2, label));
    self.instructions.push(instruction);
  }

  pub fn done(self, max_locals: u16) {
    let MethodBuilder { class_builder, access_flags, name_index, descriptor_index, mut code, labels, fills, cur_stack: _, max_stack, instructions: _ } = self;

    for (index, label) in fills {
      // rust thinks it inappropriate to have unsigned int overflow
      let label = (*labels.get(&label).unwrap() as i16 - index as i16 + 1) as u16;
      code[index as usize] = (label >> 8) as u8;
      code[index as usize + 1] = label as u8;
    }

    let attribute_name_index = unsafe { (*class_builder).define_utf8("Code") };
    let code = Code {
      attribute_name_index,
      max_stack,
      max_locals,
      code,
    };

    unsafe { (*class_builder).methods.push(Method { access_flags, name_index, descriptor_index, code }) };
  }
}