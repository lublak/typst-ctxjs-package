pub fn is_json(s: &[u8]) -> bool {
    let mut validator = JsonValidator::new(s);
    validator.validate().is_ok() && validator.is_end_of_input()
}

struct JsonValidator<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> JsonValidator<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn is_end_of_input(&mut self) -> bool {
        self.skip_whitespace();
        self.pos >= self.input.len()
    }

    fn peek(&self) -> Option<&u8> {
        self.input.get(self.pos)
    }

    fn advance(&mut self) -> Option<&u8> {
        let ch = self.input.get(self.pos);
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(&ch) = self.peek() {
            match ch {
                b' ' | b'\t' | b'\n' | b'\r' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn validate(&mut self) -> Result<(), ()> {
        self.skip_whitespace();
        match self.peek() {
            Some(&b'{') => self.parse_object(),
            Some(&b'[') => self.parse_array(),
            Some(&b'"') => self.parse_string(),
            Some(&(b'0'..=b'9') | &b'-' | &b'+' | &b'.' | &b'e' | &b'E') => self.parse_number(),
            Some(&b't') => self.parse_literal(b"true"),
            Some(&b'f') => self.parse_literal(b"false"),
            Some(&b'n') => self.parse_literal(b"null"),
            _ => Err(()),
        }
    }

    fn parse_object(&mut self) -> Result<(), ()> {
        if self.advance() != Some(&b'{') {
            return Err(());
        }

        self.skip_whitespace();
        if self.peek() == Some(&b'}') {
            self.advance();
            return Ok(());
        }

        loop {
            self.skip_whitespace();
            self.parse_string()?;

            self.skip_whitespace();
            if self.advance() != Some(&b':') {
                return Err(());
            }

            self.validate()?;

            self.skip_whitespace();
            match self.peek() {
                Some(&b',') => {
                    self.advance();
                }
                Some(&b'}') => {
                    self.advance();
                    return Ok(());
                }
                _ => return Err(()),
            }
        }
    }

    fn parse_array(&mut self) -> Result<(), ()> {
        if self.advance() != Some(&b'[') {
            return Err(());
        }

        self.skip_whitespace();
        if self.peek() == Some(&b']') {
            self.advance();
            return Ok(());
        }

        loop {
            self.validate()?;

            self.skip_whitespace();
            match self.peek() {
                Some(&b',') => {
                    self.advance();
                }
                Some(&b']') => {
                    self.advance();
                    return Ok(());
                }
                _ => return Err(()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<(), ()> {
        if self.advance() != Some(&b'"') {
            return Err(());
        }

        loop {
            match self.advance() {
                Some(&b'"') => return Ok(()),
                Some(&b'\\') => {
                    // consume escape + next char
                    if self.advance().is_none() {
                        return Err(());
                    }
                }
                Some(&c) if c < 0x20 => return Err(()),
                None => return Err(()),
                _ => {}
            }
        }
    }

    fn parse_number(&mut self) -> Result<(), ()> {
        // optional sign
        if matches!(self.peek(), Some(&b'+' | &b'-')) {
            self.advance();
        }

        // integer part
        match self.peek() {
            Some(&b'0') => {
                self.advance();
            }
            Some(&(b'1'..=b'9')) => {
                while matches!(self.peek(), Some(&(b'0'..=b'9'))) {
                    self.advance();
                }
            }
            _ => return Err(()),
        }

        // fractional part
        if self.peek() == Some(&b'.') {
            self.advance();
            if !matches!(self.peek(), Some(&(b'0'..=b'9'))) {
                return Err(());
            }
            while matches!(self.peek(), Some(&(b'0'..=b'9'))) {
                self.advance();
            }
        }

        // exponent part
        if matches!(self.peek(), Some(&b'e' | &b'E')) {
            self.advance();
            if matches!(self.peek(), Some(&b'+' | &b'-')) {
                self.advance();
            }
            if !matches!(self.peek(), Some(&(b'0'..=b'9'))) {
                return Err(());
            }
            while matches!(self.peek(), Some(&(b'0'..=b'9'))) {
                self.advance();
            }
        }

        Ok(())
    }

    fn parse_literal(&mut self, literal: &[u8]) -> Result<(), ()> {
        for &expected in literal {
            if self.advance() != Some(&expected) {
                return Err(());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::is_json;

    #[test]
    fn test_valid_primitives() {
        assert!(is_json(b"true"));
        assert!(is_json(b"false"));
        assert!(is_json(b"null"));
        assert!(is_json(b"0"));
        assert!(is_json(b"-0"));
        assert!(is_json(b"123"));
        assert!(is_json(b"-456"));
        assert!(is_json(b"3.14"));
        assert!(is_json(b"-0.5"));
        assert!(is_json(b"1e10"));
        assert!(is_json(b"1.23e-4"));
        assert!(is_json(b"1.23e+5"));
        assert!(is_json(b"1.23E+10"));
        assert!(is_json(b"\"hello\""));
        assert!(is_json(b"\"a\\\"b\""));
        assert!(is_json(b"\"\\n\""));
        assert!(is_json(b"\"\\t\""));
        assert!(is_json(b"\"\\b\""));
        assert!(is_json(b"\"\\f\""));
        assert!(is_json(b"\"\\r\""));
        assert!(is_json(b"\"\\u0000\""));
        assert!(is_json(b"\"\\u1234\""));
        assert!(is_json(b"\"\\uFFFF\""));
    }

    #[test]
    fn test_valid_objects() {
        assert!(is_json(b"{}"));
        assert!(is_json(b"{\"a\":1}"));
        assert!(is_json(b"{\"a\":1,\"b\":2}"));
        assert!(is_json(b"{\"x\":{}"));
        assert!(is_json(b"{\"x\":[1,2]}"));
        assert!(is_json(b"{\"a\": {\"b\": {\"c\": 3}}}"));
        assert!(is_json(b"{\"key\":\"value\"}"));
    }

    #[test]
    fn test_valid_arrays() {
        assert!(is_json(b"[]"));
        assert!(is_json(b"[1]"));
        assert!(is_json(b"[1,2,3]"));
        assert!(is_json(b"[\"a\",\"b\"]"));
        assert!(is_json(b"[1, \"two\", true, null]"));
        assert!(is_json(b"[1, [2, [3]]]"));
        assert!(is_json(b"[{},{},{}]"));
    }

    #[test]
    fn test_whitespace_handling() {
        assert!(is_json(b"  true  "));
        assert!(is_json(b"\nfalse\n"));
        assert!(is_json(b"\tnull\t"));
        assert!(is_json(b" [ 1 , 2 , 3 ] "));
        assert!(is_json(b"{ \"a\" : 1 }"));
        assert!(is_json(b"  { \n \"key\" : \"value\" }  "));
    }

    #[test]
    fn test_invalid_primitives() {
        // Numbers
        assert!(!is_json(b""));
        assert!(!is_json(b"123abc"));
        assert!(!is_json(b"1.2.3"));
        assert!(!is_json(b"e10"));
        assert!(!is_json(b"1e"));
        assert!(!is_json(b"1e+"));
        assert!(!is_json(b"+"));
        assert!(!is_json(b"-"));

        // Strings
        assert!(!is_json(b"\"unterminated"));
        assert!(!is_json(b"\"has\x00control\"")); // control char (0x00)
        assert!(!is_json(b"\"has\x1fcontrol\"")); // 0x1F is invalid
        assert!(!is_json(b"\"\\\""));
        assert!(!is_json(b"\"\\x\"")); // invalid escape
        assert!(!is_json(b"\"\\u\""));
        assert!(!is_json(b"\"\\u1\""));
        assert!(!is_json(b"\"\\u12\""));
        assert!(!is_json(b"\"\\u123\""));
        assert!(!is_json(b"\"\\uXXXX\"")); // non-hex

        // Literals
        assert!(!is_json(b"tru"));
        assert!(!is_json(b"tRue"));
        assert!(!is_json(b"tRUE"));
        assert!(!is_json(b"fals"));
        assert!(!is_json(b"nulll"));
    }

    #[test]
    fn test_invalid_syntax() {
        assert!(!is_json(b"{"));
        assert!(!is_json(b"{}{"));
        assert!(!is_json(b"["));
        assert!(!is_json(b"[]["));
        assert!(!is_json(b"{\"a\":1,\"b\":}"));
        assert!(!is_json(b"{\"a\":1,,\"b\":2}"));
        assert!(!is_json(b"[1,2,]"));
        assert!(!is_json(b"\"unclosed\\"));
        assert!(!is_json(b"\"\\\""));
        assert!(!is_json(b"\"\"\"")); // triple quote is not valid escape
        assert!(!is_json(b"123abc"));
        assert!(!is_json(b"123abc"));
    }

    #[test]
    fn test_empty_input() {
        assert!(!is_json(b""));
    }

    #[test]
    fn test_unicode_escapes() {
        assert!(is_json(b"\"\\u0000\"")); // null
        assert!(is_json(b"\"\\u001F\"")); // 0x1F
        assert!(is_json(b"\"\\u0020\"")); // space (valid)
        assert!(is_json(b"\"\\u007F\"")); // DEL
        assert!(is_json(b"\"\\u0080\"")); // next byte (still valid string — just not ASCII)
        assert!(is_json(b"\"\\u00FF\""));
        assert!(is_json(b"\"\\u1234\""));
        assert!(is_json(b"\"\\uABCD\""));
        assert!(is_json(b"\"\\uabcd\""));
        assert!(is_json(b"\"\\uABcd\""));

        // Invalid: non-hex digits
        assert!(!is_json(b"\"\\uGGGG\""));
        assert!(!is_json(b"\"\\u000G\""));
    }

    #[test]
    fn test_deep_nesting() {
        assert!(is_json(b"[[[[[[[[[[[]]]]]]]]]]]"));
        assert!(is_json(b"{\"a\":{\"b\":{\"c\":{\"d\":{}}}"));
        // But this is *invalid*:
        assert!(!is_json(b"{\"a\":{\"b\":{\"c\":{\"d\":{}}}")); // missing closing }
    }

    #[test]
    fn test_numbers_edge_cases() {
        assert!(is_json(b"0.0"));
        assert!(is_json(b"-0.0"));
        assert!(is_json(b"0e0"));
        assert!(is_json(b"0.0e0"));
        assert!(is_json(b"1.234567890"));
        assert!(is_json(b"12345678901234567890")); // large int

        // Negative exponent (even 0)
        assert!(is_json(b"1e0"));
        assert!(is_json(b"1.0e0"));
        assert!(is_json(b"1.0E-0"));
        assert!(is_json(b"1.0e+0"));
    }

    #[test]
    fn test_trailing_content() {
        assert!(!is_json(b"true "));
        assert!(!is_json(b"true x")); // extra char after whitespace
        assert!(!is_json(b"1234"));
        assert!(!is_json(b"1234 ")); // trailing whitespace *is* allowed
        assert!(!is_json(b"1234 5678")); // two numbers = invalid
        assert!(!is_json(b"[]{}")); // multiple top-level values
    }

    #[test]
    fn test_control_chars() {
        assert!(is_json(b"\"a b\"")); // space ✅
        assert!(is_json(b"\"a\tb\"")); // tab ✅
        assert!(is_json(b"\"a\nb\"")); // newline ✅
        assert!(is_json(b"\"a\\rb\"")); // \r ✅

        // But raw control chars in string are invalid
        assert!(!is_json(b"\"a\x00b\"")); // NUL
        assert!(!is_json(b"\"a\x01b\"")); // SOH
        assert!(!is_json(b"\"a\x1Fb\"")); // US (unit separator)
    }
}
