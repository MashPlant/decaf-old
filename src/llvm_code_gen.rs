use super::ast::*;
use super::types::*;
use super::symbol::*;
use super::util::*;
use super::tac_code_gen::resolve_field_order;

use llvm_sys::*;
use llvm_sys::prelude::*;
use llvm_sys::support::*;
use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::target::*;

use std::ffi::CString;
use std::ptr;

macro_rules! cstr {
  ($s: expr) => { concat!($s, "\0") as *const str as *const [libc::c_char] as *const libc::c_char }
}

macro_rules! cstring {
  ($s: expr) => { CString::new($s).unwrap().as_ptr() }
}

const T: *const libc::c_char = cstr!("");

pub struct LLVMCodeGen {
  context: LLVMContextRef,
  module: LLVMModuleRef,
  builder: LLVMBuilderRef,
  i32_t: LLVMTypeRef,
  i8_t: LLVMTypeRef,
  void_t: LLVMTypeRef,
  str_t: LLVMTypeRef,
  cur_this: LLVMValueRef,
  // c std library functions(malloc, memset can be built without declare)
  printf: LLVMValueRef,
  scanf: LLVMValueRef,
  strcmp: LLVMValueRef,
}

impl LLVMCodeGen {
  pub fn gen(mut program: Program) {
    unsafe {
      let context = LLVMContextCreate();
      let module = LLVMModuleCreateWithNameInContext(cstr!("Decaf Program"), context);
      let builder = LLVMCreateBuilderInContext(context);
      let i32_t = LLVMInt32TypeInContext(context);
      let i8_t = LLVMInt8TypeInContext(context);
      let void_t = LLVMVoidTypeInContext(context);
      let str_t = ptr_of(i8_t);
      let printf = LLVMAddFunction(module, cstr!("printf"), LLVMFunctionType(i32_t, [str_t].as_mut_ptr(), 1, 1));
      let scanf = LLVMAddFunction(module, cstr!("scanf"), LLVMFunctionType(i32_t, [str_t].as_mut_ptr(), 1, 1));
      let strcmp = LLVMAddFunction(module, cstr!("strcmp"), LLVMFunctionType(i32_t, [str_t, str_t].as_mut_ptr(), 2, 0));
      let mut code_gen = LLVMCodeGen { context, module, builder, i32_t, i8_t, void_t, str_t, cur_this: ptr::null_mut(), printf, scanf, strcmp };
      code_gen.program(&mut program);
      LLVMDisposeBuilder(code_gen.builder);
      LLVMDumpModule(code_gen.module);
    }
  }
}

unsafe fn ptr_of(type_: LLVMTypeRef) -> LLVMTypeRef {
  LLVMPointerType(type_, 0)
}

impl LLVMCodeGen {
  unsafe fn type_of(&self, type_: &SemanticType) -> LLVMTypeRef {
    match type_ {
      SemanticType::Int => self.i32_t,
      SemanticType::Bool => self.i8_t,
      SemanticType::Void => self.void_t,
      SemanticType::String => self.str_t,
      SemanticType::Object(class) => ptr_of(class.get().llvm_t),
      SemanticType::Array(elem) => ptr_of(self.type_of(elem)),
      _ => unreachable!(),
    }
  }

  // declare & define struct type
  // declare v table type & val
  unsafe fn make_struct_type(&self, class: &mut ClassDef) {
    if !class.llvm_t.is_null() { return; }
    if !class.p_ptr.is_null() { self.make_struct_type(class.p_ptr.get()); }
    class.llvm_t = LLVMStructCreateNamed(self.context, cstring!(class.name));
    class.llvm_v_tbl_t = LLVMStructCreateNamed(self.context, cstring!(format!("{}_VTableT", class.name)));
    let mut elem_t;
    if class.p_ptr.is_null() {
      elem_t = vec![ptr_of(class.llvm_v_tbl_t)];
    } else {
      elem_t = Vec::with_capacity(class.field_cnt as usize + 1);
      elem_t.set_len(class.p_ptr.get().field_cnt as usize + 1);
      LLVMGetStructElementTypes(class.p_ptr.get().llvm_t, elem_t.as_mut_ptr());
      elem_t[0] = ptr_of(class.llvm_v_tbl_t);
    }
    for field in &mut class.field {
      if let FieldDef::VarDef(var) = field {
        elem_t.push(self.type_of(&var.type_));
      }
    }
    LLVMStructSetBody(class.llvm_t, elem_t.as_mut_ptr(), elem_t.len() as u32, 0);
    class.llvm_v_tbl = LLVMAddGlobal(self.module, class.llvm_v_tbl_t, cstring!(format!("{}_VTable", class.name)));
  }

  unsafe fn program(&mut self, program: &mut Program) {
    // build struct type
    for class in &mut program.class {
      resolve_field_order(class);
      self.make_struct_type(class);
    }
    // parsing method
    for class in &mut program.class {
      for field in &mut class.field {
        if let FieldDef::MethodDef(method) = field {
          self.method(method);
        }
      }
    }
    // build v table type & val, type & val of method is set above
    // class.llvm_v_tbl_t & class.llvm_v_tbl is declared above, but not defined
    for class in &mut program.class {
      // actually first element is not string, just a meaningless i8*
      let mut v_tbl_elem_t = vec![if class.p_ptr.is_null() { self.str_t } else { ptr_of(class.p_ptr.get().llvm_v_tbl_t) }, self.str_t];
      for method in &class.v_tbl.methods {
        v_tbl_elem_t.push(method.get().llvm_t);
      }
      LLVMStructSetBody(class.llvm_v_tbl_t, v_tbl_elem_t.as_mut_ptr(), v_tbl_elem_t.len() as u32, 0);
    }
  }

  unsafe fn method(&mut self, method: &mut MethodDef) {
    let (context, builder) = (self.context, self.builder);
    method.llvm_t = ptr_of({
      let mut param_t = method.param.iter().map(|a| self.type_of(&a.type_)).collect::<Vec<_>>();
      LLVMFunctionType(self.type_of(&method.ret_t), param_t.as_mut_ptr(), param_t.len() as u32, 0)
    });
    method.llvm_val = LLVMAddFunction(self.module, cstring!(format!("{}_{}", method.class.get().name, method.name)), method.llvm_t);
    let bb = LLVMAppendBasicBlockInContext(context, method.llvm_val, cstr!("entry"));
    LLVMPositionBuilderAtEnd(builder, bb);
    for (index, param) in method.param.iter_mut().enumerate() {
      param.llvm_val = LLVMGetParam(method.llvm_val, index as u32);
    }
    if !method.static_ {
      self.cur_this = method.param[0].llvm_val;
    }
    self.block(&mut method.body);
    match &method.ret_t.sem {
      SemanticType::Int => LLVMBuildRet(builder, LLVMConstInt(self.i32_t, 0, 0)),
      SemanticType::Bool => LLVMBuildRetVoid(builder),
      SemanticType::String => LLVMBuildRetVoid(builder),
      SemanticType::Void => LLVMBuildRetVoid(builder),
      SemanticType::Object(class) => LLVMBuildRetVoid(builder),
      SemanticType::Array(elem) => LLVMBuildRetVoid(builder),
      _ => unreachable!(),
    };
  }

  unsafe fn stmt(&mut self, stmt: &mut Stmt) {
    match stmt {
      Stmt::Simple(simple) => self.simple(simple),
      Stmt::If(if_) => {}
      Stmt::While(while_) => {}
      Stmt::For(for_) => {}
      Stmt::Return(return_) => {}
      Stmt::Print(print) => {}
      Stmt::Break(break_) => {}
      Stmt::SCopy(s_copy) => {}
      Stmt::Foreach(foreach) => {}
      Stmt::Guarded(guarded) => {}
      Stmt::Block(block) => self.block(block),
    }
  }

  unsafe fn block(&mut self, block: &mut Block) {
    for stmt in &mut block.stmt {
      self.stmt(stmt);
    }
  }

  unsafe fn simple(&mut self, simple: &mut Simple) {
    let builder = self.builder;
    match simple {
      Simple::Assign(assign) => {
        self.expr(&mut assign.dst);
        self.expr(&mut assign.src);
        match &assign.dst.data {
          ExprData::Id(id) => {
            let var_def = id.symbol.get();
            match var_def.scope.get().kind {
              ScopeKind::Local(_) | ScopeKind::Parameter(_) => {
                LLVMBuildStore(builder, assign.src.llvm_val, var_def.llvm_val);
              }
              ScopeKind::Class(_) => {
                let base = id.owner.as_ref().unwrap().llvm_val;
                let elem_ptr = LLVMBuildStructGEP(builder, base, var_def.offset as u32 + 1, T);
                LLVMBuildStore(builder, assign.src.llvm_val, elem_ptr);
              }
              _ => unreachable!(),
            }
          }
          ExprData::Indexed(indexed) => {
            let ptr = LLVMBuildAdd(builder, indexed.arr.llvm_val, indexed.idx.llvm_val, T);
            LLVMBuildStore(builder, assign.src.llvm_val, ptr);
          }
          _ => unreachable!(),
        }
      }
      Simple::VarDef(var_def) => {
        var_def.llvm_val = LLVMBuildAlloca(builder, self.type_of(&var_def.type_), T);
        if let Some(src) = &mut var_def.src {
          self.expr(src);
          LLVMBuildStore(builder, src.llvm_val, var_def.llvm_val);
        }
      }
      Simple::Expr(expr) => self.expr(expr),
      Simple::Skip => {}
    }
  }

  unsafe fn expr(&self, expr: &mut Expr) {
    use ast::ExprData::*;
    let builder = self.builder;
    match &mut expr.data {
      Id(id) => {
        let var_def = id.symbol.get();
        expr.llvm_val = match var_def.scope.get().kind {
          ScopeKind::Local(_) | ScopeKind::Parameter(_) => LLVMBuildLoad(builder, var_def.llvm_val, T),
          ScopeKind::Class(_) => {
            let owner = id.owner.as_mut().unwrap();
            self.expr(owner);
            LLVMBuildLoad(builder, LLVMBuildStructGEP(builder, owner.llvm_val, var_def.offset as u32 + 1, T), T)
          }
          _ => unreachable!(),
        }
      }
      Indexed(indexed) => {
        self.expr(&mut indexed.arr);
        self.expr(&mut indexed.idx);
//        let check = self.check_array_index(indexed.arr.tac_reg, indexed.idx.tac_reg);
//        self.out_of_bound_to_fill.push(self.cur_method.get().len() as i32);
//        self.push(Tac::Je(check, -1)); // jump where not determined yet
//        if !indexed.for_assign { // not used, but still check index here
//          expr.tac_reg = self.array_at(indexed.arr.tac_reg, indexed.idx.tac_reg);
//        }
      }
      IntConst(v) => {
//        expr.tac_reg = self.new_reg();
//        self.push(Tac::IntConst(expr.tac_reg, *v));
      }
      BoolConst(v) => {
//        expr.tac_reg = if *v { self.int_const(1) } else { self.int_const(0) };
      }
      StringConst(v) => {
//        expr.tac_reg = self.new_reg();
//        self.push(Tac::StrConst(expr.tac_reg, quote(v)));
      }
      ArrayConst(_) => unimplemented!(),
      Null => {
//        expr.tac_reg = self.int_const(0);
      }
      Call(call) => {
//        if call.is_arr_len {
//          let owner = call.owner.as_mut().unwrap();
//          self.expr(owner);
//          expr.tac_reg = self.array_length(owner.tac_reg);
//        } else {
//          let method = call.method.get();
//          let class = method.class.get();
//          expr.tac_reg = if method.ret_t.sem != VOID { self.new_reg() } else { -1 };
//          if method.static_ {
//            // fuck vm spec
//            // 'parm' can only be consecutive for one call, there cannot be other call in this process
//            // e.g. my old version: f(1, x[0]) : parm 1 => (if <out of bound> call _Halt) => parm x[0]
//            // which caused error
//            for arg in &mut call.arg { self.expr(arg); }
//            for arg in &mut call.arg { self.push(Tac::Param(arg.tac_reg)); }
//            self.push(Tac::DirectCall(expr.tac_reg, format!("_{}.{}", class.name, method.name)));
//          } else {
//            let owner = call.owner.as_mut().unwrap();
//            self.expr(owner);
//            for arg in &mut call.arg { self.expr(arg); }
//            self.push(Tac::Param(owner.tac_reg));
//            for arg in &mut call.arg { self.push(Tac::Param(arg.tac_reg)); }
//            let slot = self.new_reg();
//            self.push(Tac::Load(slot, owner.tac_reg, 0));
//            self.push(Tac::Load(slot, slot, (method.offset + 2) * INT_SIZE));
//            self.push(Tac::IndirectCall(expr.tac_reg, slot));
//          }
//        }
      }
      Unary(unary) => {
        self.expr(&mut unary.r);
        expr.llvm_val = match unary.op {
          Operator::Neg => LLVMBuildNeg(builder, unary.r.llvm_val, T),
          Operator::Not => LLVMBuildNot(builder, unary.r.llvm_val, T),
          _ => unimplemented!(),
        }
      }
      Binary(binary) => {
        use ast::Operator::*;
        self.expr(&mut binary.l);
        self.expr(&mut binary.r);
        let (l, r) = (binary.l.llvm_val, binary.r.llvm_val);
        expr.llvm_val = match binary.op {
          Add => LLVMBuildAdd(builder, l, r, T),
          Sub => LLVMBuildSub(builder, l, r, T),
          Mul => LLVMBuildMul(builder, l, r, T),
          Div => LLVMBuildSDiv(builder, l, r, T),
          Lt => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLT, l, r, T),
          Le => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSLE, l, r, T),
          Gt => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGT, l, r, T),
          Ge => LLVMBuildICmp(builder, LLVMIntPredicate::LLVMIntSGE, l, r, T),
          And => {
            unimplemented!();
          }
          Or => {
            unimplemented!();
          }
          Eq => {
            unimplemented!();
          }
          Ne => {
            unimplemented!();
          }
          Repeat => {
            unimplemented!();
          }
          _ => unimplemented!(),
        }
      }
      This => expr.llvm_val = self.cur_this,
      ReadInt => {
//        expr.tac_reg = self.intrinsic_call(READ_INT);
      }
      ReadLine => {
//        expr.tac_reg = self.intrinsic_call(READ_LINE);
      }
      NewClass { name } => {
//        expr.tac_reg = self.new_reg();
//        self.push(Tac::DirectCall(expr.tac_reg, format!("_{}_New", name)));
      }
      NewArray { elem_t: _, len } => {
        self.expr(len);
//        let (halt, before_cond, finish) = (self.new_label(), self.new_label(), self.new_label());
//        let (zero, int_size, cmp, i, msg) = (self.int_const(0), self.int_const(INT_SIZE), self.new_reg(), self.new_reg(), self.new_reg());
//        self.push(Tac::Lt(cmp, len.tac_reg, zero));
//        self.push(Tac::Jne(cmp, halt));
//        self.push(Tac::Mul(i, len.tac_reg, int_size));
//        self.push(Tac::Add(i, i, int_size)); // allocate (len + 1) * INT_SIZE
//        self.push(Tac::Param(i));
//        expr.tac_reg = self.intrinsic_call(ALLOCATE);
//        self.push(Tac::Add(i, expr.tac_reg, i));
//        self.push(Tac::Label(before_cond));
//        self.push(Tac::Sub(i, i, int_size));
//        self.push(Tac::Eq(cmp, i, expr.tac_reg));
//        self.push(Tac::Jne(cmp, finish));
//        self.push(Tac::Store(i, 0, zero));
//        self.push(Tac::Jmp(before_cond));
//        self.push(Tac::Label(halt));
//        self.push(Tac::StrConst(msg, quote(NEGATIVE_ARRAY_SIZE)));
//        self.push(Tac::Param(msg));
//        self.intrinsic_call(PRINT_STRING);
//        self.intrinsic_call(HALT);
//        self.push(Tac::Label(finish));
//        self.push(Tac::Store(i, 0, len.tac_reg)); // array[-1] = len
//        self.push(Tac::Add(expr.tac_reg, expr.tac_reg, int_size));
      }
      TypeTest { expr: src, name } => {
        self.expr(src);
//        expr.tac_reg = self.instance_of(src.tac_reg, name);
      }
      TypeCast { name, expr: src } => {
        self.expr(src);
//        expr.tac_reg = src.tac_reg;
//        let check = self.instance_of(src.tac_reg, name);
//        let ok = self.new_label();
//        let (msg, v_tbl) = (self.new_reg(), self.new_reg());
//        self.push(Tac::Jne(check, ok));
//        self.push(Tac::StrConst(msg, quote(CLASS_CAST1)));
//        self.push(Tac::Param(msg));
//        self.intrinsic_call(PRINT_STRING);
//        self.push(Tac::Load(v_tbl, src.tac_reg, 0));
//        self.push(Tac::Load(msg, v_tbl, INT_SIZE)); // name info is in v-table[1]
//        self.push(Tac::Param(msg));
//        self.intrinsic_call(PRINT_STRING);
//        self.push(Tac::StrConst(msg, quote(CLASS_CAST2)));
//        self.push(Tac::Param(msg));
//        self.intrinsic_call(PRINT_STRING);
//        self.push(Tac::StrConst(msg, quote(name)));
//        self.push(Tac::Param(msg));
//        self.intrinsic_call(PRINT_STRING);
//        self.push(Tac::StrConst(msg, quote(CLASS_CAST3)));
//        self.push(Tac::Param(msg));
//        self.intrinsic_call(PRINT_STRING);
//        self.intrinsic_call(HALT);
//        self.push(Tac::Label(ok));
      }
      Range(_) => unimplemented!(),
      Default(default) => {
        self.expr(&mut default.arr);
        self.expr(&mut default.idx);
//        expr.tac_reg = self.new_reg();
//        let (use_dft, after) = (self.new_label(), self.new_label());
//        let check = self.check_array_index(default.arr.tac_reg, default.idx.tac_reg);
//        self.push(Tac::Je(check, use_dft));
//        let idx_res = self.array_at(default.arr.tac_reg, default.idx.tac_reg);
//        self.push(Tac::Assign(expr.tac_reg, idx_res));
//        self.push(Tac::Jmp(after));
//        self.push(Tac::Label(use_dft));
//        self.expr(&mut default.dft);
//        self.push(Tac::Assign(expr.tac_reg, default.dft.tac_reg));
//        self.push(Tac::Label(after));
      }
      Comprehension(_) => unimplemented!(),
    };
  }
}