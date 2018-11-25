use super::ast::*;
use std::io;

pub fn quote(s: &str) -> String {
  let mut ret = "\"".to_string();
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

  pub fn pop_space(&mut self) {
    if self.content.ends_with(" ") { self.content.pop(); }
  }

  pub fn inc_indent(&mut self) {
    self.indent += "    ";
  }

  pub fn dec_indent(&mut self) {
    for _ in 0..4 {
      self.indent.pop();
    }
  }

  // automatic add a space
  pub fn print(&mut self, s: &str) {
    if self.newline { self.content += &self.indent; }
    self.content += s;
    self.content += " ";
    self.newline = false;
  }

  pub fn println(&mut self, s: &str) {
    if self.newline { self.content += &self.indent; }
    self.content += s;
    self.content += "\n";
    self.newline = true;
  }

  pub fn newline(&mut self) {
    self.pop_space();
    self.content += "\n";
    self.newline = true;
  }

  pub fn flush<W: io::Write>(&mut self, writer: &mut W) {
    self.pop_space();
    writer.write(self.content.as_bytes()).unwrap();
  }
}

pub trait ASTData {
  fn print_ast(&self, printer: &mut IndentPrinter);
}

pub trait ScopeData {
  fn print_scope(&self, printer: &mut IndentPrinter);
}

impl ASTData for Program {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("program");
    printer.inc_indent();
    for class in &self.classes { class.print_ast(printer); }
    printer.dec_indent();
  }
}

impl ScopeData for Program {
  fn print_scope(&self, printer: &mut IndentPrinter) {
    printer.println("GLOBAL SCOPE:");
    printer.inc_indent();
    for symbol in self.scope.sorted() {
      printer.println(&symbol.to_string());
    }
    for class in &self.classes {
      class.print_scope(printer);
    }
    printer.dec_indent();
  }
}

impl ASTData for ClassDef {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    if self.sealed { printer.print("sealed"); }
    printer.print("class");
    printer.print(self.name);
    match self.parent {
      Some(name) => printer.print(&name),
      None => printer.print("<empty>"),
    };
    printer.newline();
    printer.inc_indent();
    for field in &self.fields { field.print_ast(printer); }
    printer.dec_indent();
  }
}

impl ScopeData for ClassDef {
  fn print_scope(&self, printer: &mut IndentPrinter) {
    printer.println(&format!("CLASS SCOPE OF '{}':", self.name));
    printer.inc_indent();
    for symbol in self.scope.sorted() {
      printer.println(&symbol.to_string());
    }
    for field_def in &self.fields {
      if let FieldDef::MethodDef(method_def) = field_def {
        method_def.print_scope(printer);
      }
    }
    printer.dec_indent();
  }
}

impl ASTData for FieldDef {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    match &self {
      FieldDef::MethodDef(method_def) => method_def.print_ast(printer),
      FieldDef::VarDef(var_def) => var_def.print_ast(printer),
    }
  }
}

impl ASTData for MethodDef {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    if self.static_ { printer.print("static"); }
    printer.print("func");
    printer.print(self.name);
    self.ret_t.print_ast(printer);
    printer.newline();
    printer.inc_indent();
    printer.println("formals");
    printer.inc_indent();
    for parameter in &self.params {
      parameter.print_ast(printer);
    }
    printer.dec_indent();
    self.body.print_ast(printer);
    printer.dec_indent();
  }
}

impl ScopeData for MethodDef {
  fn print_scope(&self, printer: &mut IndentPrinter) {
    printer.println(&format!("FORMAL SCOPE OF '{}':", self.name));
    printer.inc_indent();
    for symbol in self.scope.sorted() {
      printer.println(&symbol.to_string());
    }
    printer.println("LOCAL SCOPE:");
    self.body.print_scope(printer);
    printer.dec_indent();
  }
}

impl ASTData for VarDef {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("vardef");
    printer.print(self.name);
    self.type_.print_ast(printer);
    printer.newline();
  }
}

impl ASTData for Stmt {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    use ast::Stmt::*;
    match &self {
      VarDef(var_def) => var_def.print_ast(printer),
      Simple(simple) => simple.print_ast(printer),
      If(if_) => if_.print_ast(printer),
      While(while_) => while_.print_ast(printer),
      For(for_) => for_.print_ast(printer),
      Return(return_) => return_.print_ast(printer),
      Print(print) => print.print_ast(printer),
      Break(break_) => break_.print_ast(printer),
      SCopy(object_copy) => object_copy.print_ast(printer),
      Foreach(foreach) => foreach.print_ast(printer),
      Guarded(guarded) => guarded.print_ast(printer),
      Block(block) => block.print_ast(printer),
    };
  }
}

impl ASTData for Simple {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    match &self {
      Simple::Assign(assign) => assign.print_ast(printer),
      Simple::VarAssign(var_assign) => var_assign.print_ast(printer),
      Simple::Expr(expr) => expr.print_ast(printer),
      Simple::Skip(skip) => skip.print_ast(printer),
    }
  }
}

impl ASTData for Block {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("stmtblock");
    printer.inc_indent();
    for stmt in &self.stmts {
      stmt.print_ast(printer);
    }
    printer.dec_indent();
  }
}

impl ScopeData for Block {
  fn print_scope(&self, printer: &mut IndentPrinter) {
    printer.inc_indent();
    for symbol in self.scope.sorted() {
      printer.println(&symbol.to_string());
    }
    for stmt in &self.stmts {
      if let Stmt::Block(block) = stmt {
        block.print_scope(printer)
      }
    }
    printer.dec_indent();
  }
}

impl ASTData for If {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("if");
    printer.inc_indent();
    self.cond.print_ast(printer);
    self.on_true.print_ast(printer);
    printer.dec_indent();
    if let Some(on_false) = &self.on_false {
      printer.println("else");
      printer.inc_indent();
      on_false.print_ast(printer);
      printer.dec_indent();
    }
  }
}

impl ASTData for While {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("while");
    printer.inc_indent();
    self.cond.print_ast(printer);
    self.body.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for For {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("for");
    printer.inc_indent();
    self.init.print_ast(printer);
    self.cond.print_ast(printer);
    self.update.print_ast(printer);
    self.body.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Foreach {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("foreach");
    printer.inc_indent();
    printer.print("varbind");
    printer.print(self.var_def.name);
    self.var_def.type_.print_ast(printer);
    printer.newline();
    self.array.print_ast(printer);
    match &self.cond {
      Some(cond) => cond.print_ast(printer),
      None => printer.println("boolconst true"),
    }
    self.body.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Return {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("return");
    if let Some(expr) = &self.expr {
      printer.inc_indent();
      expr.print_ast(printer);
      printer.dec_indent();
    }
  }
}

impl ASTData for Print {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("print");
    printer.inc_indent();
    for expr in &self.print { expr.print_ast(printer) }
    printer.dec_indent();
  }
}

impl ASTData for Break {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("break");
  }
}

impl ASTData for SCopy {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("scopy");
    printer.inc_indent();
    printer.println(self.dst);
    self.src.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Guarded {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("guarded");
    printer.inc_indent();
    if self.guarded.is_empty() {
      printer.println("<empty>");
    } else {
      for (e, b) in &self.guarded {
        printer.println("guard");
        printer.inc_indent();
        e.print_ast(printer);
        b.print_ast(printer);
        printer.dec_indent();
      }
    }
    printer.dec_indent();
  }
}

impl ASTData for Assign {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("assign");
    printer.inc_indent();
    self.dst.print_ast(printer);
    self.src.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for VarAssign {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("assign");
    printer.inc_indent();
    printer.print("var");
    printer.println(self.name);
    self.src.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Skip {
  fn print_ast(&self, _printer: &mut IndentPrinter) {}
}

impl ASTData for Expr {
  fn print_ast(&self, printer: &mut IndentPrinter) {
    use ast::Expr::*;
    match &self {
      Identifier(identifier) => identifier.print_ast(printer),
      Indexed(indexed) => indexed.print_ast(printer),
      Const(const_) => const_.print_ast(printer),
      Call(call) => call.print_ast(printer),
      Unary(unary) => unary.print_ast(printer),
      Binary(binary) => binary.print_ast(printer),
      This(this) => this.print_ast(printer),
      ReadInt(read_int) => read_int.print_ast(printer),
      ReadLine(read_line) => read_line.print_ast(printer),
      NewClass(new_class) => new_class.print_ast(printer),
      NewArray(new_array) => new_array.print_ast(printer),
      TypeTest(type_test) => type_test.print_ast(printer),
      TypeCast(type_cast) => type_cast.print_ast(printer),
      Range(range) => range.print_ast(printer),
      Default(default) => default.print_ast(printer),
      Comprehension(comprehension) => comprehension.print_ast(printer),
    };
  }
}

impl ASTData for Identifier {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("varref");
    printer.println(self.name);
    if let Some(owner) = &self.owner {
      printer.inc_indent();
      owner.print_ast(printer);
      printer.dec_indent();
    }
  }
}

impl ASTData for Indexed {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("arrref");
    printer.inc_indent();
    self.array.print_ast(printer);
    self.index.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Const {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    use self::Const::*;
    match &self {
      IntConst(int_const) => int_const.print_ast(printer),
      BoolConst(bool_const) => bool_const.print_ast(printer),
      StringConst(string_const) => string_const.print_ast(printer),
      ArrayConst(array_const) => array_const.print_ast(printer),
      Null(null) => null.print_ast(printer),
    }
  }
}

impl ASTData for IntConst {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("intconst");
    printer.println(&self.value.to_string());
  }
}

impl ASTData for BoolConst {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("boolconst");
    printer.println(&self.value.to_string());
  }
}

impl ASTData for StringConst {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("stringconst");
    printer.println(&quote(&self.value));
  }
}

impl ASTData for ArrayConst {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("array const");
    printer.inc_indent();
    if self.value.is_empty() {
      printer.println("<empty>");
    } else {
      for const_ in &self.value { const_.print_ast(printer); }
    }
    printer.dec_indent();
  }
}

impl ASTData for Null {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("null");
  }
}

impl ASTData for Call {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("call");
    printer.println(self.name);
    printer.inc_indent();
    match &self.owner {
      Some(receiver) => receiver.print_ast(printer),
      None => printer.println("<empty>"),
    };
    for expr in &self.args { expr.print_ast(printer); }
    printer.dec_indent();
  }
}

impl ASTData for Unary {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    use ast::Operator::*;
    let opname = match self.opt {
      Neg => "neg",
      Not => "not",
      _ => unreachable!(),
    };
    printer.println(opname);
    printer.inc_indent();
    self.opr.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Binary {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    use self::Operator::*;
    let opname = match self.opt {
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
      Concat => "array concat",
      _ => unreachable!(),
    };
    printer.println(opname);
    printer.inc_indent();
    self.left.print_ast(printer);
    self.right.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for This {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("this");
  }
}

impl ASTData for ReadInt {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("readint");
  }
}

impl ASTData for ReadLine {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("readline");
  }
}

impl ASTData for NewClass {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("newobj");
    printer.println(self.name);
  }
}

impl ASTData for NewArray {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.print("newarray");
    self.elem_type.print_ast(printer);
    printer.newline();
    printer.inc_indent();
    self.len.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for TypeTest {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("instanceof");
    printer.inc_indent();
    self.expr.print_ast(printer);
    printer.println(self.name);
    printer.dec_indent();
  }
}

impl ASTData for TypeCast {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("classcast");
    printer.inc_indent();
    printer.println(self.name);
    self.expr.print_ast(printer);
    printer.dec_indent();
  }
}

impl ASTData for Range {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("arrref");
    printer.inc_indent();
    self.array.print_ast(printer);
    printer.println("range");
    printer.inc_indent();
    self.lower.print_ast(printer);
    self.upper.print_ast(printer);
    printer.dec_indent();
    printer.dec_indent();
  }
}

impl ASTData for Default {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("arrref");
    printer.inc_indent();
    self.array.print_ast(printer);
    self.index.print_ast(printer);
    printer.println("default");
    printer.inc_indent();
    self.default.print_ast(printer);
    printer.dec_indent();
    printer.dec_indent();
  }
}

impl ASTData for Comprehension {
   fn print_ast(&self, printer: &mut IndentPrinter) {
    printer.println("array comp");
    printer.inc_indent();
    printer.print("varbind");
    printer.println(self.name);
    self.array.print_ast(printer);
    match &self.cond {
      Some(cond) => cond.print_ast(printer),
      None => printer.println("boolconst true"),
    };
    self.expr.print_ast(printer);
    printer.dec_indent();
  }
}