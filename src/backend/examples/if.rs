extern crate backend;

use backend::jvm::class::*;
use backend::jvm::builder::*;
use backend::jvm::types::*;
use backend::jvm::writer::*;

fn main() {
  let mut class = ClassBuilder::new(ACC_PUBLIC, "If", "java/lang/Object");

  {
    // create main method
    let mut method = MethodBuilder::new(&mut class, ACC_PUBLIC | ACC_STATIC, "main", &[JavaType::Array(Box::new(JavaType::Class("java/lang/String")))], &JavaType::Void);

    // if (args.length > 0) {
    //     System.out.println("Hello with args!");
    // } else {
    //     System.out.println("Hello without args!");
    // }
    method.a_load(0);
    method.array_length();
    method.if_le(0);

    // true case
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.string_const("Hello with args!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);
    method.goto(1);

    // false case
    method.label(0);
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.string_const("Hello without args!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    // after
    method.label(1);
    method.return_();

    // finish!
    method.done();
  }

  class.done().write_to_file("If.class");
}
