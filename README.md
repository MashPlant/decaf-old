## Build

### 1. Parser

Actually src/parser.rs already exists, so you may skip this step.

If you want to regenerate src/parser.rs, please use my customized version of the 'syntax' tool. I have made many change to that tool which are not so suitable for a pull request.

```shell
git clone -b decaf https://github.com/MashPlant/syntax.git
cd syntax  
npm install
npm run build
./bin/syntax -g [decaf dir]/parser.rs.g -m lalr1 -o [decaf dir]/src/parser.rs --validate
```


### 2. LLVM

You need to build LLVM in order to build this project. You can follow the steps in this site: 
[llvm-sys](https://github.com/tari/llvm-sys.rs) 

I use LLVM version 6.0, you can modify it in src/llvm/Cargo.toml. (I copy the source code of [llvm-sys](https://github.com/tari/llvm-sys.rs), because when I tried to use cargo dependency, some error with the linker occurred).

### 3. build
Finally build the project.
```shell
cd [decaf dir]
cargo build --release
```

After that you can run tests.
```shell
cd testcases/[s1 or s2 or s3]
python runAll.py
```

Or you can compile the RadixSort.decaf in the testcases folder and run it, on my computer it takes about 2s to generate and sort 100000000 ints and check result correctness(when using llvm or jvm codegen).

## Changes 

I have made some changes to the grammar:
```
1. Not treat 'var id' as 'lvalue', so that all test cases in s2 starting with 'q4' will get syntax error, and I have removed them all
 
2. Make the associate statements of if/for/foreach/while/guard blocks, so that they will have independent namespaces naturally, some test cases in s1 may fail because of it

3. Support initialization of local variable when declaring it(like 'int x = 0;')

4. Support: ++ -- & ^ | << >> (only for llvm & jvm codegen)
```