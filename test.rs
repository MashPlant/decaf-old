use std::collections::HashMap;

struct S<'a> {
    table: HashMap<i32, &'a C>,
}

struct P<'a> {
    s: S<'a>,
    cs: Vec<C>,
}

struct C {
    name: &'static str,
}

#[derive(Default)]
struct D {
    i: i32,
}

fn main() {
//    let mut p = P { s: S { table: HashMap::new() }, cs: Vec::new() };
//    p.cs.push(C { name: "hello" });
//    p.s.table.insert(1, &p.cs[0]);
//    p.s.table.insert(1, &p);
//    if let Some(s) = &mut p.s {
//        s.table.insert(1, &p);
//    }
//    p.s.unwrap().table.insert(1, &p);
}
