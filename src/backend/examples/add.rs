extern crate backend;

use backend::jvm::class::*;
use backend::jvm::builder::*;
use backend::jvm::types::*;
use backend::jvm::writer::*;

fn main() {
    let mut class = ClassBuilder::new(ACC_PUBLIC, "Add", "java/lang/Object");

    {
        // create main method
        let mut method = MethodBuilder::new(&mut class,ACC_PUBLIC | ACC_STATIC, "main", &[JavaType::Array(Box::new(JavaType::Class("java/lang/String")))], &JavaType::Void);

        // push PrintStream object onto the stack for later use
        method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));

        // execute 11 + 37 + 42
        method.int_const(11);
        method.int_const(37);
        method.i_add();
        method.int_const(42);
        method.i_add();

        // print the result
        method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Int], &JavaType::Void);

        // add return statement
        method.return_();

        // finish!
        method.done();
    }

    class.done().write_to_file("Add.class");
}
