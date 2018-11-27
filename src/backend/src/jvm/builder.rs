use super::class::*;
use super::types::*;
use super::writer::*;

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
pub enum DelayedInstruction {
  Ok(Instruction),
  Unfilled(u16, Instruction),
}

pub struct MethodBuilder<'a> {
  class_builder: &'a mut ClassBuilder,
  access_flags: u16,
  name_index: u16,
  descriptor_index: u16,
  instructions: Vec<(u16, DelayedInstruction)>,
  labels: HashMap<u16, u16>,
  // map label to the index of instructions with the label
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

  pub fn iconstm1(&mut self) {
    self.push_instruction(Instruction::IconstM1);
    self.increase_stack_depth();
  }

  pub fn iconst0(&mut self) {
    self.push_instruction(Instruction::Iconst0);
    self.increase_stack_depth();
  }

  pub fn iconst1(&mut self) {
    self.push_instruction(Instruction::Iconst1);
    self.increase_stack_depth();
  }

  pub fn iconst2(&mut self) {
    self.push_instruction(Instruction::Iconst2);
    self.increase_stack_depth();
  }

  pub fn iconst3(&mut self) {
    self.push_instruction(Instruction::Iconst3);
    self.increase_stack_depth();
  }

  pub fn iconst4(&mut self) {
    self.push_instruction(Instruction::Iconst4);
    self.increase_stack_depth();
  }

  pub fn iconst5(&mut self) {
    self.push_instruction(Instruction::Iconst5);
    self.increase_stack_depth();
  }

  pub fn bipush(&mut self, value: i8) {
    self.push_instruction(Instruction::Bipush(value as u8));
    self.increase_stack_depth();
  }

  pub fn load_constant(&mut self, value: &str) {
    let string_index = self.classfile.define_string(value);
    if string_index > ::std::u8::MAX as u16 {
      panic!("Placed a constant in too high of an index: {}", string_index)
    }
    self.push_instruction(Instruction::LoadConstant(string_index as u8));
    self.increase_stack_depth();
  }

  pub fn aload0(&mut self) {
    self.push_instruction(Instruction::Aload0);
    self.increase_stack_depth();
  }

  pub fn aload1(&mut self) {
    self.push_instruction(Instruction::Aload1);
    self.increase_stack_depth();
  }

  pub fn aload2(&mut self) {
    self.push_instruction(Instruction::Aload2);
    self.increase_stack_depth();
  }

  pub fn aload3(&mut self) {
    self.push_instruction(Instruction::Aload3);
    self.increase_stack_depth();
  }

  pub fn aaload(&mut self) {
    self.push_instruction(Instruction::Aaload);
    self.decrease_stack_depth();
  }

  pub fn iadd(&mut self) {
    self.push_instruction(Instruction::Iadd);
    self.decrease_stack_depth();
  }

  pub fn ifeq(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfEq(0));
    self.decrease_stack_depth();
  }

  pub fn ifne(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfNe(0));
    self.decrease_stack_depth();
  }

  pub fn iflt(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfLt(0));
    self.decrease_stack_depth();
  }

  pub fn ifge(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfGe(0));
    self.decrease_stack_depth();
  }

  pub fn ifgt(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfGt(0));
    self.decrease_stack_depth();
  }

  pub fn ifle(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfLe(0));
    self.decrease_stack_depth();
  }

  pub fn if_icmp_eq(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfIcmpEq(0));
    self.decrease_stack_depth_by(2);
  }

  pub fn if_icmp_ne(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfIcmpNe(0));
    self.decrease_stack_depth_by(2);
  }

  pub fn if_icmp_lt(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfIcmpLt(0));
    self.decrease_stack_depth_by(2);
  }

  pub fn if_icmp_ge(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfIcmpGe(0));
    self.decrease_stack_depth_by(2);
  }

  pub fn if_icmp_gt(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfIcmpGt(0));
    self.decrease_stack_depth_by(2);
  }

  pub fn if_icmp_le(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::IfIcmpLe(0));
    self.decrease_stack_depth_by(2);
  }

  pub fn goto(&mut self, label: &'a str) {
    self.delay_instruction(label, Instruction::Goto(0));
  }

  pub fn do_return(&mut self) {
    self.push_instruction(Instruction::Return);
  }

  pub fn get_static(&mut self, class: &str, name: &str, argument_type: &Java) {
    let fieldref_index = self.classfile.define_fieldref(class, name, argument_type);
    self.push_instruction(Instruction::GetStatic(fieldref_index));
    self.increase_stack_depth();
  }

  pub fn invoke_virtual(&mut self, class: &str, name: &str, argument_types: &[Java], return_type: &Java) {
    let methodref_index = self.classfile.define_methodref(class, name, argument_types, return_type);
    self.push_instruction(Instruction::InvokeVirtual(methodref_index));
    self.decrease_stack_depth_by(argument_types.len() as u8 + 1);
    if *return_type != Java::Void { self.increase_stack_depth(); }
  }

  pub fn invoke_special(&mut self, class: &str, name: &str, argument_types: &[Java], return_type: &Java) {
    let methodref_index = self.classfile.define_methodref(class, name, argument_types, return_type);
    self.push_instruction(Instruction::InvokeSpecial(methodref_index));
    self.decrease_stack_depth_by(argument_types.len() as u8 + 1);
    if *return_type != Java::Void { self.increase_stack_depth(); }
  }

  pub fn invoke_static(&mut self, class: &str, name: &str, argument_types: &[Java], return_type: &Java) {
    let methodref_index = self.classfile.define_methodref(class, name, argument_types, return_type);
    self.push_instruction(Instruction::InvokeStatic(methodref_index));
    self.decrease_stack_depth_by(argument_types.len() as u8);
    if *return_type != Java::Void { self.increase_stack_depth(); }
  }

  pub fn array_length(&mut self) {
    self.push_instruction(Instruction::ArrayLength);
  }

  pub fn label(&mut self, name: &str) {
    self.labels.insert(name.to_owned(), self.stack_index);

    // create a stack map table entry
    let offset = match self.last_stack_frame_index {
      Some(i) => self.stack_index - i - 1,
      None => self.stack_index
    };
    let frame = if offset > ::std::u8::MAX as u16 {
      StackMapFrame::SameFrameExtended(offset)
    } else {
      StackMapFrame::SameFrame(offset as u8)
    };
    self.stack_frames.push(frame);
    self.last_stack_frame_index = Some(self.stack_index);
  }

  fn push_instruction(&mut self, instruction: Instruction) {
    let index = self.stack_index;
    self.stack_index += instruction.size() as u16;
    self.instructions.push((index, IntermediateInstruction::Ready(instruction)));
  }

  fn delay_instruction(&mut self, label: &'a str, instruction: Instruction) {
    let index = self.stack_index;
    self.stack_index += instruction.size() as u16;
    self.instructions.push((index, IntermediateInstruction::Waiting(label, instruction)));
  }

  fn increase_stack_depth(&mut self) {
    self.curr_stack_depth += 1;
    if self.curr_stack_depth > self.max_stack_depth {
      self.max_stack_depth = self.curr_stack_depth;
    }
  }

  fn decrease_stack_depth(&mut self) {
    self.curr_stack_depth -= 1;
  }

  fn decrease_stack_depth_by(&mut self, n: u8) {
    self.curr_stack_depth -= n as u16;
  }

  pub fn done(self) {
    if self.curr_stack_depth != 0 {
      println!("Warning: stack depth at the end of a method should be 0, but is {} instead", self.curr_stack_depth);
    }

    let classfile = self.classfile;
    let labels = self.labels;
    let real_instructions = self.instructions.into_iter().map(|(pos, ir)| match ir {
      IntermediateInstruction::Ready(i) => i,
      IntermediateInstruction::Waiting(l, i) => {
        let label_pos = labels.get(l).unwrap();
        let offset = label_pos - pos;
        fill_offset(i, offset)
      }
    }).collect();

    let stack_map_table_index = classfile.define_utf8("StackMapTable");
    let stack_map_table = Attribute::StackMapTable(stack_map_table_index, self.stack_frames);

    // TODO track max_locals counts instead of hard-coding to 1
    let code_index = classfile.define_utf8("Code");
    let code = Attribute::Code(code_index, self.max_stack_depth, 1, real_instructions, vec![], vec![stack_map_table]);

    let method = Method::new(self.access_flags, self.name_index, self.descriptor_index, vec![code]);
    classfile.methods.push(method);
  }
}