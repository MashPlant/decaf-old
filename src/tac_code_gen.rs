use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::tac::*;

pub struct TacCodeGen {
  cur_method: Vec<Tac>,
  reg_cnt: i32,
}

impl TacCodeGen {
  pub fn gen(&mut self, program: &mut Program) -> String {
    self.program(program);
    unimplemented!()
  }

  fn new_reg(&mut self) -> i32 {
    self.reg_cnt += 1;
    self.reg_cnt
  }

  fn add(&mut self, tac: Tac) -> &mut Self {
    self.cur_method.push(tac);
    self
  }
}

fn resolve_field_order(class_def: &mut ClassDef) {
  if class_def.field_cnt >= 0 { return; } // already handled
  let mut field_cnt = if class_def.p_ptr.is_null() { 0 } else {
    let p = class_def.p_ptr.get();
    resolve_field_order(p);
    class_def.v_tbl = p.v_tbl.clone();
    p.field_cnt
  };
  'out: for field in &mut class_def.field {
    match field {
      FieldDef::MethodDef(method_def) => if !method_def.static_ {
        let p = class_def.p_ptr.get();
        for p_method in &p.v_tbl.methods {
          if p_method.get().name == method_def.name {
            method_def.offset = p_method.get().offset;
            class_def.v_tbl.methods[method_def.offset as usize] = method_def;
            continue 'out;
          }
        }
        method_def.offset = class_def.v_tbl.methods.len() as i32;
        class_def.v_tbl.methods.push(method_def);
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
          // "this" is already inserted as 1st by symbol builder
          for param in &mut method_def.param {
            param.offset = self.new_reg();
          }
          self.block(&mut method_def.body);
        }
      }
    }
  }

  fn stmt(&mut self, stmt: &mut Stmt) {
    match stmt {}
  }

  fn simple(&mut self, simple: &mut Simple) {
    match simple {
      Simple::Assign(assign) => {
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        /*
        switch (assign.left.lvKind) {
            case ARRAY_ELEMENT:
                Tree.Indexed arrayRef = (Tree.Indexed) assign.left;
                Temp esz = tr.genLoadImm4(OffsetCounter.WORD_SIZE);
                Temp t = tr.genMul(arrayRef.index.val, esz);
                Temp base = tr.genAdd(arrayRef.array.val, t);
                tr.genStore(assign.expr.val, base, 0);
                break;
            case MEMBER_VAR:
                Tree.Ident varRef = (Tree.Ident) assign.left;
                tr.genStore(assign.expr.val, varRef.owner.val, varRef.symbol
                        .getOffset());
                break;
            case PARAM_VAR:
            case LOCAL_VAR:
                tr.genAssign(((Tree.Ident) assign.left).symbol.getTemp(),
                        assign.expr.val);
                break;
        }
        */
        match &assign.dst {
          Expr::Id(id) => {
            let var_def = id.symbol.get();
            match var_def.scope.get().kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => { self.add(Tac::Assign(var_def.offset, assign.src.reg)); }
              ScopeKind::Class(_) => {}
            }
          }
          Expr::Indexed(indexed) => {}
        }
      }
      Simple::VarDef(var_def) => {}
      Simple::Expr(expr) => self.expr(expr),
      Simple::Skip => {}
    }
  }

  fn block(&mut self, block: &mut Block) {}

  fn expr(&mut self, expr: &mut Expr) {}
}
