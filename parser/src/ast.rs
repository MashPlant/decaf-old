use std::io;

pub struct IndentPrinter {
    indent: String,
    content: String,
}

impl IndentPrinter {
    pub fn new() -> IndentPrinter {
        IndentPrinter {
            indent: String::new(),
            content: String::new(),
        }
    }

    fn inc_indent(&mut self) {
        self.indent += "  ";
    }

    fn dec_indent(&mut self) {
        self.indent.pop();
        self.indent.pop();
    }

    fn println(&mut self, s: &str) {
        self.content += &self.indent;
        self.content += s;
        self.content += "\n";
    }

    pub fn flush<W: io::Write>(&mut self, writer: &mut W) {
        writer.write(self.content.as_bytes()).unwrap();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Location(pub i32, pub i32);

pub const NO_LOCATION: Location = Location(-1, -1);

#[derive(Debug)]
pub struct Tree {
    pub loc: Location,
    pub data: TreeData,
}

#[derive(Debug)]
pub enum TreeData {
    Program(Program),
    ClassDef(ClassDef),
    Identifier(String),
    VarDef(VarDef),
    Type(Type),
    None,
}

impl Tree {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        match &self.data {
            TreeData::Program(program) => program.print_to(printer),
            _ => {}
        };
    }

    pub fn accept<V: Visitor>(&self, visitor: &mut V) {
        match &self.data {
            TreeData::Program(program) => visitor.visit_program(program),
            _ => {}
        };
    }
}

#[derive(Debug)]
pub struct Program {
    pub classes: Vec<ClassDef>,
}

impl Program {
    fn print_to(&self, printer: &mut IndentPrinter) {
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
    pub fields: Vec<Tree>,
    pub sealed: bool,
}

impl ClassDef {
    fn accept<V: Visitor>(&self, visitor: &mut V) { visitor.visit_class_def(self); }

    fn print_to(&self, printer: &mut IndentPrinter) {
        let mut description = "class ".to_string() + &self.name + " ";
        if self.sealed { description += "sealed"; }
        match &self.parent {
            Some(name) => description += name,
            None => description += "<empty>",
        };
        printer.println(&description);
        printer.inc_indent();
        for field in &self.fields { field.print_to(printer); }
        printer.dec_indent();
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

#[derive(Debug)]
pub struct VarDef {
    pub loc: Location,
    pub name: String,
    pub type_: Type,
}

#[derive(Debug)]
pub enum Type {
    Var,
    // int, string, bool, void
    Basic(&'static str),
    // user defined class
    Class(String),
    // type [][]...
    Array(Option<Box<Type>>),
}

#[derive(Debug)]
pub struct Block {
    pub loc: Location,
    pub statements: Vec<Tree>,
}

#[derive(Debug)]
pub struct If {
    pub loc: Location,
    pub true_branch: Option<Block>,
    pub false_branch: Option<Block>,
}

#[derive(Debug)]
pub struct Expr {
    pub loc: Location,
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

#[derive(Debug)]
pub struct Sem {
    pub loc: Location,
    pub value: SemValue,
}

#[derive(Debug)]
pub enum SemValue {
    IntLiteral(i32),
    BoolLiteral(bool),
    StringLiteral(String),
    Identifier(String),
    ClassList(Vec<ClassDef>),
    FieldList(Vec<Tree>),
    VarDefList(Vec<VarDef>),
    StatementList(Vec<Tree>),
    Program(Program),
    Statement(Tree),
    Expr(Expr),
    VarDef(VarDef),
    Type(Type),
    Sealed(bool),

}