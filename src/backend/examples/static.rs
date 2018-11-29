extern crate backend;

use backend::jvm::class::*;
use backend::jvm::builder::*;
use backend::jvm::types::*;
use backend::jvm::writer::*;

fn main() {
  let mut class = ClassBuilder::new(ACC_PUBLIC, "Static", "java/lang/Object");

  {
    let mut method = MethodBuilder::new(&mut class, ACC_PUBLIC | ACC_STATIC, "main", &[JavaType::Array(Box::new(JavaType::Class("java/lang/String")))], &JavaType::Void);
    method.invoke_static("Static", "hello_world", &[], &JavaType::Void);
    method.string_const("Rust");
    method.invoke_static("Static", "hello_someone", &[JavaType::Class("java/lang/String")], &JavaType::Void);
    method.return_();
    method.done();
  }

  {
    let mut method = MethodBuilder::new(&mut class,ACC_STATIC, "hello_world", &[], &JavaType::Void);
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.string_const("Hello, World!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);
    method.return_();
    method.done();
  }

  {
    let mut method = MethodBuilder::new(&mut class,ACC_STATIC, "hello_someone", &[JavaType::Class("java/lang/String")], &JavaType::Void);
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.string_const("Hello, ");
    method.invoke_virtual("java/io/PrintStream", "print", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.a_load(0);
    method.invoke_virtual("java/io/PrintStream", "print", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.string_const("!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    method.return_();
    method.done();
  }

  class.done().write_to_file("Static.class");
}
