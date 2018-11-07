extern crate util;
extern crate common;

use self::util::{IndentPrinter, quote};
use self::common::Location;

#[derive(Debug)]
pub struct Program {
    pub classes: Vec<ClassDef>,
}

impl Program {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("program");
        printer.inc_indent();
        for class in &self.classes { class.print_to(printer); }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct ClassDef {
    pub loc: Location,
    pub name: String,
    pub parent: Option<String>,
    pub fields: Vec<FieldDef>,
    pub sealed: bool,
}

impl ClassDef {
    pub fn accept<V: Visitor>(&self, visitor: &mut V) { visitor.visit_class_def(self); }

    pub fn print_to(&self, printer: &mut IndentPrinter) {
        if self.sealed { printer.print("sealed"); }
        printer.print("class");
        printer.print(&self.name);
        match &self.parent {
            Some(name) => printer.print(&name),
            None => printer.print("<empty>"),
        };
        printer.newline();
        printer.inc_indent();
        for field in &self.fields { field.print_to(printer); }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub enum FieldDef {
    MethodDef(MethodDef),
    VarDef(VarDef),
}

impl FieldDef {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match &self {
            FieldDef::MethodDef(method_def) => method_def.print_to(printer),
            FieldDef::VarDef(var_def) => var_def.print_to(printer),
        }
    }
}

#[derive(Debug)]
pub struct MethodDef {
    pub loc: Location,
    pub name: String,
    pub return_type: Type,
    pub parameters: Vec<VarDef>,
    pub static_: bool,
    pub body: Block,
}

impl MethodDef {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        if self.static_ { printer.print("static"); }
        printer.print("func");
        printer.print(&self.name);
        self.return_type.print_to(printer);
        printer.newline();
        printer.inc_indent();
        printer.println("formals");
        printer.inc_indent();
        for parameter in &self.parameters {
            parameter.print_to(printer);
        }
        printer.dec_indent();
        self.body.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct VarDef {
    pub loc: Location,
    pub name: String,
    pub type_: Type,
}

impl VarDef {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("vardef");
        printer.print(&self.name);
        self.type_.print_to(printer);
        printer.newline();
    }
}

#[derive(Debug)]
pub enum Type {
    Var,
    // int, string, bool, void
    Basic(&'static str),
    // user defined class
    Class(String),
    // type [][]...
    Array(Box<Type>),
}

impl Type {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match &self {
            Type::Var => printer.print("var"),
            Type::Basic(name) => printer.print(&(name.to_string() + "type")),
            Type::Class(name) => {
                printer.print("classtype");
                printer.print(name);
            }
            Type::Array(name) => {
                printer.print("arrtype");
                name.print_to(printer);
            }
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    VarDef(VarDef),
    Simple(Simple),
    If(If),
    While(While),
    For(For),
    Return(Return),
    Print(Print),
    Break(Break),
    ObjectCopy(ObjectCopy),
    Foreach(Foreach),
    Guarded(Guarded),
    Block(Block),
}

impl Statement {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        use Statement::*;
        match &self {
            VarDef(var_def) => var_def.print_to(printer),
            Simple(simple) => simple.print_to(printer),
            If(if_) => if_.print_to(printer),
            While(while_) => while_.print_to(printer),
            For(for_) => for_.print_to(printer),
            Return(return_) => return_.print_to(printer),
            Print(print) => print.print_to(printer),
            Break(break_) => break_.print_to(printer),
            ObjectCopy(object_copy) => object_copy.print_to(printer),
            Foreach(foreach) => foreach.print_to(printer),
            Guarded(guarded) => guarded.print_to(printer),
            Block(block) => block.print_to(printer),
        };
    }
}

#[derive(Debug)]
pub enum Simple {
    Assign(Assign),
    VarAssign(VarAssign),
    Expr(Expr),
    Skip(Skip),
}

impl Simple {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match &self {
            Simple::Assign(assign) => assign.print_to(printer),
            Simple::VarAssign(var_assign) => var_assign.print_to(printer),
            Simple::Expr(expr) => expr.print_to(printer),
            Simple::Skip(skip) => skip.print_to(printer),
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub loc: Location,
    pub statements: Vec<Statement>,
}

impl Block {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("stmtblock");
        printer.inc_indent();
        for statement in &self.statements {
            statement.print_to(printer);
        }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct If {
    pub loc: Location,
    pub cond: Expr,
    pub on_true: Box<Statement>,
    pub on_false: Option<Box<Statement>>,
}

impl If {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("if");
        printer.inc_indent();
        self.cond.print_to(printer);
        self.on_true.print_to(printer);
        printer.dec_indent();
        if let Some(on_false) = &self.on_false {
            printer.println("else");
            printer.inc_indent();
            on_false.print_to(printer);
            printer.dec_indent();
        }
    }
}

#[derive(Debug)]
pub struct While {
    pub loc: Location,
    pub cond: Expr,
    pub body: Box<Statement>,
}

impl While {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("while");
        printer.inc_indent();
        self.cond.print_to(printer);
        self.body.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct For {
    pub loc: Location,
    // Skip for no init or update
    pub init: Simple,
    pub cond: Expr,
    pub update: Simple,
    pub body: Box<Statement>,
}

impl For {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("for");
        printer.inc_indent();
        self.init.print_to(printer);
        self.cond.print_to(printer);
        self.update.print_to(printer);
        self.body.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Return {
    pub loc: Location,
    pub expr: Option<Expr>,
}

impl Return {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("return");
        if let Some(expr) = &self.expr {
            printer.inc_indent();
            expr.print_to(printer);
            printer.dec_indent();
        }
    }
}

#[derive(Debug)]
pub struct Print {
    pub loc: Location,
    pub print: Vec<Expr>,
}

impl Print {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("print");
        printer.inc_indent();
        for expr in &self.print { expr.print_to(printer) }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Break {
    pub loc: Location,
}

impl Break {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("break");
    }
}

#[derive(Debug)]
pub struct ObjectCopy {
    pub loc: Location,
    pub dst: String,
    pub src: Expr,
}

impl ObjectCopy {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("scopy");
        printer.inc_indent();
        printer.println(&self.dst);
        self.src.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Foreach {
    pub loc: Location,
    pub type_: Type,
    pub name: String,
    pub array: Expr,
    pub cond: Option<Expr>,
    pub body: Box<Statement>,
}

impl Foreach {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("foreach");
        printer.inc_indent();
        printer.print("varbind");
        printer.print(&self.name);
        self.type_.print_to(printer);
        printer.newline();
        self.array.print_to(printer);
        match &self.cond {
            Some(cond) => cond.print_to(printer),
            None => printer.println("boolconst true"),
        }
        self.body.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Guarded {
    pub loc: Location,
    pub guarded: Vec<(Expr, Statement)>,
}

impl Guarded {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("guarded");
        printer.inc_indent();
        if self.guarded.is_empty() {
            printer.println("<empty>");
        } else {
            for (e, s) in &self.guarded {
                printer.println("guard");
                printer.inc_indent();
                e.print_to(printer);
                s.print_to(printer);
                printer.dec_indent();
            }
        }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Assign {
    pub loc: Location,
    pub dst: LValue,
    pub src: Expr,
}

impl Assign {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("assign");
        printer.inc_indent();
        self.dst.print_to(printer);
        self.src.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct VarAssign {
    pub loc: Location,
    pub name: String,
    pub src: Expr,
}

impl VarAssign {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("assign");
        printer.inc_indent();
        printer.print("var");
        printer.println(&self.name);
        self.src.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Skip {
    pub loc: Location,
}

impl Skip {
    pub fn print_to(&self, _printer: &mut IndentPrinter) {
        // no op
    }
}

#[derive(Debug)]
pub enum Operator {
    Neg,
    Not,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Repeat,
    Concat,
}

#[derive(Debug)]
pub enum Expr {
    LValue(LValue),
    Const(Const),
    Call(Call),
    Unary(Unary),
    Binary(Binary),
    This(This),
    ReadInt(ReadInt),
    ReadLine(ReadLine),
    NewClass(NewClass),
    NewArray(NewArray),
    TypeTest(TypeTest),
    TypeCast(TypeCast),
    Range(Range),
    Default(Default),
    Comprehension(Comprehension),
}

impl Expr {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        use Expr::*;
        match &self {
            LValue(lvalue) => lvalue.print_to(printer),
            Const(const_) => const_.print_to(printer),
            Call(call) => call.print_to(printer),
            Unary(unary) => unary.print_to(printer),
            Binary(binary) => binary.print_to(printer),
            This(this) => this.print_to(printer),
            ReadInt(read_int) => read_int.print_to(printer),
            ReadLine(read_line) => read_line.print_to(printer),
            NewClass(new_class) => new_class.print_to(printer),
            NewArray(new_array) => new_array.print_to(printer),
            TypeTest(type_test) => type_test.print_to(printer),
            TypeCast(type_cast) => type_cast.print_to(printer),
            Range(range) => range.print_to(printer),
            Default(default) => default.print_to(printer),
            Comprehension(comprehension) => comprehension.print_to(printer),
        };
    }

    pub fn get_loc(&self) -> Location {
        use Expr::*;
        match &self {
            LValue(lvalue) => lvalue.get_loc(),
            Const(const_) => const_.get_loc(),
            Call(call) => call.loc,
            Unary(unary) => unary.loc,
            Binary(binary) => binary.loc,
            This(this) => this.loc,
            ReadInt(read_int) => read_int.loc,
            ReadLine(read_line) => read_line.loc,
            NewClass(new_class) => new_class.loc,
            NewArray(new_array) => new_array.loc,
            TypeTest(type_test) => type_test.loc,
            TypeCast(type_cast) => type_cast.loc,
            Range(range) => range.loc,
            Default(default) => default.loc,
            Comprehension(comprehension) => comprehension.loc,
        }
    }
}

#[derive(Debug)]
pub enum LValue {
    Indexed(Indexed),
    Identifier(Identifier),
}

impl LValue {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match &self {
            LValue::Indexed(indexed) => indexed.print_to(printer),
            LValue::Identifier(identifier) => identifier.print_to(printer),
        }
    }

    pub fn get_loc(&self) -> Location {
        match &self {
            LValue::Indexed(indexed) => indexed.loc,
            LValue::Identifier(identifier) => identifier.loc,
        }
    }
}

#[derive(Debug)]
pub struct Indexed {
    pub loc: Location,
    pub array: Box<Expr>,
    pub index: Box<Expr>,
}

impl Indexed {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("arrref");
        printer.inc_indent();
        self.array.print_to(printer);
        self.index.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Identifier {
    pub loc: Location,
    pub owner: Option<Box<Expr>>,
    pub name: String,
}

impl Identifier {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("varref");
        printer.println(&self.name);
        if let Some(owner) = &self.owner {
            printer.inc_indent();
            owner.print_to(printer);
            printer.dec_indent();
        }
    }
}

#[derive(Debug)]
pub enum Const {
    IntConst(IntConst),
    BoolConst(BoolConst),
    StringConst(StringConst),
    ArrayConst(ArrayConst),
    Null(Null),
}

impl Const {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        use Const::*;
        match &self {
            IntConst(int_const) => int_const.print_to(printer),
            BoolConst(bool_const) => bool_const.print_to(printer),
            StringConst(string_const) => string_const.print_to(printer),
            ArrayConst(array_const) => array_const.print_to(printer),
            Null(null) => null.print_to(printer),
        }
    }

    pub fn get_loc(&self) -> Location {
        use Const::*;
        match &self {
            IntConst(int_const) => int_const.loc,
            BoolConst(bool_const) => bool_const.loc,
            StringConst(string_const) => string_const.loc,
            ArrayConst(array_const) => array_const.loc,
            Null(null) => null.loc,
        }
    }
}

#[derive(Debug)]
pub struct IntConst {
    pub loc: Location,
    pub value: i32,
}

impl IntConst {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("intconst");
        printer.println(&self.value.to_string());
    }
}

#[derive(Debug)]
pub struct BoolConst {
    pub loc: Location,
    pub value: bool,
}

impl BoolConst {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("boolconst");
        printer.println(&self.value.to_string());
    }
}

#[derive(Debug)]
pub struct StringConst {
    pub loc: Location,
    pub value: String,
}

impl StringConst {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("stringconst");
        printer.println(&quote(&self.value));
    }
}

#[derive(Debug)]
pub struct ArrayConst {
    pub loc: Location,
    pub value: Vec<Const>,
}

impl ArrayConst {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("array const");
        printer.inc_indent();
        if self.value.is_empty() {
            printer.println("<empty>");
        } else {
            for const_ in &self.value { const_.print_to(printer); }
        }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Null {
    pub loc: Location,
}

impl Null {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("null");
    }
}

#[derive(Debug)]
pub struct Call {
    pub loc: Location,
    pub receiver: Option<Box<Expr>>,
    pub name: String,
    pub arguments: Vec<Expr>,
}

impl Call {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("call");
        printer.println(&self.name);
        printer.inc_indent();
        match &self.receiver {
            Some(receiver) => receiver.print_to(printer),
            None => printer.println("<empty>"),
        };
        for expr in &self.arguments { expr.print_to(printer); }
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Unary {
    pub loc: Location,
    pub opt: Operator,
    pub opr: Box<Expr>,
}

impl Unary {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        use Operator::*;
        let opname = match self.opt {
            Neg => "neg",
            Not => "not",
            _ => unreachable!(),
        };
        printer.println(opname);
        printer.inc_indent();
        self.opr.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Binary {
    pub loc: Location,
    pub opt: Operator,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Binary {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        use Operator::*;
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
        self.left.print_to(printer);
        self.right.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct This {
    pub loc: Location,
}

impl This {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("this");
    }
}

#[derive(Debug)]
pub struct ReadInt {
    pub loc: Location,
}

impl ReadInt {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("readint");
    }
}

#[derive(Debug)]
pub struct ReadLine {
    pub loc: Location,
}

impl ReadLine {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("readline");
    }
}

#[derive(Debug)]
pub struct NewClass {
    pub loc: Location,
    pub name: String,
}

impl NewClass {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("newobj");
        printer.println(&self.name);
    }
}

#[derive(Debug)]
pub struct NewArray {
    pub loc: Location,
    pub type_: Type,
    pub len: Box<Expr>,
}

impl NewArray {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("newarray");
        self.type_.print_to(printer);
        printer.newline();
        printer.inc_indent();
        self.len.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct TypeTest {
    pub loc: Location,
    pub expr: Box<Expr>,
    pub name: String,
}

impl TypeTest {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("instanceof");
        printer.inc_indent();
        self.expr.print_to(printer);
        printer.println(&self.name);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct TypeCast {
    pub loc: Location,
    pub name: String,
    pub expr: Box<Expr>,
}

impl TypeCast {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("classcast");
        printer.inc_indent();
        printer.println(&self.name);
        self.expr.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Range {
    pub loc: Location,
    pub array: Box<Expr>,
    pub lower: Box<Expr>,
    pub upper: Box<Expr>,
}

impl Range {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("arrref");
        printer.inc_indent();
        self.array.print_to(printer);
        printer.println("range");
        printer.inc_indent();
        self.lower.print_to(printer);
        self.upper.print_to(printer);
        printer.dec_indent();
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Default {
    pub loc: Location,
    pub array: Box<Expr>,
    pub index: Box<Expr>,
    pub default: Box<Expr>,
}

impl Default {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("arrref");
        printer.inc_indent();
        self.array.print_to(printer);
        self.index.print_to(printer);
        printer.println("default");
        printer.inc_indent();
        self.default.print_to(printer);
        printer.dec_indent();
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Comprehension {
    pub loc: Location,
    pub expr: Box<Expr>,
    pub name: String,
    pub array: Box<Expr>,
    pub cond: Option<Box<Expr>>,
}

impl Comprehension {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("array comp");
        printer.inc_indent();
        printer.print("varbind");
        printer.println(&self.name);
        self.array.print_to(printer);
        match &self.cond {
            Some(cond) => cond.print_to(printer),
            None => printer.println("boolconst true"),
        };
        self.expr.print_to(printer);
        printer.dec_indent();
    }
}

pub trait Visitor {
    fn visit_program(&mut self, program: &Program);

    fn visit_class_def(&mut self, class_def: &ClassDef);

    fn visit_method_def(&mut self);

    fn visit_var_def(&mut self);

    fn visit_skip(&mut self);

    fn visit_block(&mut self);

    fn visit_while_loop(&mut self);

    fn visit_for_loop(&mut self);

    fn visit_if(&mut self);

    fn visit_exec(&mut self);

    fn visit_break(&mut self);

    fn visit_return(&mut self);

    fn visit_apply(&mut self);

    fn visit_new_class(&mut self);

    fn visit_new_array(&mut self);

    fn visit_assign(&mut self);

    fn visit_unary(&mut self);

    fn visit_binary(&mut self);

    fn visit_call_expr(&mut self);

    fn visit_read_int_expr(&mut self);

    fn visit_read_line_expr(&mut self);

    fn visit_read_print(&mut self);

    fn visit_this_expr(&mut self);

    fn visit_lvalue(&mut self);

    fn visit_type_cast(&mut self);

    fn visit_type_test(&mut self);

    fn visit_indexed(&mut self);

    fn visit_identifier(&mut self);

    fn visit_literal(&mut self);

    fn visit_null(&mut self);

    fn visit_type_identifier(&mut self);

    fn visit_type_class(&mut self);

    fn visit_type_array(&mut self);
}