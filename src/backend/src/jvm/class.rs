pub const MAGIC: u32 = 0xCAFEBABE;
pub const MAJOR_VERSION: u16 = 52;
pub const MINOR_VERSION: u16 = 0;
pub const ACC_PUBLIC: u16 = 0x1;
pub const ACC_PRIVATE: u16 = 0x2;
pub const ACC_STATIC: u16 = 0x8;
pub const ACC_FINAL: u16 = 0x10;

pub struct Class {
  // magic: u32 : doesn't need it here since it is const
  pub minor_version: u16,
  pub major_version: u16,
  pub constant_pool: Vec<Constant>,
  pub access_flags: u16,
  pub this_class: u16,
  pub super_class: u16,
  // interfaces: Vec<Interface> : not implemented
  pub fields: Vec<Field>,
  pub methods: Vec<Method>,
  // attributes: Vec<Attribute> : not implemented
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Constant {
  /* 1 */ Utf8(String),
  /* 3 */ Integer { bytes: u32 },
  /* 7 */ Class { name_index: u16 },
  /* 8 */ String { string_index: u16 },
  /* 9 */ FieldRef { class_index: u16, name_and_type_index: u16 },
  /* 10 */ MethodRef { class_index: u16, name_and_type_index: u16 },
  /* 12 */ NameAndType { name_index: u16, descriptor_index: u16 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Field {}

#[derive(Clone, Debug, PartialEq)]
pub struct Method {
  pub access_flags: u16,
  pub name_index: u16,
  pub descriptor_index: u16,
  // attributes: Vec<Attribute> : not implemented, instead we have...
  pub code: Code,
}

// actually Code is a kind of Attribute
// but the only Attribute I will implement is Code
// so it is extracted out
#[derive(Clone, Debug, PartialEq)]
pub struct Code {
  pub attribute_name_index: u16,
  // attribute_length: u32 : calculated instead of stored
  pub max_stack: u16,
  pub max_locals: u16,
  pub code: Vec<u8>,
  // exception_table: Vec<Exception>: not implemented
  // attributes: Vec<Attribute>: not implemented
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
  /* 0x02 */ IConstM1,
  /* 0x03 */ IConst0,
  /* 0x04 */ IConst1,
  /* 0x05 */ IConst2,
  /* 0x06 */ IConst3,
  /* 0x07 */ IConst4,
  /* 0x08 */ IConst5,
  /* 0x10 */ BIPush(u8),
  /* 0x12 */ LoadConstant(u8),
  /* 0x2A */ ALoad0,
  /* 0x2B */ ALoad1,
  /* 0x2C */ ALoad2,
  /* 0x2D */ ALoad3,
  /* 0x32 */ AALoad,
  /* 0x60 */ IAdd,
  /* 0x99 */ IfEq(u16),
  /* 0x9A */ IfNe(u16),
  /* 0x9B */ IfLt(u16),
  /* 0x9C */ IfGe(u16),
  /* 0x9D */ IfGt(u16),
  /* 0x9E */ IfLe(u16),
  /* 0x9F */ IfICmpEq(u16),
  /* 0xA0 */ IfICmpNe(u16),
  /* 0xA1 */ IfICmpLt(u16),
  /* 0xA2 */ IfICmpGe(u16),
  /* 0xA3 */ IfICmpGt(u16),
  /* 0xA4 */ IfICmpLe(u16),
  /* 0xA7 */ Goto(u16),
  /* 0xB1 */ Return,
  /* 0xB2 */ GetStatic(u16),
  /* 0xB6 */ InvokeVirtual(u16),
  /* 0xB7 */ InvokeSpecial(u16),
  /* 0xB8 */ InvokeStatic(u16),
  /* 0xBE */ ArrayLength,
}

impl Code {
//  pub fn attribute_length(&self) -> u32 {
//    2 /* max_stack */ + 2 /* max_locals */
//      + 4 /* code_length */ + self.code.len() as u32 /* code */
//      + 2 /* exception_table_length */ + 2 /* attributes_count */
//  }
}