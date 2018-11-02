use std::io;

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

    fn inc_indent(&mut self) {
        self.indent += "    ";
    }

    fn dec_indent(&mut self) {
        for _ in 0..4 {
            self.indent.pop();
        }
    }

    // automatic add a space
    fn print(&mut self, s: &str) {
        if self.newline { self.content += &self.indent; }
        self.content += s;
        self.content += " ";
        self.newline = false;
    }

    fn println(&mut self, s: &str) {
        if self.newline { self.content += &self.indent; }
        self.content += s;
        self.content += "\n";
        self.newline = true;
    }

    fn newline(&mut self) {
        self.content += "\n";
        self.newline = true;
    }

    pub fn flush<W: io::Write>(&mut self, writer: &mut W) {
        writer.write(self.content.as_bytes()).unwrap();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Location(pub i32, pub i32);

pub const NO_LOCATION: Location = Location(-1, -1);

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
        printer.print("class");
        printer.print(&self.name);
        if self.sealed { printer.print("sealed"); }
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
        };
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
    Array(Option<Box<Type>>),
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
                if let Some(name) = name { name.print_to(printer); }
            }
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    If(If),
}

impl Statement {
    pub fn print_to(&self, _printer: &mut IndentPrinter) {}
}

#[derive(Debug)]
pub struct Block {
    pub loc: Location,
    pub statements: Vec<Statement>,
}

impl Block {
    pub fn print_to(&self, printer: &mut IndentPrinter) {
        for statement in &self.statements {
            statement.print_to(printer);
        }
    }
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