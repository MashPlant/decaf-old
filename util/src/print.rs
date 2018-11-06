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
    pub  fn print(&mut self, s: &str) {
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
