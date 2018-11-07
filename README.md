# Build

Please use my customized version of the 'syntax' tool. I have made many change to that tool which are not so general for a pull request.
```shell
git clone -b decaf https://github.com/MashPlant/syntax.git
cd syntax  
npm install
npm run build
```

And use the parser generator to generate parser/src/lib.rs(actually it already exists, so you may skip all these steps).
```shell
./bin/syntax -g [decaf dir]/parser.rs.g -m lalr1 -o [decaf dir]/lib.rs --validate
```

Finally build the project.
```shell
cd [decaf dir]
cargo run --release
```

After that you can run tests.
```shell
cd testcases
python runAll.py
```