use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::tac::*;
use super::config::*;
use super::print::quote;

use std::ptr;

pub struct TacCodeGen {
  cur_method: *mut Vec<Tac>,
  break_stack: Vec<i32>,
  methods: Vec<TacMethod>,
  reg_cnt: i32,
  label_cnt: i32,
  cur_this: i32,
  // for one method, merge all 'array_index_out_of_bound' together for better performance
  out_of_bound_to_fill: Vec<i32>,
}

impl TacCodeGen {
  pub fn gen(program: &mut Program) -> TacProgram {
    let mut code_gen = TacCodeGen {
      cur_method: ptr::null_mut(),
      break_stack: Vec::new(),
      methods: Vec::new(),
      reg_cnt: -1,
      label_cnt: -1,
      cur_this: -1,
      out_of_bound_to_fill: Vec::new(),
    };
    code_gen.program(program);
    TacProgram {
      v_tables: program.class.iter().map(|class| class.v_tbl.clone()).collect(),
      methods: code_gen.methods,
    }
  }

  fn new_reg(&mut self) -> i32 {
    self.reg_cnt += 1;
    self.reg_cnt
  }

  fn int_const(&mut self, value: i32) -> i32 {
    let ret = self.new_reg();
    self.push(Tac::IntConst(ret, value));
    ret
  }

  fn new_label(&mut self) -> i32 {
    self.label_cnt += 1;
    self.label_cnt
  }

  fn array_length(&mut self, array: i32) -> i32 {
    let ret = self.new_reg();
    self.push(Tac::Load(ret, array, -INT_SIZE));
    ret
  }

  fn array_at(&mut self, array: i32, index: i32) -> i32 {
    let (ret, int_size, offset) = (self.new_reg(), self.int_const(INT_SIZE), self.new_reg());
    self.push(Tac::Mul(offset, index, int_size));
    self.push(Tac::Add(offset, array, offset));
    self.push(Tac::Load(ret, offset, 0));
    ret
  }

  fn check_array_index(&mut self, array: i32, index: i32) -> i32 {
    let (ret, zero, arr_len, cmp) = (self.new_reg(), self.new_reg(), self.array_length(array), self.new_reg());
    self.push(Tac::IntConst(zero, 0));
    self.push(Tac::Ge(ret, index, zero));
    self.push(Tac::Lt(cmp, index, arr_len));
    self.push(Tac::And(ret, ret, cmp));
    ret
  }

  fn check_div_0(&mut self, r: i32) {
    let ok = self.new_label();
    let msg = self.new_reg();
    self.push(Tac::Jne(r, ok));
    self.push(Tac::StrConst(msg, quote(DIV_0)));
    self.push(Tac::Param(msg));
    self.intrinsic_call(PRINT_STRING);
    self.intrinsic_call(HALT);
    self.push(Tac::Label(ok));
  }

  fn intrinsic_call(&mut self, call: IntrinsicCall) -> i32 {
    let ret = if call.ret { self.new_reg() } else { -1 };
    self.push(Tac::DirectCall(ret, call.name.to_owned()));
    ret
  }

  fn instance_of(&mut self, object: i32, class: &'static str) -> i32 {
    // ret = 0
    // while (cur)
    //   if cur == target
    //     ret = 1
    //     break
    //   cur = cur->parent
    let (ret, cur, target) = (self.new_reg(), self.new_reg(), self.new_reg());
    let (before_cond, after_body) = (self.new_label(), self.new_label());
    self.push(Tac::IntConst(ret, 0));
    self.push(Tac::LoadVTbl(target, class));
    self.push(Tac::Load(cur, object, 0));
    self.push(Tac::Label(before_cond));
    self.push(Tac::Je(cur, after_body));
    self.push(Tac::Eq(ret, cur, target));
    self.push(Tac::Jne(ret, after_body));
    self.push(Tac::Load(cur, cur, 0));
    self.push(Tac::Jmp(before_cond));
    self.push(Tac::Label(after_body));
    ret
  }

  fn push(&mut self, tac: Tac) {
    self.cur_method.get().push(tac);
  }
}

// also used in llvm code gen
pub fn resolve_field_order(class_def: &mut ClassDef) {
  if class_def.field_cnt >= 0 { return; } // already handled
  class_def.field_cnt = if class_def.p_ptr.is_null() { 0 } else {
    let p = class_def.p_ptr.get();
    resolve_field_order(p);
    class_def.v_tbl = p.v_tbl.clone();
    p.field_cnt
  };
  class_def.v_tbl.class = class_def;
  'out: for field in &mut class_def.field {
    match field {
      FieldDef::MethodDef(method_def) => if !method_def.static_ {
        if !class_def.p_ptr.is_null() {
          let p = class_def.p_ptr.get();
          for p_method in &p.v_tbl.methods {
            if p_method.get().name == method_def.name {
              method_def.offset = p_method.get().offset;
              class_def.v_tbl.methods[method_def.offset as usize] = method_def;
              continue 'out;
            }
          }
        }
        method_def.offset = class_def.v_tbl.methods.len() as i32;
        class_def.v_tbl.methods.push(method_def);
      }
      FieldDef::VarDef(var_def) => {
        var_def.offset = class_def.field_cnt;
        class_def.field_cnt += 1;
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
        }
      }
      self.methods.push(TacMethod { name: format!("_{}_New", class_def.name), code: Vec::new(), method: ptr::null() });
      self.cur_method = &mut self.methods.last_mut().unwrap().code;
      let size = self.new_reg();
      self.push(Tac::IntConst(size, (class_def.field_cnt + 1) * INT_SIZE));
      self.push(Tac::Param(size));
      let ret = self.intrinsic_call(ALLOCATE);
      let v_tbl = self.new_reg();
      self.push(Tac::LoadVTbl(v_tbl, class_def.name));
      self.push(Tac::Store(ret, 0, v_tbl));
      let zero = self.new_reg();
      self.push(Tac::IntConst(zero, 0));
      for i in 0..class_def.field_cnt {
        self.push(Tac::Store(ret, (i + 1) * INT_SIZE, zero));
      }
      self.push(Tac::Ret(ret));
      let class_def_ptr = class_def as *const _;
      for field_def in &mut class_def.field {
        if let FieldDef::MethodDef(method_def) = field_def {
          self.methods.push(TacMethod {
            name: if class_def_ptr == program.main && method_def.name == MAIN_METHOD { "main".to_owned() } else { format!("_{}.{}", class_def.name, method_def.name) },
            code: Vec::new(),
            method: method_def,
          });
          self.cur_method = &mut self.methods.last_mut().unwrap().code;
          if !method_def.static_ {
            self.cur_this = method_def.param[0].offset;
          }
          self.out_of_bound_to_fill.clear();
          self.block(&mut method_def.body);
          if !self.out_of_bound_to_fill.is_empty() {
            let (halt, after) = (self.new_label(), self.new_label());
            let msg = self.new_reg();
            self.push(Tac::Jmp(after));
            self.push(Tac::Label(halt));
            self.push(Tac::StrConst(msg, quote(ARRAY_INDEX_OUT_OF_BOUND)));
            self.push(Tac::Param(msg));
            self.intrinsic_call(PRINT_STRING);
            self.intrinsic_call(HALT);
            self.push(Tac::Label(after));
            for to_fill in &self.out_of_bound_to_fill {
              if let Tac::Je(_, label) = &mut self.cur_method.get()[*to_fill as usize] {
                *label = halt;
              } else { unreachable!(); }
            }
          }
        }
      }
    }
  }

  fn stmt(&mut self, stmt: &mut Stmt) {
    use ast::Stmt::*;
    match stmt {
      Simple(simple) => self.simple(simple),
      If(if_) => {
        let before_else = self.new_label();
        self.expr(&mut if_.cond);
        self.push(Tac::Je(if_.cond.tac_reg, before_else));
        self.block(&mut if_.on_true);
        if let Some(on_false) = &mut if_.on_false {
          let after_else = self.new_label();
          self.push(Tac::Jmp(after_else));
          self.push(Tac::Label(before_else));
          self.block(on_false);
          self.push(Tac::Label(after_else));
        } else {
          self.push(Tac::Label(before_else));
        }
      }
      While(while_) => {
        let (before_cond, after_body) = (self.new_label(), self.new_label());
        self.push(Tac::Label(before_cond));
        self.expr(&mut while_.cond);
        self.push(Tac::Je(while_.cond.tac_reg, after_body));
        self.break_stack.push(after_body);
        self.block(&mut while_.body);
        self.break_stack.pop();
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(after_body));
      }
      For(for_) => {
        let (before_cond, after_body) = (self.new_label(), self.new_label());
        self.simple(&mut for_.init);
        self.push(Tac::Label(before_cond));
        self.expr(&mut for_.cond);
        self.push(Tac::Je(for_.cond.tac_reg, after_body));
        self.break_stack.push(after_body);
        self.block(&mut for_.body);
        self.break_stack.pop();
        self.simple(&mut for_.update);
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(after_body));
      }
      Return(return_) => if let Some(expr) = &mut return_.expr {
        self.expr(expr);
        self.push(Tac::Ret(expr.tac_reg));
      } else {
        self.push(Tac::Ret(-1));
      }
      Print(print) => for expr in &mut print.print {
        self.expr(expr);
        self.push(Tac::Param(expr.tac_reg));
        match &expr.type_ {
          SemanticType::Int => { self.intrinsic_call(PRINT_INT); }
          SemanticType::Bool => { self.intrinsic_call(PRINT_BOOL); }
          SemanticType::String => { self.intrinsic_call(PRINT_STRING); }
          _ => unreachable!(),
        }
      }
      Break(_) => {
        let after_loop = *self.break_stack.last().unwrap();
        self.push(Tac::Jmp(after_loop));
      }
      SCopy(s_copy) => {
        self.expr(&mut s_copy.src);
        let new_obj = self.new_reg();
        let tmp = self.new_reg();
        let class = s_copy.src.type_.get_class();
        self.push(Tac::DirectCall(new_obj, format!("_{}_New", class.name)));
        for i in 0..class.field_cnt {
          self.push(Tac::Load(tmp, s_copy.src.tac_reg, (i + 1) * INT_SIZE));
          self.push(Tac::Store(new_obj, (i + 1) * INT_SIZE, tmp));
        }
        self.push(Tac::Assign(s_copy.dst_sym.get().offset, new_obj));
      }
      Foreach(foreach) => {
        self.expr(&mut foreach.arr);
        foreach.def.offset = self.new_reg();
        let (i, int_size, cmp) = (self.new_reg(), self.int_const(INT_SIZE), self.new_reg());
        let (before_cond, after_body) = (self.new_label(), self.new_label());
        self.push(Tac::IntConst(i, 0));
        let end = self.array_length(foreach.arr.tac_reg);
        self.push(Tac::Mul(end, end, int_size));
        self.push(Tac::Add(end, end, foreach.arr.tac_reg));
        self.push(Tac::Assign(i, foreach.arr.tac_reg));
        self.push(Tac::Label(before_cond));
        self.push(Tac::Lt(cmp, i, end));
        self.push(Tac::Je(cmp, after_body));
        self.push(Tac::Load(foreach.def.offset, i, 0));
        if let Some(cond) = &mut foreach.cond {
          self.expr(cond);
          self.push(Tac::Je(cond.tac_reg, after_body));
        }
        self.break_stack.push(after_body);
        self.block(&mut foreach.body);
        self.break_stack.pop();
        self.push(Tac::Add(i, i, int_size));
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(after_body));
      }
      Guarded(guarded) => for (e, b) in &mut guarded.guarded {
        self.expr(e);
        let after_body = self.new_label();
        self.push(Tac::Je(e.tac_reg, after_body));
        self.block(b);
        self.push(Tac::Label(after_body));
      }
      Block(block) => self.block(block),
    }
  }

  fn simple(&mut self, simple: &mut Simple) {
    match simple {
      Simple::Assign(assign) => {
        if let ExprData::Id(id) = &mut assign.dst.data { id.for_assign = true; }
        if let ExprData::Indexed(indexed) = &mut assign.dst.data { indexed.for_assign = true; }
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        match &assign.dst.data {
          ExprData::Id(id) => {
            let var_def = id.symbol.get();
            match var_def.scope.get().kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => { self.push(Tac::Assign(var_def.offset, assign.src.tac_reg)); }
              ScopeKind::Class(_) => {
                self.push(Tac::Store(id.owner.as_ref().unwrap().tac_reg, (var_def.offset + 1) * INT_SIZE, assign.src.tac_reg));
              }
              _ => unreachable!(),
            }
          }
          ExprData::Indexed(indexed) => {
            let (int_size, offset) = (self.int_const(INT_SIZE), self.new_reg());
            self.push(Tac::Mul(offset, indexed.idx.tac_reg, int_size));
            self.push(Tac::Add(offset, indexed.arr.tac_reg, offset));
            self.push(Tac::Store(offset, 0, assign.src.tac_reg));
          }
          _ => unreachable!(),
        }
      }
      Simple::VarDef(var_def) => {
        var_def.offset = self.new_reg();
        if let Some(src) = &mut var_def.src {
          self.expr(src);
          self.push(Tac::Assign(var_def.offset, src.tac_reg));
        }
      }
      Simple::Expr(expr) => self.expr(expr),
      Simple::Skip => {}
    }
  }

  fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt { self.stmt(stmt); }
  }

  fn expr(&mut self, expr: &mut Expr) {
    use ast::ExprData::*;
    match &mut expr.data {
      Id(id) => {
        let var_def = id.symbol.get();
        match var_def.scope.get().kind {
          ScopeKind::Local(_) | ScopeKind::Parameter(_) => expr.tac_reg = var_def.offset,
          ScopeKind::Class(_) => {
            let owner = id.owner.as_mut().unwrap();
            self.expr(owner);
            if !id.for_assign {
              expr.tac_reg = self.new_reg();
              self.push(Tac::Load(expr.tac_reg, owner.tac_reg, (var_def.offset + 1) * INT_SIZE));
            }
          }
          _ => unreachable!(),
        };
      }
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
        let check = self.check_array_index(indexed.arr.tac_reg, indexed.idx.tac_reg);
        self.out_of_bound_to_fill.push(self.cur_method.get().len() as i32);
        self.push(Tac::Je(check, -1)); // jump where not determined yet
        if !indexed.for_assign { // not used, but still check index here
          expr.tac_reg = self.array_at(indexed.arr.tac_reg, indexed.idx.tac_reg);
        }
      }
      IntConst(v) => {
        expr.tac_reg = self.new_reg();
        self.push(Tac::IntConst(expr.tac_reg, *v));
      }
      BoolConst(v) => expr.tac_reg = if *v { self.int_const(1) } else { self.int_const(0) },
      StringConst(v) => {
        expr.tac_reg = self.new_reg();
        self.push(Tac::StrConst(expr.tac_reg, quote(v)));
      }
      ArrayConst(_) => unimplemented!(),
      Null => expr.tac_reg = self.int_const(0),
      Call(call) => if call.is_arr_len {
        let owner = call.owner.as_mut().unwrap();
        self.expr(owner);
        expr.tac_reg = self.array_length(owner.tac_reg);
      } else {
        let method = call.method.get();
        let class = method.class.get();
        expr.tac_reg = if method.ret_t.sem != VOID { self.new_reg() } else { -1 };
        if method.static_ {
          // fuck vm spec
          // 'parm' can only be consecutive for one call, there cannot be other call in this process
          // e.g. my old version: f(1, x[0]) : parm 1 => (if <out of bound> call _Halt) => parm x[0]
          // which caused error
          for arg in &mut call.arg { self.expr(arg); }
          for arg in &mut call.arg { self.push(Tac::Param(arg.tac_reg)); }
          self.push(Tac::DirectCall(expr.tac_reg, format!("_{}.{}", class.name, method.name)));
        } else {
          let owner = call.owner.as_mut().unwrap();
          self.expr(owner);
          for arg in &mut call.arg { self.expr(arg); }
          self.push(Tac::Param(owner.tac_reg));
          for arg in &mut call.arg { self.push(Tac::Param(arg.tac_reg)); }
          let slot = self.new_reg();
          self.push(Tac::Load(slot, owner.tac_reg, 0));
          self.push(Tac::Load(slot, slot, (method.offset + 2) * INT_SIZE));
          self.push(Tac::IndirectCall(expr.tac_reg, slot));
        }
      }
      Unary(unary) => {
        self.expr(&mut unary.r);
        expr.tac_reg = self.new_reg();
        match unary.op {
          Operator::Neg => self.push(Tac::Neg(expr.tac_reg, unary.r.tac_reg)),
          Operator::Not => self.push(Tac::Not(expr.tac_reg, unary.r.tac_reg)),
          _ => unimplemented!(),
        }
      }
      Binary(binary) => {
        use ast::Operator::*;
        self.expr(&mut binary.l);
        self.expr(&mut binary.r);
        expr.tac_reg = self.new_reg();
        let (l, r, d) = (binary.l.tac_reg, binary.r.tac_reg, expr.tac_reg);
        match binary.op {
          Add => self.push(Tac::Add(d, l, r)),
          Sub => self.push(Tac::Sub(d, l, r)),
          Mul => self.push(Tac::Mul(d, l, r)),
          Div => {
            self.check_div_0(r);
            self.push(Tac::Div(d, l, r));
          }
          Mod => {
            self.check_div_0(r);
            self.push(Tac::Mod(d, l, r));
          }
          Lt => self.push(Tac::Lt(d, l, r)),
          Le => self.push(Tac::Le(d, l, r)),
          Gt => self.push(Tac::Gt(d, l, r)),
          Ge => self.push(Tac::Ge(d, l, r)),
          And => self.push(Tac::And(d, l, r)),
          Or => self.push(Tac::Or(d, l, r)),
          Eq | Ne => if binary.l.type_ == STRING {
            self.push(Tac::Param(l));
            self.push(Tac::Param(r));
            expr.tac_reg = self.intrinsic_call(STRING_EQUAL);
            if binary.op == Ne { self.push(Tac::Not(expr.tac_reg, expr.tac_reg)); }
          } else {
            self.push(if binary.op == Eq { Tac::Eq(d, l, r) } else { Tac::Ne(d, l, r) });
          }
          Repeat => {
            let (ok, before_cond, finish) = (self.new_label(), self.new_label(), self.new_label());
            let (zero, int_size, cmp, msg, i) = (self.int_const(0), self.int_const(INT_SIZE), self.new_reg(), self.new_reg(), self.new_reg());
            self.push(Tac::Ge(cmp, r, zero));
            self.push(Tac::Jne(cmp, ok));
            self.push(Tac::StrConst(msg, quote(REPEAT_NEG)));
            self.push(Tac::Param(msg));
            self.intrinsic_call(PRINT_STRING);
            self.intrinsic_call(HALT);
            self.push(Tac::Label(ok));
            self.push(Tac::Mul(i, r, int_size));
            self.push(Tac::Add(i, i, int_size));
            self.push(Tac::Param(i));
            expr.tac_reg = self.intrinsic_call(ALLOCATE);
            self.push(Tac::Add(i, expr.tac_reg, i));
            self.push(Tac::Label(before_cond));
            self.push(Tac::Sub(i, i, int_size));
            self.push(Tac::Eq(cmp, i, expr.tac_reg));
            self.push(Tac::Jne(cmp, finish));
            match &binary.l.type_ {
              SemanticType::Object(class) => {
                // perform s_copy
                let (new_obj, tmp) = (self.new_reg(), self.new_reg());
                let class = class.get();
                self.push(Tac::DirectCall(new_obj, format!("_{}_New", class.name)));
                for i in 0..class.field_cnt {
                  self.push(Tac::Load(tmp, l, (i + 1) * INT_SIZE));
                  self.push(Tac::Store(new_obj, (i + 1) * INT_SIZE, tmp));
                }
                self.push(Tac::Store(i, 0, new_obj));
              }
              _ => self.push(Tac::Store(i, 0, l)),
            }
            self.push(Tac::Jmp(before_cond));
            self.push(Tac::Label(finish));
            self.push(Tac::Store(i, 0, r));
            self.push(Tac::Add(expr.tac_reg, expr.tac_reg, int_size));
          }
          _ => unimplemented!(),
        }
      }
      This => expr.tac_reg = self.cur_this,
      ReadInt => expr.tac_reg = self.intrinsic_call(READ_INT),
      ReadLine => expr.tac_reg = self.intrinsic_call(READ_LINE),
      NewClass { name } => {
        expr.tac_reg = self.new_reg();
        self.push(Tac::DirectCall(expr.tac_reg, format!("_{}_New", name)));
      }
      NewArray { elem_t: _, len } => {
        self.expr(len);
        let (halt, before_cond, finish) = (self.new_label(), self.new_label(), self.new_label());
        let (zero, int_size, cmp, i, msg) = (self.int_const(0), self.int_const(INT_SIZE), self.new_reg(), self.new_reg(), self.new_reg());
        self.push(Tac::Lt(cmp, len.tac_reg, zero));
        self.push(Tac::Jne(cmp, halt));
        self.push(Tac::Mul(i, len.tac_reg, int_size));
        self.push(Tac::Add(i, i, int_size)); // allocate (len + 1) * INT_SIZE
        self.push(Tac::Param(i));
        expr.tac_reg = self.intrinsic_call(ALLOCATE);
        self.push(Tac::Add(i, expr.tac_reg, i));
        self.push(Tac::Label(before_cond));
        self.push(Tac::Sub(i, i, int_size));
        self.push(Tac::Eq(cmp, i, expr.tac_reg));
        self.push(Tac::Jne(cmp, finish));
        self.push(Tac::Store(i, 0, zero));
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(halt));
        self.push(Tac::StrConst(msg, quote(NEGATIVE_ARRAY_SIZE)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.intrinsic_call(HALT);
        self.push(Tac::Label(finish));
        self.push(Tac::Store(i, 0, len.tac_reg)); // array[-1] = len
        self.push(Tac::Add(expr.tac_reg, expr.tac_reg, int_size));
      }
      TypeTest { expr: src, name } => {
        self.expr(src);
        expr.tac_reg = self.instance_of(src.tac_reg, name);
      }
      TypeCast { name, expr: src } => {
        self.expr(src);
        expr.tac_reg = src.tac_reg;
        let check = self.instance_of(src.tac_reg, name);
        let ok = self.new_label();
        let (msg, v_tbl) = (self.new_reg(), self.new_reg());
        self.push(Tac::Jne(check, ok));
        self.push(Tac::StrConst(msg, quote(CLASS_CAST1)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.push(Tac::Load(v_tbl, src.tac_reg, 0));
        self.push(Tac::Load(msg, v_tbl, INT_SIZE)); // name info is in v-table[1]
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.push(Tac::StrConst(msg, quote(CLASS_CAST2)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.push(Tac::StrConst(msg, quote(name)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.push(Tac::StrConst(msg, quote(CLASS_CAST3)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.intrinsic_call(HALT);
        self.push(Tac::Label(ok));
      }
      Range(_) => unimplemented!(),
      Default(default) => {
        self.expr(&mut default.arr);
        self.expr(&mut default.idx);
        expr.tac_reg = self.new_reg();
        let (use_dft, after) = (self.new_label(), self.new_label());
        let check = self.check_array_index(default.arr.tac_reg, default.idx.tac_reg);
        self.push(Tac::Je(check, use_dft));
        let idx_res = self.array_at(default.arr.tac_reg, default.idx.tac_reg);
        self.push(Tac::Assign(expr.tac_reg, idx_res));
        self.push(Tac::Jmp(after));
        self.push(Tac::Label(use_dft));
        self.expr(&mut default.dft);
        self.push(Tac::Assign(expr.tac_reg, default.dft.tac_reg));
        self.push(Tac::Label(after));
      }
      Comprehension(_) => unimplemented!(),
    }
  }
}