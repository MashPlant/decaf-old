use super::ast::*;
use super::types::*;
use std::io;

pub fn quote(s: &str) -> String {
  let mut ret = "\"".to_owned();
  for ch in s.chars() {
    match ch {
      '"' => ret.push_str("\\\""),
      '\n' => ret.push_str("\\n"),
      '\t' => ret.push_str("\\t"),
      '\\' => ret.push_str("\\\\"),
      ch => ret.push(ch),
    };
  }
  ret + "\""
}

pub struct IndentPrinter {
  newline: bool,
  indent: String,
  content: String,
}

impl IndentPrinter {
  pub fn new() -> IndentPrinter {
    IndentPrinter {
      newline: false,
      indent: String::new(),
      content: String::new(),
    }
  }

  pub fn clear(&mut self) {
    self.newline = false;
    self.indent.clear();
    self.content.clear();
  }

  pub fn pop_space(&mut self) -> &mut Self {
    if self.content.ends_with(" ") { self.content.pop(); }
    self
  }

  pub fn inc_indent(&mut self) -> &mut Self {
    self.indent += "    ";
    self
  }

  pub fn dec_indent(&mut self) -> &mut Self {
    for _ in 0..4 {
      self.indent.pop();
    }
    self
  }

  // automatic add a space
  pub fn print(&mut self, s: &str) -> &mut Self {
    if self.newline { self.content += &self.indent; }
    self.content += s;
    self.content += " ";
    self.newline = false;
    self
  }

  pub fn println(&mut self, s: &str) -> &mut Self {
    if self.newline { self.content += &self.indent; }
    self.content += s;
    self.content += "\n";
    self.newline = true;
    self
  }

  fn accept<T: ASTData>(&mut self, t: &T) -> &mut Self {
    t.print_ast(self);
    self
  }

  pub fn newline(&mut self) -> &mut Self {
    self.pop_space();
    self.content += "\n";
    self.newline = true;
    self
  }

  pub fn flush<W: io::Write>(&mut self, writer: &mut W) {
    self.pop_space();
    writer.write(self.content.as_bytes()).unwrap();
  }
}

pub trait ASTData {
  fn print_ast(&self, p: &mut IndentPrinter);
}

macro_rules! make_ast_data {
  ($self_: ident, $printer: ident, $($kind: ident => $body: block),*) => {
    $(impl ASTData for $kind {
      fn print_ast(&$self_, $printer: &mut IndentPrinter) {
        $body
      }
    })*
  };
}

make_ast_data!(self, p,
  Program => {
    p.println("program").inc_indent();
    for class in &self.class { class.print_ast(p); }
    p.dec_indent();
  },
  ClassDef => {
    if self.sealed { p.print("sealed"); }
    p.print("class").print(self.name);
    match self.parent {
      Some(name) => p.print(&name),
      None => p.print("<empty>"),
    };
    p.newline().inc_indent();
    for field in &self.field { field.print_ast(p); }
    p.dec_indent();
  },
  FieldDef => {
    match &self {
      FieldDef::MethodDef(method_def) => method_def.print_ast(p),
      FieldDef::VarDef(var_def) => var_def.print_ast(p),
    }
  },
  MethodDef => {
    if self.static_ { p.print("static"); }
    p.print("func").print(self.name).accept(&self.ret_t.sem).newline().inc_indent().println("formals").inc_indent();
    for parameter in &self.param {
      parameter.print_ast(p);
    }
    p.dec_indent().accept(&self.body).dec_indent();
  },
  VarDef => {
    if self.type_.sem == VAR {
      p.println("assign").inc_indent().print("var").println(self.name);
      if let Some(src) = &self.src { src.print_ast(p); } else { unreachable!(); }
      p.dec_indent();
    } else {
      p.print("vardef").print(self.name).accept(&self.type_.sem).newline();
      if let Some(src) = &self.src {
        p.println("assign").inc_indent().println(self.name).accept(src).dec_indent();
        src.print_ast(p);
      }
    }
  },
  Stmt => {
    use ast::Stmt::*;
    match &self {
      Simple(simple) => simple.print_ast(p),
      If(if_) => if_.print_ast(p),
      While(while_) => while_.print_ast(p),
      For(for_) => for_.print_ast(p),
      Return(return_) => return_.print_ast(p),
      Print(print) => print.print_ast(p),
      Break(break_) => break_.print_ast(p),
      SCopy(object_copy) => object_copy.print_ast(p),
      Foreach(foreach) => foreach.print_ast(p),
      Guarded(guarded) => guarded.print_ast(p),
      Block(block) => block.print_ast(p),
    };
  },
  Simple => {
    match &self {
      Simple::Assign(assign) => assign.print_ast(p),
      Simple::VarDef(var_def) => var_def.print_ast(p),
      Simple::Expr(expr) => expr.print_ast(p),
      Simple::Skip => {},
    }
  },
  Block => {
    p.println("stmtblock").inc_indent();
    for stmt in &self.stmt { stmt.print_ast(p); }
    p.dec_indent();
  },
  If => {
    p.println("if").inc_indent().accept(&self.cond).accept(&self.on_true).dec_indent();
    if let Some(on_false) = &self.on_false {
      p.println("else").inc_indent().accept(on_false).dec_indent();
    }
  },
  While => { p.println("while").inc_indent().accept(&self.cond).accept(&self.body).dec_indent(); },
  For => { p.println("for").inc_indent().accept(&self.init).accept(&self.cond).accept(&self.update).accept(&self.body).dec_indent(); },
  Foreach => {
    p.println("foreach").inc_indent().print("varbind").print(self.def.name).accept(&self.def.type_.sem).newline().accept(&self.arr);
    match &self.cond {
      Some(cond) => cond.print_ast(p),
      None => { p.println("boolconst true"); }
    }
    p.accept(&self.body).dec_indent();
  },
  Return => {
    p.println("return");
    if let Some(expr) = &self.expr {
      p.inc_indent().accept(expr).dec_indent();
    }
  },
  Print => {
    p.println("print").inc_indent();
    for expr in &self.print { expr.print_ast(p) }
    p.dec_indent();
  },
  Break => { p.println("break"); },
  Assign => { p.println("assign").inc_indent().accept(&self.dst).accept(&self.src).dec_indent(); },
  SCopy => { p.println("scopy").inc_indent().println(self.dst).accept(&self.src).dec_indent(); },
  Guarded => {
    p.println("guarded").inc_indent();
    if self.guarded.is_empty() {
      p.println("<empty>");
    } else {
      for (e, b) in &self.guarded {
        p.println("guard").inc_indent().accept(e).accept(b).dec_indent();
      }
    }
    p.dec_indent();
  },
  Expr => {
    use ast::ExprData::*;
    match &self.data {
      Id(id) => id.print_ast(p),
      Indexed(indexed) => indexed.print_ast(p),
      IntConst(v) => { p.print("intconst").println(&v.to_string()); }
      BoolConst(v) => { p.print("boolconst").println(&v.to_string()); }
      StringConst(v) => { p.print("stringconst").println(&quote(&v)); }
      ArrayConst(v) => {
        p.println("array const").inc_indent();
        if v.is_empty() {
          p.println("<empty>");
        } else {
          for expr in v { expr.print_ast(p); }
        }
        p.dec_indent();
      }
      Null => { p.println("null"); },
      Call(call) => call.print_ast(p),
      Unary(unary) => unary.print_ast(p),
      Binary(binary) => binary.print_ast(p),
      This => { p.println("this"); },
      ReadInt => { p.println("readint"); },
      ReadLine => { p.println("readline"); },
      NewClass{ name } => { p.print("newobj").println(name); },
      NewArray{ elem_t, len } => { p.print("newarray").accept(&elem_t.sem).newline().inc_indent().accept(len.as_ref()).dec_indent(); }
      TypeTest{ expr, name, target_class: _ } => { p.println("instanceof").inc_indent().accept(expr.as_ref()).println(name).dec_indent(); }
      TypeCast{ name, expr } => { p.println("classcast").inc_indent().println(name).accept(expr.as_ref()).dec_indent(); }
      Range(range) => range.print_ast(p),
      Default(default) => default.print_ast(p),
      Comprehension(comprehension) => comprehension.print_ast(p),
    };
  },
  Id => {
    p.print("varref").println(self.name);
    if let Some(owner) = &self.owner {
      p.inc_indent().accept(owner.as_ref()).dec_indent();
    }
  },
  Indexed => { p.println("arrref").inc_indent().accept(self.arr.as_ref()).accept(self.idx.as_ref()).dec_indent(); },
  Call => {
    p.print("call").println(self.name).inc_indent();
    match &self.owner {
      Some(receiver) => receiver.print_ast(p),
      None => { p.println("<empty>"); }
    };
    for expr in &self.arg { expr.print_ast(p); }
    p.dec_indent();
  },
  Unary => {
    use ast::Operator::*;
    let opname = match self.op {
      Neg => "neg",
      Not => "not",
      PreInc => "preinc",
      PreDec => "predec",
      PostInc => "postinc",
      PostDec => "postdec",
      _ => unreachable!(),
    };
    p.println(opname).inc_indent().accept(self.r.as_ref()).dec_indent();
  },
  Binary => {
    use self::Operator::*;
    let opname = match self.op {
      Add => "add",
      Sub => "sub",
      Mul => "mul",
      Div => "div",
      Mod => "mod",
      And => "and",
      Or => "or",
      Eq => "equ",
      Ne => "neq",
      Lt => "les",
      Le => "leq",
      Gt => "gtr",
      Ge => "geq",
      Repeat => "array repeat",
      /* unimplemented */ Concat => "array concat",
      BAnd => "bitand",
      BOr => "bitand",
      BXor => "bitxor",
      Shl => "shl",
      Shr => "shr",
      _ => unreachable!(),
    };
    p.println(opname).inc_indent().accept(self.l.as_ref()).accept(self.r.as_ref()).dec_indent();
  },
  Range => { p.println("arrref").inc_indent().accept(self.arr.as_ref()).println("range").inc_indent().accept(self.lb.as_ref()).accept(self.ub.as_ref()).dec_indent().dec_indent(); },
  Default => { p.println("arrref").inc_indent().accept(self.arr.as_ref()).accept(self.idx.as_ref()).println("default").inc_indent().accept(self.dft.as_ref()).dec_indent().dec_indent(); },
  Comprehension => {
    p.println("array comp").inc_indent().print("varbind").println(self.name).accept(self.arr.as_ref());
    match &self.cond {
      Some(cond) => cond.print_ast(p),
      None => { p.println("boolconst true"); }
    };
    p.accept(self.expr.as_ref()).dec_indent();
  }
);

pub trait ScopeData {
  fn print_scope(&self, p: &mut IndentPrinter);
}

impl ScopeData for Program {
  fn print_scope(&self, p: &mut IndentPrinter) {
    p.println("GLOBAL SCOPE:").inc_indent();
    for symbol in self.scope.sorted() {
      p.println(&symbol.to_string());
    }
    for class in &self.class {
      class.print_scope(p);
    }
    p.dec_indent();
  }
}

impl ScopeData for ClassDef {
  fn print_scope(&self, p: &mut IndentPrinter) {
    p.println(&format!("CLASS SCOPE OF '{}':", self.name)).inc_indent();
    for symbol in self.scope.sorted() {
      p.println(&symbol.to_string());
    }
    for field_def in &self.field {
      if let FieldDef::MethodDef(method_def) = field_def {
        method_def.print_scope(p);
      }
    }
    p.dec_indent();
  }
}

impl ScopeData for MethodDef {
  fn print_scope(&self, p: &mut IndentPrinter) {
    p.println(&format!("FORMAL SCOPE OF '{}':", self.name)).inc_indent();
    for symbol in self.scope.sorted() {
      p.println(&symbol.to_string());
    }
    p.println("LOCAL SCOPE:");
    self.body.print_scope(p);
    p.dec_indent();
  }
}

impl ScopeData for Block {
  fn print_scope(&self, p: &mut IndentPrinter) {
    p.inc_indent();
    for symbol in self.scope.sorted() {
      p.println(&symbol.to_string());
    }
    for stmt in &self.stmt {
      if let Stmt::Block(block) = stmt {
        block.print_scope(p)
      }
    }
    p.dec_indent();
  }
}