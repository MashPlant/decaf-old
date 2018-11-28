//use std::ops::Add;
//
//pub trait Writer<Dst> {
//  fn write_to(&self, dst: &mut Dst);
//}
//
//pub trait FluentVec {
//  fn write() -> &FluentVec;
//}

use super::class::*;

pub trait Writer<DST> {
  fn write_to(self, dst: &mut DST);
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

impl Writer<Vec<u8>> for Class {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.write(MAGIC).write(self.minor_version).write(self.major_version)
      .write(self.constant_pool)
      .write(self.access_flags)
      .write(self.this_class).write(self.super_class)
      .write(1 as u8) // interfaces, count = 0, len = 1
      .write(self.fields)
      .write(self.methods)
      .write(1 as u8) // attributes, count = 0, len = 1
    ;
  }
}

impl Writer<Vec<u8>> for Vec<Constant> {
  fn write_to(self, dst: &mut Vec<u8>) {
    unimplemented!()
  }
}

impl Writer<Vec<u8>> for Vec<Field> {
  fn write_to(self, dst: &mut Vec<u8>) {
//    unimplemented!()
  }
}

impl Writer<Vec<u8>> for Vec<Method> {
  fn write_to(self, dst: &mut Vec<u8>) {
    dst.write(self.len() as u16);
    for method in self {
      dst.write(method.access_flags)
        .write(method.name_index)
        .write(method.descriptor_index)
        .write(2 as u8) // attributes, count = 1, len = 2
        .write(method.code) // the only attribute
      ;
    }
  }
}

impl Writer<Vec<u8>> for Code {
  fn write_to(mut self, dst: &mut Vec<u8>) {
    dst.append(&mut self.code);
  }
}

impl Writer<Vec<u8>> for Instruction {
  fn write_to(self, dst: &mut Vec<u8>) {
    use super::class::Instruction::*;
    match self {
      IConstM1 => dst.write(0x02 as u8),
      IConst0 => dst.write(0x03 as u8),
      IConst1 => dst.write(0x04 as u8),
      IConst2 => dst.write(0x05 as u8),
      IConst3 => dst.write(0x06 as u8),
      IConst4 => dst.write(0x07 as u8),
      IConst5 => dst.write(0x08 as u8),
      BIPush(byte) => dst.write(0x10 as u8).write(byte),
      LoadConstant(index) => dst.write(0x12 as u8).write(index),
      ALoad0 => dst.write(0x2A as u8),
      ALoad1 => dst.write(0x2B as u8),
      ALoad2 => dst.write(0x2C as u8),
      ALoad3 => dst.write(0x2D as u8),
      AALoad => dst.write(0x32 as u8),
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
      Goto(offset) => dst.write(0xA7 as u8).write(offset),
      IAdd => dst.write(0x60 as u8),
      Return => dst.write(0xB1 as u8),
      GetStatic(index) => dst.write(0xB2 as u8).write(index),
      InvokeVirtual(index) => dst.write(0xB6 as u8).write(index),
      InvokeSpecial(index) => dst.write(0xB7 as u8).write(index),
      InvokeStatic(index) => dst.write(0xB8 as u8).write(index),
      ArrayLength => dst.write(0xBE as u8),
    };
  }
}
