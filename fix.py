# maybe the author of the parser generator made some mistakes about regex

with open('parser/src/lib.rs', 'r') as f:
    s = f.read()
    s = s.replace(r'r"^"[^\"]*""', r'"^\"[^\"]*\""')
    s = s.replace('r"^\/"', 'r"^/"')
    with open('parser/src/lib.rs', 'w') as f1:
        f1.write(s)