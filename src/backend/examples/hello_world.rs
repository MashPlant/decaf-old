extern crate backend;

use backend::jvm::class::*;
use backend::jvm::builder::*;
use backend::jvm::types::*;
use backend::jvm::writer::*;

fn main() {
  let mut class = ClassBuilder::new(ACC_PUBLIC, "HelloWorld", "java/lang/Object");
  {
    // create main method
    let mut method = MethodBuilder::new(&mut class, ACC_PUBLIC | ACC_STATIC, "main", &[JavaType::Array(Box::new(JavaType::Class("java/lang/String")))], &JavaType::Void);

    // push PrintStream object and string to print onto the stack, and then call println function
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.load_constant("Hello, World!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    // add return statement
    method.do_return();

    method.done();
  }

  let class = class.done();
  class.write_to_file("HelloWorld.class");
}
