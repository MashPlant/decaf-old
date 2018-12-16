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
}

impl TacCodeGen {
  pub fn new() -> TacCodeGen {
    TacCodeGen {
      cur_method: ptr::null_mut(),
      break_stack: Vec::new(),
      methods: Vec::new(),
      reg_cnt: -1,
      label_cnt: -1,
      cur_this: -1,
    }
  }

  pub fn gen(mut self, program: &mut Program) -> TacProgram {
    self.program(program);
    TacProgram {
      v_tables: program.class.iter().map(|class| class.v_tbl.clone()).collect(),
      methods: self.methods,
    }
  }

  fn new_reg(&mut self) -> i32 {
    self.reg_cnt += 1;
    self.reg_cnt
  }

  fn new_label(&mut self) -> i32 {
    self.label_cnt += 1;
    self.label_cnt
  }

  fn array_length(&mut self, array: i32) -> i32 {
    let ret = self.new_reg();
    self.push(Tac::Load { dst: ret, base: array, offset: -INT_SIZE });
    ret
  }

  fn array_at(&mut self, array: i32, index: i32) -> i32 {
    let (ret, int_size, offset) = (self.new_reg(), self.new_reg(), self.new_reg());
    self.push(Tac::IntConst(int_size, INT_SIZE));
    self.push(Tac::Mul(offset, index, int_size));
    self.push(Tac::Add(offset, array, offset));
    self.push(Tac::Load { dst: ret, base: offset, offset: 0 });
    ret
  }

  fn check_array_index(&mut self, array: i32, index: i32) -> i32 {
    let (ret, zero, arr_len, cmp) = (self.new_reg(), self.new_reg(), self.array_length(array), self.new_reg());
    let (err, after) = (self.new_label(), self.new_label());
    self.push(Tac::IntConst(zero, 0));
    self.push(Tac::Lt(cmp, index, zero));
    self.push(Tac::Jne(cmp, err));
    self.push(Tac::Lt(cmp, index, arr_len));
    self.push(Tac::Je(cmp, err));
    self.push(Tac::IntConst(ret, 1));
    self.push(Tac::Jmp(after));
    self.push(Tac::Label(err));
    self.push(Tac::IntConst(ret, 0));
    self.push(Tac::Label(after));
    ret
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
    let ret = self.new_reg();
    let (before_cond, after_body) = (self.new_label(), self.new_label());
    let (cur, target) = (self.new_reg(), self.new_reg());
    self.push(Tac::IntConst(ret, 0));
    self.push(Tac::LoadVTbl(target, class));
    self.push(Tac::Load { dst: cur, base: object, offset: 0 });
    self.push(Tac::Label(before_cond));
    self.push(Tac::Je(cur, after_body));
    self.push(Tac::Eq(ret, cur, target));
    self.push(Tac::Jne(ret, after_body));
    self.push(Tac::Load { dst: cur, base: cur, offset: 0 });
    self.push(Tac::Jmp(before_cond));
    self.push(Tac::Label(after_body));
    ret
  }

  fn push(&mut self, tac: Tac) {
    self.cur_method.get().push(tac);
  }
}

fn resolve_field_order(class_def: &mut ClassDef) {
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
      self.push(Tac::Store { base: ret, offset: 0, src: v_tbl });
      let zero = self.new_reg();
      self.push(Tac::IntConst(zero, 0));
      for i in 0..class_def.field_cnt {
        self.push(Tac::Store { base: ret, offset: (i + 1) * INT_SIZE, src: zero });
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
          self.block(&mut method_def.body);
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
        self.push(Tac::Je(if_.cond.reg, before_else));
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
        self.push(Tac::Je(while_.cond.reg, after_body));
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
        self.push(Tac::Je(for_.cond.reg, after_body));
        self.break_stack.push(after_body);
        self.block(&mut for_.body);
        self.break_stack.pop();
        self.simple(&mut for_.update);
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(after_body));
      }
      Return(return_) => if let Some(expr) = &mut return_.expr {
        self.expr(expr);
        self.push(Tac::Ret(expr.reg));
      } else {
        self.push(Tac::Ret(-1));
      }
      Print(print) => for expr in &mut print.print {
        self.expr(expr);
        self.push(Tac::Param(expr.reg));
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
          self.push(Tac::Load { dst: tmp, base: s_copy.src.reg, offset: (i + 1) * INT_SIZE });
          self.push(Tac::Store { base: new_obj, offset: (i + 1) * INT_SIZE, src: tmp });
        }
        self.push(Tac::Assign(s_copy.dst_sym.get().offset, new_obj));
      }
      Foreach(foreach) => {
        self.expr(&mut foreach.arr);
        foreach.def.offset = self.new_reg();
        let (x, i, one, cmp) = (self.new_reg(), self.new_reg(), self.new_reg(), self.new_reg());
        let (before_cond, after_body) = (self.new_label(), self.new_label());
        self.push(Tac::IntConst(i, 0));
        self.push(Tac::IntConst(one, 1));
        self.push(Tac::Label(before_cond));
        let array_length = self.array_length(foreach.arr.reg);
        self.push(Tac::Le(cmp, i, array_length));
        self.push(Tac::Je(cmp, after_body));
        let array_elem = self.array_at(foreach.arr.reg, i);
        self.push(Tac::Assign(x, array_elem));
        if let Some(cond) = &mut foreach.cond {
          self.expr(cond);
          self.push(Tac::Je(cond.reg, after_body));
        }
        self.break_stack.push(after_body);
        self.block(&mut foreach.body);
        self.break_stack.pop();
        self.push(Tac::Add(i, i, one));
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(after_body));
      }
      Guarded(guarded) => for (e, b) in &mut guarded.guarded {
        self.expr(e);
        let after_body = self.new_label();
        self.push(Tac::Je(e.reg, after_body));
        self.block(b);
        self.push(Tac::Label(after_body));
      }
      Block(block) => self.block(block),
    }
  }

  fn simple(&mut self, simple: &mut Simple) {
    match simple {
      Simple::Assign(assign) => {
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        match &assign.dst.data {
          ExprData::Id(id) => {
            let var_def = id.symbol.get();
            match var_def.scope.get().kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => { self.push(Tac::Assign(var_def.offset, assign.src.reg)); }
              ScopeKind::Class(_) => {
                self.push(Tac::Store { base: id.owner.as_ref().unwrap().reg, offset: (var_def.offset + 1) * INT_SIZE, src: assign.src.reg });
              }
              _ => unreachable!(),
            }
          }
          ExprData::Indexed(indexed) => {
            let (int_size, offset) = (self.new_reg(), self.new_reg());
            self.push(Tac::IntConst(int_size, INT_SIZE));
            self.push(Tac::Mul(offset, indexed.idx.reg, int_size));
            self.push(Tac::Add(offset, indexed.arr.reg, offset));
            self.push(Tac::Store { base: offset, offset: 0, src: assign.src.reg });
          }
          _ => unreachable!(),
        }
      }
      Simple::VarDef(var_def) => {
        var_def.offset = self.new_reg();
        if let Some(src) = &mut var_def.src {
          self.expr(src);
          self.push(Tac::Assign(var_def.offset, src.reg));
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
          ScopeKind::Local(_) | ScopeKind::Parameter(_) => expr.reg = var_def.offset,
          ScopeKind::Class(_) => {
            let owner = id.owner.as_mut().unwrap();
            self.expr(owner);
            expr.reg = self.new_reg();
            self.push(Tac::Load { dst: expr.reg, base: owner.reg, offset: (var_def.offset + 1) * INT_SIZE });
          }
          _ => unreachable!(),
        };
      }
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
        let check = self.check_array_index(indexed.arr.reg, indexed.idx.reg);
        let (halt, after) = (self.new_label(), self.new_label());
        self.push(Tac::Je(check, halt));
        expr.reg = self.array_at(indexed.arr.reg, indexed.idx.reg);
        self.push(Tac::Jmp(after));
        self.push(Tac::Label(halt));
        let msg = self.new_reg();
        self.push(Tac::StrConst(msg, quote(ARRAY_INDEX_OUT_OF_BOUND)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.intrinsic_call(HALT);
        self.push(Tac::Label(after));
      }
      IntConst(v) => {
        expr.reg = self.new_reg();
        self.push(Tac::IntConst(expr.reg, *v));
      }
      BoolConst(v) => {
        expr.reg = self.new_reg();
        self.push(Tac::IntConst(expr.reg, if *v { 1 } else { 0 }));
      }
      StringConst(v) => {
        expr.reg = self.new_reg();
        self.push(Tac::StrConst(expr.reg, quote(v)));
      }
      ArrayConst(_) => unimplemented!(),
      Null => {
        expr.reg = self.new_reg();
        self.push(Tac::IntConst(expr.reg, 0));
      }
      Call(call) => if call.is_arr_len {
        let owner = call.owner.as_mut().unwrap();
        self.expr(owner);
        expr.reg = self.array_length(owner.reg);
      } else {
        let method = call.method.get();
        let class = method.class.get();
        expr.reg = if method.ret_t.sem != VOID { self.new_reg() } else { -1 };
        if method.static_ {
          for arg in &mut call.arg {
            self.expr(arg);
            self.push(Tac::Param(arg.reg));
          }
          self.push(Tac::DirectCall(expr.reg, format!("_{}.{}", class.name, method.name)));
        } else {
          let owner = call.owner.as_mut().unwrap();
          self.expr(owner);
          self.push(Tac::Param(owner.reg));
          for arg in &mut call.arg {
            self.expr(arg);
            self.push(Tac::Param(arg.reg));
          }
          let slot = self.new_reg();
          self.push(Tac::Load { dst: slot, base: owner.reg, offset: 0 });
          self.push(Tac::Load { dst: slot, base: slot, offset: (method.offset + 2) * INT_SIZE });
          self.push(Tac::IndirectCall(expr.reg, slot));
        }
      }
      Unary(unary) => {
        self.expr(&mut unary.r);
        expr.reg = self.new_reg();
        match unary.op {
          Operator::Neg => self.push(Tac::Neg(expr.reg, unary.r.reg)),
          Operator::Not => self.push(Tac::Not(expr.reg, unary.r.reg)),
          _ => unimplemented!(),
        }
      }
      Binary(binary) => {
        use ast::Operator::*;
        self.expr(&mut binary.l);
        self.expr(&mut binary.r);
        expr.reg = self.new_reg();
        let (l, r, d) = (binary.l.reg, binary.r.reg, expr.reg);
        match binary.op {
          Add => self.push(Tac::Add(d, l, r)),
          Sub => self.push(Tac::Sub(d, l, r)),
          Mul => self.push(Tac::Mul(d, l, r)),
          Div => self.push(Tac::Div(d, l, r)),
          Mod => self.push(Tac::Mod(d, l, r)),
          Lt => self.push(Tac::Lt(d, l, r)),
          Le => self.push(Tac::Le(d, l, r)),
          Gt => self.push(Tac::Gt(d, l, r)),
          Ge => self.push(Tac::Ge(d, l, r)),
          And => self.push(Tac::And(d, l, r)),
          Or => self.push(Tac::Or(d, l, r)),
          Eq | Ne => if binary.l.type_ == STRING {
            self.push(Tac::Param(l));
            self.push(Tac::Param(r));
            expr.reg = self.intrinsic_call(STRING_EQUAL);
            if binary.op == Ne { self.push(Tac::Not(expr.reg, expr.reg)); }
          } else {
            self.push(if binary.op == Eq { Tac::Eq(d, l, r) } else { Tac::Ne(d, l, r) });
          }
          Repeat => {
            unimplemented!();
          }
          _ => unimplemented!(),
        }
      }
      This => expr.reg = self.cur_this,
      ReadInt => expr.reg = self.intrinsic_call(READ_INT),
      ReadLine => expr.reg = self.intrinsic_call(READ_LINE),
      NewClass { name } => {
        expr.reg = self.new_reg();
        self.push(Tac::DirectCall(expr.reg, format!("_{}_New", name)));
      }
      NewArray { elem_t: _, len } => {
        self.expr(len);
        let (halt, before_cond, finish) = (self.new_label(), self.new_label(), self.new_label());
        let (zero, int_size, cmp, i, msg) = (self.new_reg(), self.new_reg(), self.new_reg(), self.new_reg(), self.new_reg());
        self.push(Tac::IntConst(zero, 0));
        self.push(Tac::IntConst(int_size, INT_SIZE));
        self.push(Tac::Lt(cmp, len.reg, zero));
        self.push(Tac::Jne(cmp, halt));
        self.push(Tac::Mul(i, len.reg, int_size));
        self.push(Tac::Add(i, i, int_size)); // allocate (len + 1) * INT_SIZE
        self.push(Tac::Param(i));
        expr.reg = self.intrinsic_call(ALLOCATE);
        self.push(Tac::Add(i, expr.reg, i));
        self.push(Tac::Label(before_cond));
        self.push(Tac::Sub(i, i, int_size));
        self.push(Tac::Lt(cmp, i, expr.reg));
        self.push(Tac::Jne(cmp, finish));
        self.push(Tac::Store { base: i, offset: 0, src: zero });
        self.push(Tac::Jmp(before_cond));
        self.push(Tac::Label(halt));
        self.push(Tac::StrConst(msg, quote(ARRAY_INDEX_OUT_OF_BOUND)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.intrinsic_call(HALT);
        self.push(Tac::Label(finish));
        self.push(Tac::Store { base: i, offset: 0, src: len.reg }); // array[-1] = len
        self.push(Tac::Sub(expr.reg, expr.reg, int_size));
      }
      TypeTest { expr: src, name } => {
        self.expr(src);
        expr.reg = self.instance_of(src.reg, name);
      }
      TypeCast { name, expr: src } => {
        self.expr(src);
        expr.reg = src.reg;
        let check = self.instance_of(src.reg, name);
        let ok = self.new_label();
        self.push(Tac::Jne(check, ok));
        let msg = self.new_reg();
        self.push(Tac::StrConst(msg, quote(CLASS_CAST1)));
        self.push(Tac::Param(msg));
        self.intrinsic_call(PRINT_STRING);
        self.push(Tac::Load { dst: msg, base: src.reg, offset: INT_SIZE });
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
        expr.reg = self.new_reg();
        let (use_dft, after) = (self.new_label(), self.new_label());
        let check = self.check_array_index(default.arr.reg, default.idx.reg);
        self.push(Tac::Je(check, use_dft));
        let idx_res = self.array_at(default.arr.reg, default.idx.reg);
        self.push(Tac::Assign(expr.reg, idx_res));
        self.push(Tac::Jmp(after));
        self.push(Tac::Label(use_dft));
        self.expr(&mut default.dft);
        self.push(Tac::Assign(expr.reg, default.dft.reg));
        self.push(Tac::Label(after));
      }
      Comprehension(_) => unimplemented!(),
    }
  }
}
