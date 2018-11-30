use super::class::*;
use std::fs;

pub trait Writer<DST> {
  fn write_to(self, dst: &mut DST);
}

pub trait FileWriter {
  fn write_to_file(self, file: &str);
}

impl<T: Writer<Vec<u8>>> FileWriter for T {
  fn write_to_file(self, file: &str) {
    use std::io::Write;
    let mut file = fs::File::create(file).unwrap();
    let mut dst = Vec::new();
    self.write_to(&mut dst);
    file.write_all(&dst).unwrap();
  }
}

trait FluentVec {
  fn write<T>(&mut self, t: T) -> &mut Self
    where T: Writer<Self>, Self: Sized {
    t.write_to(self);
    self
  }
}

impl FluentVec for Vec<u8> {}

impl Writer<Vec<u8>> for u8 {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.push(self);
  }
}

impl Writer<Vec<u8>> for u16 {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.push((self >> 8) as u8);
    dst.push(self as u8);
  }
}

impl Writer<Vec<u8>> for u32 {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.push((self >> 24) as u8);
    dst.push((self >> 16) as u8);
    dst.push((self >> 8) as u8);
    dst.push(self as u8);
  }
}

impl Writer<Vec<u8>> for Vec<u8> {
  fn write_to(mut self, dst: &mut Vec<u8>) {
    dst.write(self.len() as u32)
      .append(&mut self);
  }
}

impl Writer<Vec<u8>> for Class {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.write(MAGIC).write(MINOR_VERSION).write(MAJOR_VERSION)
      .write(self.constant_pool)
      .write(self.access_flags)
      .write(self.this_class).write(self.super_class)
      .write(0 as u16) // interfaces_count
      .write(self.fields)
      .write(self.methods)
      .write(0 as u16) // attributes_count
    ;
  }
}

impl Writer<Vec<u8>> for Vec<Constant> {
  fn write_to(self, dst: &mut Vec<u8>) {
    use super::class::Constant::*;
    dst.write(self.len() as u16 + 1);
    for constant in self {
      match constant {
        Utf8(s) => { dst.write(1 as u8).write(s.len() as u16).append(&mut s.into_bytes()); }
        Integer { bytes } => { dst.write(3 as u8).write(bytes); }
        Class { name_index } => { dst.write(7 as u8).write(name_index); }
        String { string_index } => { dst.write(8 as u8).write(string_index); }
        FieldRef { class_index, name_and_type_index } => { dst.write(9 as u8).write(class_index).write(name_and_type_index); }
        MethodRef { class_index, name_and_type_index } => { dst.write(10 as u8).write(class_index).write(name_and_type_index); }
        NameAndType { name_index, descriptor_index } => { dst.write(12 as u8).write(name_index).write(descriptor_index); }
      };
    }
  }
}

impl Writer<Vec<u8>> for Vec<Field> {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.write(self.len() as u16);
    for field in self {
      dst.write(field.access_flags)
        .write(field.name_index)
        .write(field.descriptor_index)
        .write(0 as u16) // attributes_count
      ;
    }
  }
}

impl Writer<Vec<u8>> for Vec<Method> {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.write(self.len() as u16);
    for method in self {
      dst.write(method.access_flags)
        .write(method.name_index)
        .write(method.descriptor_index)
        .write(1 as u16) // attributes_count
        .write(method.code) // the only attribute
      ;
    }
  }
}

impl Writer<Vec<u8>> for Code {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.write(self.attribute_name_index)
      .write(2 /* max_stack */ + 2 /* max_locals */
               + 4 /* code_length */ + self.code.len() as u32 /* code */
               + 2 /* exception_table_length */ + 2 /* attributes_count */)
      .write(self.max_stack)
      .write(self.max_locals)
      .write(self.code)
      .write(0 as u16) // exception_table_length
      .write(0 as u16) // attributes_count
    ;
  }
}

impl Writer<Vec<u8>> for Instruction {
  fn write_to(self, dst: &mut Vec<u8>) {
    use super::class::Instruction::*;
    match self {
      AConstNull => dst.write(0x01 as u8),
      IConstM1 => dst.write(0x02 as u8),
      IConst0 => dst.write(0x03 as u8),
      IConst1 => dst.write(0x04 as u8),
      IConst2 => dst.write(0x05 as u8),
      IConst3 => dst.write(0x06 as u8),
      IConst4 => dst.write(0x07 as u8),
      IConst5 => dst.write(0x08 as u8),
      BIPush(byte) => dst.write(0x10 as u8).write(byte),
      SIPush(bytes) => dst.write(0x11 as u8).write(bytes),
      Ldc(index) => dst.write(0x12 as u8).write(index),
      LdcW(index) => dst.write(0x13 as u8).write(index),
      ILoad(stack_index) => dst.write(0x15 as u8).write(stack_index),
      ALoad(stack_index) => dst.write(0x19 as u8).write(stack_index),
      ILoad0 => dst.write(0x1A as u8),
      ILoad1 => dst.write(0x1B as u8),
      ILoad2 => dst.write(0x1C as u8),
      ILoad3 => dst.write(0x1D as u8),
      ALoad0 => dst.write(0x2A as u8),
      ALoad1 => dst.write(0x2B as u8),
      ALoad2 => dst.write(0x2C as u8),
      ALoad3 => dst.write(0x2D as u8),
      IALoad => dst.write(0x2E as u8),
      AALoad => dst.write(0x32 as u8),
      BALoad => dst.write(0x33 as u8),
      IStore(index) => dst.write(0x36 as u8).write(index),
      AStore(index) => dst.write(0x3A as u8).write(index),
      IStore0 => dst.write(0x3B as u8),
      IStore1 => dst.write(0x3C as u8),
      IStore2 => dst.write(0x3D as u8),
      IStore3 => dst.write(0x3E as u8),
      AStore0 => dst.write(0x4B as u8),
      AStore1 => dst.write(0x4C as u8),
      AStore2 => dst.write(0x4D as u8),
      AStore3 => dst.write(0x4E as u8),
      IAStore => dst.write(0x4F as u8),
      AAStore => dst.write(0x53 as u8),
      BAStore => dst.write(0x54 as u8),
      Dup => dst.write(0x59 as u8),
      Swap => dst.write(0x5F as u8),
      IAdd => dst.write(0x60 as u8),
      ISub => dst.write(0x64 as u8),
      IMul => dst.write(0x68 as u8),
      IDiv => dst.write(0x6C as u8),
      IRem => dst.write(0x70 as u8),
      INeg => dst.write(0x74 as u8),
      IInc(index, value) => dst.write(0x84 as u8).write(index).write(value),
      IfEq(offset) => dst.write(0x99 as u8).write(offset),
      IfNe(offset) => dst.write(0x9A as u8).write(offset),
      IfLt(offset) => dst.write(0x9B as u8).write(offset),
      IfGe(offset) => dst.write(0x9C as u8).write(offset),
      IfGt(offset) => dst.write(0x9D as u8).write(offset),
      IfLe(offset) => dst.write(0x9E as u8).write(offset),
      IfICmpEq(offset) => dst.write(0x9F as u8).write(offset),
      IfICmpNe(offset) => dst.write(0xA0 as u8).write(offset),
      IfICmpLt(offset) => dst.write(0xA1 as u8).write(offset),
      IfICmpGe(offset) => dst.write(0xA2 as u8).write(offset),
      IfICmpGt(offset) => dst.write(0xA3 as u8).write(offset),
      IfICmpLe(offset) => dst.write(0xA4 as u8).write(offset),
      IfACmpEq(offset) => dst.write(0xA5 as u8).write(offset),
      IfACmpNe(offset) => dst.write(0xA6 as u8).write(offset),
      Goto(offset) => dst.write(0xA7 as u8).write(offset),
      IReturn => dst.write(0xAC as u8),
      AReturn => dst.write(0xB0 as u8),
      Return => dst.write(0xB1 as u8),
      GetStatic(index) => dst.write(0xB2 as u8).write(index),
      GetField(index) => dst.write(0xB4 as u8).write(index),
      PutField(index) => dst.write(0xB5 as u8).write(index),
      InvokeVirtual(index) => dst.write(0xB6 as u8).write(index),
      InvokeSpecial(index) => dst.write(0xB7 as u8).write(index),
      InvokeStatic(index) => dst.write(0xB8 as u8).write(index),
      New(index) => dst.write(0xBB as u8).write(index),
      NewArray(a_type) => dst.write(0xBC as u8).write(a_type),
      ANewArray(index) => dst.write(0xBD as u8).write(index),
      ArrayLength => dst.write(0xBE as u8),
      CheckCast(index) => dst.write(0xC0 as u8).write(index),
      InstanceOf(index) => dst.write(0xC1 as u8).write(index),
    };
  }
}
