extern crate backend;

use backend::jvm::class::*;
use backend::jvm::builder::*;
use backend::jvm::types::*;
use backend::jvm::writer::*;

fn main() {
  let mut class = ClassBuilder::new(ACC_PUBLIC, "If2", "java/lang/Object");

  {
    // create main method
    let mut method = MethodBuilder::new(&mut class, ACC_PUBLIC | ACC_STATIC, "main", &[JavaType::Array(Box::new(JavaType::Class("java/lang/String")))], &JavaType::Void);

    // if (args.length > 0) {
    //     System.out.println("Hello with args!");
    //     if (args[0].length() >= 5) {
    //         System.out.println("First arg had at least 5 characters");
    //     } else {
    //         System.out.println("First arg has less than 5 characters");
    //     }
    // } else {
    //     System.out.println("Hello without args!");
    // }
    method.a_load_0();
    method.array_length();
    method.if_le(2);

    // outer if: true case
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.load_constant("Hello with args!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    // inner if: load the first arg and calculate string length
    method.a_load_0();
    method.i_const_0();
    method.a_a_load();
    method.invoke_virtual("java/lang/String", "length", &[], &JavaType::Int);

    // inner if: do comparison against 5
    method.i_const_5();
    method.if_i_cmp_lt(0);

    // inner if: true case
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.load_constant("First arg has at least 5 characters");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);
    method.goto(1);

    // inner if: false case
    method.label(0);
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.load_constant("First arg has less than 5 characters");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    // outer if: done true case
    method.goto(1);

    // outer if: false case
    method.label(2);
    method.get_static("java/lang/System", "out", &JavaType::Class("java/io/PrintStream"));
    method.load_constant("Hello without args!");
    method.invoke_virtual("java/io/PrintStream", "println", &[JavaType::Class("java/lang/String")], &JavaType::Void);

    // after outer if
    method.label(1);
    method.do_return();

    // finish!
    method.done();
  }
  
  class.done().write_to_file("If2.class");
}
