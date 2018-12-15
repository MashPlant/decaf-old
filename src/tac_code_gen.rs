use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::tac::*;

pub struct TacCodeGen {
  v_tbl: Vec<VTable>,
  cur_method: *const MethodDef,
}

impl TacCodeGen {
  pub fn gen(program: &mut Program) -> String {
    unimplemented!()
  }
}

fn resolve_field_order(class_def: &mut ClassDef) {
  if class_def.field_cnt >= 0 { return; }
  if !class_def.p_ptr.is_null() { resolve_field_order(class_def.p_ptr.get()); }
  let (mut field_cnt, mut v_fn_cnt) = if class_def.p_ptr.is_null() { (0, 0) } else {
    (class_def.p_ptr.get().field_cnt, class_def.p_ptr.get().v_tbl.methods.len() as i32)
  };
  'out: for field in &mut class_def.field {
    match field {
      FieldDef::MethodDef(method_def) => if !method_def.static_ {
        let p = class_def.p_ptr.get();
        for p_method in &p.v_tbl.methods {
          if p_method.get().name == method_def.name {
            method_def.offset = p_method.get().offset;
            continue 'out;
          }
        }
        method_def.offset = v_fn_cnt;
        v_fn_cnt += 1;
      }
      FieldDef::VarDef(var_def) => {
        var_def.offset = field_cnt;
        field_cnt += 1;
      }
    }
  }
}

impl TacCodeGen {
  fn program(&mut self, program: &mut Program) {
    for class_def in &mut program.class {
      resolve_field_order(class_def);
      for field_def in &mut class_def.field {
        if let FieldDef::MethodDef(method_def) = field_def {
          for param in &mut method_def.param {}
        }
      }
    }
  }
}
