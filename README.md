# Build

```shell
npm install -g syntax-cli
syntax-cli -g parser/parser.rs.g -m lalr1 -o parser/src/lib.rs --validate
python fix.py
cargo build 
```