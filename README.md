# Build

Please use my customized version of the 'syntax' tool. I have made many change to that tool which are not so general for a pull request

Actually src/parser.rs already exists, so you may skip the next steps.

```shell
git clone -b decaf https://github.com/MashPlant/syntax.git
cd syntax  
npm install
npm run build
./bin/syntax -g [decaf dir]/parser.rs.g -m lalr1 -o [decaf dir]/src/parser.rs --validate
```

Finally build the project.
```shell
cd [decaf dir]
cargo build --release
```

After that you can run tests.
```shell
cd testcases/[s1 or s2]
python runAll.py
```

I have made some changes to the grammar:

   1. Not treat 'var id' as 'lvalue', so that all test cases in s2 starting with 'q4' will get syntax error, and I have removed them all
    
   2. Make the associate statements of if/for/foreach/while/guard blocks, so that they will have independent namespaces, some test cases in s1 may fail because of it
   
   3. (not done yet) support initialization of local variable when declaring it(like 'int x = 0;')