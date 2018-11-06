pub fn quote(s: &str) -> String {
    let mut ret = "\"".to_string();
    for ch in s.chars() {
        match ch {
            '"' => ret.push_str("\\\""),
            '\n' => ret.push_str("\\n"),
            '\t' => ret.push_str("\\t"),
            '\\' => ret.push_str("\\\\"),
            ch => ret.push(ch),
        };
    }
    ret + "\""
}
