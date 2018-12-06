use super::ast::*;

pub struct Temp {
  pub id: u32,

}

pub struct VTable {
  pub class: String,
}

pub struct Memo {
  pub method: *const MethodDef,
}

pub enum Tac {
  Add(u32, u32, u32),
  Sub(u32, u32, u32),
  Mul(u32, u32, u32),
  Div(u32, u32, u32),
  Mod(u32, u32, u32),
  Neg(u32, u32),
  And(u32, u32, u32),
  Or(u32, u32, u32),
  Not(u32, u32),
  Gt(u32, u32, u32),
  Ge(u32, u32, u32),
  Eq(u32, u32, u32),
  Ne(u32, u32, u32),
  Lq(u32, u32, u32),
  Le(u32, u32, u32),
  Assign(u32, u32),
  LoadVTbl(u32, *const VTable),
  IndirectCall,
  DirectCall,
  Ret,
  Jmp,
  Je,
  Jne,
  Load,
  Store,
  LoadIntConst,
  LoadStrConst,
  Label(u32),
  Param(u32),
}