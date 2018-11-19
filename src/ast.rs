use super::util::*;
use super::loc::*;
use std::collections::HashMap;
use std::default::Default as D;
use std::ptr;

#[derive(Debug)]
pub struct Program {
    pub classes: Vec<ClassDef>,
    pub symbols: HashMap<&'static str, *mut ClassDef>,
    pub main: *const ClassDef,
}

impl D for Program {
    fn default() -> Self {
        Program {
            classes: D::default(),
            symbols: D::default(),
            main: ptr::null(),
        }
    }
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
    // syntax part
    pub loc: Loc,
    pub name: &'static str,
    pub parent: Option<&'static str>,
    pub fields: Vec<FieldDef>,
    pub sealed: bool,
    // semantic part
    pub order: i32,
    pub parent_ref: *mut ClassDef,
    pub symbols: HashMap<&'static str, *mut FieldDef>,
}

impl D for ClassDef {
    fn default() -> Self {
        ClassDef {
            loc: D::default(),
            name: D::default(),
            parent: D::default(),
            fields: D::default(),
            sealed: D::default(),
            order: D::default(),
            parent_ref: ptr::null_mut(),
            symbols: D::default(),
        }
    }
}

impl ClassDef {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        if self.sealed { printer.print("sealed"); }
        printer.print("class");
        printer.print(self.name);
        match self.parent {
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
    pub fn get_loc(&self) -> Loc {
        match self {
            FieldDef::MethodDef(method_def) => method_def.loc,
            FieldDef::VarDef(var_def) => var_def.loc,
        }
    }

    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match &self {
            FieldDef::MethodDef(method_def) => method_def.print_to(printer),
            FieldDef::VarDef(var_def) => var_def.print_to(printer),
        }
    }
}

#[derive(Debug, Default)]
pub struct MethodDef {
    pub loc: Loc,
    pub name: &'static str,
    pub return_type: Type,
    pub parameters: Vec<VarDef>,
    pub static_: bool,
    pub body: Block,
    // symbols for parameters
    pub symbols: HashMap<&'static str, *mut VarDef>,
}

impl MethodDef {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        if self.static_ { printer.print("static"); }
        printer.print("func");
        printer.print(self.name);
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
    pub loc: Loc,
    pub name: &'static str,
    pub type_: Type,
}

impl VarDef {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("vardef");
        printer.print(self.name);
        self.type_.print_to(printer);
        printer.newline();
    }
}

#[derive(Debug)]
pub enum Type {
    Error,
    Var,
    // int, string, bool, void
    Basic(&'static str),
    // user defined class
    Class(&'static str),
    // type [][]...
    Array(Box<Type>),
}

impl D for Type {
    fn default() -> Self {
        Type::Error
    }
}

impl Type {
    pub fn is_error(&self) -> bool {
        match self {
            Type::Error => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        if let Type::Basic(name) = self {
            name == "void"
        }
        false
    }

    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match self {
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
            _ => unimplemented!()
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
        use self::Statement::*;
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

#[derive(Debug, Default)]
pub struct Block {
    // syntax part
    pub loc: Loc,
    pub statements: Vec<Statement>,
    // semantic part
    pub is_method: bool,
    pub symbols: HashMap<&'static str, *mut VarDef>,
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
}

impl Break {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("break");
    }
}

#[derive(Debug)]
pub struct ObjectCopy {
    pub loc: Loc,
    pub dst: &'static str,
    pub src: Expr,
}

impl ObjectCopy {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("scopy");
        printer.inc_indent();
        printer.println(self.dst);
        self.src.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Foreach {
    pub loc: Loc,
    pub type_: Type,
    pub name: &'static str,
    pub array: Expr,
    pub cond: Option<Expr>,
    pub body: Box<Statement>,
}

impl Foreach {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("foreach");
        printer.inc_indent();
        printer.print("varbind");
        printer.print(self.name);
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
    pub name: &'static str,
    pub src: Expr,
}

impl VarAssign {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("assign");
        printer.inc_indent();
        printer.print("var");
        printer.println(self.name);
        self.src.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Skip {
    pub loc: Loc,
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
        use self::Expr::*;
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

    pub fn get_loc(&self) -> Loc {
        use self::Expr::*;
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

    pub fn get_loc(&self) -> Loc {
        match &self {
            LValue::Indexed(indexed) => indexed.loc,
            LValue::Identifier(identifier) => identifier.loc,
        }
    }
}

#[derive(Debug)]
pub struct Indexed {
    pub loc: Loc,
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
    pub loc: Loc,
    pub owner: Option<Box<Expr>>,
    pub name: &'static str,
}

impl Identifier {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("varref");
        printer.println(self.name);
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
        use self::Const::*;
        match &self {
            IntConst(int_const) => int_const.print_to(printer),
            BoolConst(bool_const) => bool_const.print_to(printer),
            StringConst(string_const) => string_const.print_to(printer),
            ArrayConst(array_const) => array_const.print_to(printer),
            Null(null) => null.print_to(printer),
        }
    }

    pub fn get_loc(&self) -> Loc {
        use self::Const::*;
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
}

impl Null {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("null");
    }
}

#[derive(Debug)]
pub struct Call {
    pub loc: Loc,
    pub receiver: Option<Box<Expr>>,
    pub name: &'static str,
    pub arguments: Vec<Expr>,
}

impl Call {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("call");
        printer.println(self.name);
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
    pub loc: Loc,
    pub opt: Operator,
    pub opr: Box<Expr>,
}

impl Unary {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        use self::Operator::*;
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
    pub loc: Loc,
    pub opt: Operator,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

impl Binary {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
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
        self.left.print_to(printer);
        self.right.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct This {
    pub loc: Loc,
}

impl This {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("this");
    }
}

#[derive(Debug)]
pub struct ReadInt {
    pub loc: Loc,
}

impl ReadInt {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("readint");
    }
}

#[derive(Debug)]
pub struct ReadLine {
    pub loc: Loc,
}

impl ReadLine {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("readline");
    }
}

#[derive(Debug)]
pub struct NewClass {
    pub loc: Loc,
    pub name: &'static str,
}

impl NewClass {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.print("newobj");
        printer.println(self.name);
    }
}

#[derive(Debug)]
pub struct NewArray {
    pub loc: Loc,
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
    pub loc: Loc,
    pub expr: Box<Expr>,
    pub name: &'static str,
}

impl TypeTest {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("instanceof");
        printer.inc_indent();
        self.expr.print_to(printer);
        printer.println(self.name);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct TypeCast {
    pub loc: Loc,
    pub name: &'static str,
    pub expr: Box<Expr>,
}

impl TypeCast {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("classcast");
        printer.inc_indent();
        printer.println(self.name);
        self.expr.print_to(printer);
        printer.dec_indent();
    }
}

#[derive(Debug)]
pub struct Range {
    pub loc: Loc,
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
    pub loc: Loc,
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
    pub loc: Loc,
    pub expr: Box<Expr>,
    pub name: &'static str,
    pub array: Box<Expr>,
    pub cond: Option<Box<Expr>>,
}

impl Comprehension {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        printer.println("array comp");
        printer.inc_indent();
        printer.print("varbind");
        printer.println(self.name);
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
    fn visit_program(&mut self, _program: &mut Program) {}

    fn visit_class_def(&mut self, _class_def: &mut ClassDef) {}

    fn visit_field_def(&mut self, _field_def: &mut FieldDef) {}

    fn visit_statement(&mut self, statement: &mut Statement) {
        use self::Statement::*;
        match statement {
            VarDef(var_def) => self.visit_var_def(var_def),
            Simple(simple) => self.visit_simple(simple),
            If(if_) => self.visit_if(if_),
            While(while_) => self.visit_while(while_),
            For(for_) => self.visit_for(for_),
            Return(return_) => self.visit_return(return_),
            Print(print) => self.visit_print(print),
            Break(break_) => self.visit_break(break_),
            ObjectCopy(object_copy) => self.visit_object_copy(object_copy),
            Foreach(foreach) => self.visit_foreach(foreach),
            Guarded(guarded) => self.visit_guarded(guarded),
            Block(block) => self.visit_block(block),
        };
    }

    fn visit_simple(&mut self, _simple: &mut Simple) {}

    fn visit_var_def(&mut self, _var_def: &mut VarDef) {}

    fn visit_skip(&mut self, _skip: &mut Skip) {}

    fn visit_block(&mut self, _block: &mut Block) {}

    fn visit_while(&mut self, _while: &mut While) {}

    fn visit_for(&mut self, _for: &mut For) {}

    fn visit_if(&mut self, _if: &mut If) {}

    fn visit_break(&mut self, _break: &mut Break) {}

    fn visit_return(&mut self, _return: &mut Return) {}

    fn visit_object_copy(&mut self, _object_copy: &mut ObjectCopy) {}

    fn visit_foreach(&mut self, _foreach: &mut Foreach) {}

    fn visit_guarded(&mut self, _guarded: &mut Guarded) {}

    fn visit_new_class(&mut self, _new_class: &mut NewClass) {}

    fn visit_new_array(&mut self, _new_array: &mut NewArray) {}

    fn visit_assign(&mut self, _assign: &mut Assign) {}

    fn visit_expr(&mut self, expr: &mut Expr) {
        use self::Expr::*;
        match expr {
            LValue(lvalue) => self.visit_lvalue(lvalue),
            Const(const_) => self.visit_const(const_),
            Call(call) => self.visit_call(call),
            Unary(unary) => self.visit_unary(unary),
            Binary(binary) => self.visit_binary(binary),
            This(this) => self.visit_this(this),
            ReadInt(read_int) => self.visit_read_int(read_int),
            ReadLine(read_line) => self.visit_read_line(read_line),
            NewClass(new_class) => self.visit_new_class(new_class),
            NewArray(new_array) => self.visit_new_array(new_array),
            TypeTest(type_test) => self.visit_type_test(type_test),
            TypeCast(type_cast) => self.visit_type_cast(type_cast),
            Range(range) => self.visit_range(range),
            Default(default) => self.visit_default(default),
            Comprehension(comprehension) => self.visit_comprehension(comprehension),
        };
    }

    fn visit_lvalue(&mut self, _lvalue: &mut LValue) {}

    fn visit_const(&mut self, _const_: &mut Const) {}

    fn visit_unary(&mut self, _unary: &mut Unary) {}

    fn visit_binary(&mut self, _binary: &mut Binary) {}

    fn visit_call(&mut self, _call: &mut Call) {}

    fn visit_read_int(&mut self, _read_int: &mut ReadInt) {}

    fn visit_read_line(&mut self, _read_line: &mut ReadLine) {}

    fn visit_print(&mut self, _print: &mut Print) {}

    fn visit_this(&mut self, _this: &mut This) {}

    fn visit_type_cast(&mut self, _type_cast: &mut TypeCast) {}

    fn visit_type_test(&mut self, _type_test: &mut TypeTest) {}

    fn visit_indexed(&mut self, _indexed: &mut Indexed) {}

    fn visit_identifier(&mut self, _identifier: &mut Identifier) {}

    fn visit_range(&mut self, _range: &mut Range) {}

    fn visit_default(&mut self, _default: &mut Default) {}

    fn visit_comprehension(&mut self, _comprehension: &mut Comprehension) {}

    fn visit_type(&mut self, _type: &mut Type) {}
}