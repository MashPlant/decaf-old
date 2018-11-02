use std::mem;

fn main() {
    let mut vv = vec![vec![1, 2, 3], vec![4, 5, 6]];
    vv[0] = mem::replace(&mut vv[1], vec![7, 8, 9]);
    println!("{:?}", vv);
}