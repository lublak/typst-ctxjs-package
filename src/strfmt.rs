use std::{collections::HashMap, str::Utf8Error};

pub fn strfmt(s: &[u8], m: &HashMap<&str, String>) -> Result<Vec<u8>, Utf8Error> {
    let l = s.len();
    let mut output = Vec::with_capacity(l);
    let mut i = 0;
    while i < l {
        match s[i] {
            b'{' => {
                i += 1;
                let mut key_start = i;
                while i < l {
                    let c = s[i];
                    match c {
                        b'}' => {
                            if let Some(value) = m.get(&str::from_utf8(&s[key_start..i])?) {
                                for ele in value.as_bytes() {
                                    output.push(*ele);
                                }
                            } else {
                                output.push(b'{');
                                for k in key_start..i {
                                    output.push(s[k]);
                                }
                                output.push(c);
                            }
                            i += 1;
                            break;
                        }
                        (b'a'..=b'z') | (b'A'..=b'Z') | (b'0'..=b'9') | b'_' | b'-' => {
                            i += 1;
                        }
                        b'{' => {
                            output.push(b'{');
                            i += 1;
                            key_start = i;
                        }
                        _ => {
                            output.push(b'{');
                            for k in key_start..i {
                                output.push(s[k]);
                            }
                            output.push(c);
                            i += 1;
                            break;
                        }
                    }
                }
            }
            b => {
                output.push(b);
                i += 1;
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::strfmt;

    fn test(s: &[u8], m: &HashMap<&str, String>, t: &[u8]) -> bool {
        match strfmt(s, m) {
            Ok(res) => res.iter().eq(t.iter()),
            Err(_) => false,
        }
    }

    #[test]
    fn test_valid_literals() {
        let mut kv = HashMap::<&str, String>::new();
        kv.insert("key", "value".to_string());
        kv.insert("hello", "world".to_string());
        kv.insert("foo", "bar".to_string());

        assert!(test(b"test", &kv, b"test"));
        assert!(test(b"{key}", &kv, b"value"));
        assert!(test(b"{invalid}", &kv, b"{invalid}"));

        assert!(test(b"{invalid}", &kv, b"{invalid}"));

        assert!(test(
            b" {key} test {foo} more {hello}",
            &kv,
            b" value test bar more world"
        ));

        assert!(test(b"test {open value", &kv, b"test {open value"));

        assert!(test(b"{{key}", &kv, b"{value"));
    }
}
