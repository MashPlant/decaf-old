use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum JavaType {
  /* Z */ Boolean,
  /* B */ Byte,
  /* C */ Char,
  /* S */ Short,
  /* I */ Int,
  /* J */ Long,
  /* F */ Float,
  /* D */ Double,
  /* V */ Void,
  /* LClassName; */ Class(&'static str),
  /* [type */ Array(Box<JavaType>),
}

impl fmt::Display for JavaType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::JavaType::*;
    match self {
      Boolean => write!(f, "Z"),
      Byte => write!(f, "B"),
      Char => write!(f, "C"),
      Short => write!(f, "S"),
      Int => write!(f, "I"),
      Long => write!(f, "J"),
      Float => write!(f, "F"),
      Double => write!(f, "D"),
      Void => write!(f, "V"),
      Class(s) => write!(f, "L{};", s),
      Array(t) => write!(f, "[{}", t),
    }
  }
}

pub fn make_method_type(argument_types: &[JavaType], return_type: &JavaType) -> String {
  let mut args = String::new();
  for t in argument_types {
    args += &t.to_string();
  }
  format!("({}){}", args, return_type)
}
