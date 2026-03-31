pub fn is_json(s: &[u8]) -> bool {
    let mut validator = JsonValidator::new(s);
    validator.skip_whitespace();
    if validator.validate_value() {
        validator.skip_whitespace();
        validator.is_end_of_input()
    } else {
        false
    }
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
        self.pos >= self.input.len()
    }

    fn current(&self) -> Option<u8> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            match ch {
                b' ' | b'\t' | b'\n' | b'\r' => self.advance(),
                _ => break,
            }
        }
    }

    fn validate_value(&mut self) -> bool {
        match self.current() {
            Some(b'{') => self.advance_and_validate_object(),
            Some(b'[') => self.advance_and_validate_array(),
            Some(b'"') => self.advance_and_validate_string(),

            Some(b'0') => self.advance_and_validate_fractional(),
            Some(b'-' | b'+') => self.advance_and_validate_signed_number(),
            Some((b'1'..=b'9')) => self.advance_and_validate_number(),

            Some(b't') => self.advance_and_parse_literal(b"rue"),
            Some(b'f') => self.advance_and_parse_literal(b"alse"),
            Some(b'n') => self.advance_and_parse_literal(b"ull"),
            _ => false,
        }
    }

    // |{...
    fn advance_and_validate_object(&mut self) -> bool {
        self.advance();
        self.skip_whitespace();

        // {|

        if self.current() == Some(b'}') {
            self.advance(); // {}|
            return true;
        }

        loop {
            if self.current() != Some(b'"') {
                return false;
            }

            // {"|

            if !self.advance_and_validate_string() {
                return false;
            }
            self.skip_whitespace();

            // {"key"|

            if self.current() != Some(b':') {
                return false;
            }
            self.advance();
            self.skip_whitespace();

            // {"key":|

            if !self.validate_value() {
                return false;
            }
            self.skip_whitespace();

            // {"key":value|

            match self.current() {
                Some(b',') => {
                    self.advance();
                    self.skip_whitespace();

                    // {"key":value,|
                }
                Some(b'}') => {
                    self.advance();
                    return true; // {"key":value}|
                }
                _ => return false,
            }
        }
    }

    // |[...
    fn advance_and_validate_array(&mut self) -> bool {
        self.advance();
        self.skip_whitespace();

        // [|

        if self.current() == Some(b']') {
            self.advance(); // []|
            return true;
        }

        loop {
            if !self.validate_value() {
                return false;
            }
            self.skip_whitespace();

            // [value|

            match self.current() {
                Some(b',') => {
                    self.advance();
                    self.skip_whitespace();

                    // [value,|
                }
                Some(b']') => {
                    self.advance();
                    return true; // [value]|
                }
                _ => return false,
            }
        }
    }

    fn validate_continuation_bytes(&mut self, len: usize) -> bool {
        for _ in 1..len {
            match self.current() {
                Some(c) if (c >> 6) == 0b10 => {
                    self.advance();
                }
                _ => return false,
            }
        }
        return true;
    }

    // |"
    fn advance_and_validate_string(&mut self) -> bool {
        self.advance();

        // "|

        loop {
            match self.current() {
                Some(b'"') => {
                    self.advance(); // ""|
                    return true;
                }
                Some(b'\\') => {
                    self.advance();
                    match self.current() {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                            self.advance(); // "\n|
                        }
                        Some(b'u') => {
                            self.advance();

                            // "\u|

                            // 4 hex digits
                            for _ in 0..4 {
                                match self.current() {
                                    Some(b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F') => {
                                        self.advance();
                                    }
                                    _ => return false,
                                }
                            }

                            // "\u0000|
                        }
                        _ => return false,
                    }
                }
                Some(b) => {
                    match b {
                        b if (b >> 7) == 0 => {
                            // one byte
                            self.advance();
                        }
                        b if (b >> 5) == 0b110 => {
                            // two bytes (110xxxxx 10xxxxxx)
                            self.advance();
                            if !self.validate_continuation_bytes(1) {
                                return false;
                            }
                        }
                        b if (b >> 4) == 0b1110 => {
                            // three bytes (1110xxxx 10xxxxxx 10xxxxxx)
                            self.advance();
                            if !self.validate_continuation_bytes(2) {
                                return false;
                            }
                        }
                        b if (b >> 3) == 0b11110 => {
                            // four bytes (11110xxx 10xxxxxx 10xxxxxx, 10xxxxxx)
                            self.advance();
                            if !self.validate_continuation_bytes(3) {
                                return false;
                            }
                        }
                        _ => return false,
                    };
                }
                _ => return false,
            }
        }
    }

    // |+0123.0123
    fn advance_and_validate_signed_number(&mut self) -> bool {
        self.advance();
        // +|0123.0123
        match self.current() {
            Some(b'0') => self.advance_and_validate_fractional(),
            Some((b'1'..=b'9')) => self.advance_and_validate_number(),
            _ => false,
        }
    }

    fn advance_and_validate_fractional(&mut self) -> bool {
        self.advance();
        self.validate_fractional()
    }

    // 123|.0123
    fn validate_fractional(&mut self) -> bool {
        if self.current() == Some(b'.') {
            self.advance();
            if !matches!(self.current(), Some((b'0'..=b'9'))) {
                return false;
            }
            while matches!(self.current(), Some((b'0'..=b'9'))) {
                self.advance();
            }
        }

        // exponent part
        if matches!(self.current(), Some(b'e' | b'E')) {
            self.advance();
            if matches!(self.current(), Some(b'+' | b'-')) {
                self.advance();
            }
            if !matches!(self.current(), Some((b'0'..=b'9'))) {
                return false;
            }
            while matches!(self.current(), Some((b'0'..=b'9'))) {
                self.advance();
            }
        }

        true
    }

    // |123.0123
    fn advance_and_validate_number(&mut self) -> bool {
        self.advance();

        // 1|23.0123
        while matches!(self.current(), Some((b'0'..=b'9'))) {
            self.advance();
        }

        // 123|.0123

        self.validate_fractional()
    }

    fn advance_and_parse_literal(&mut self, literal: &[u8]) -> bool {
        self.advance();

        for expected in literal {
            match self.current() {
                Some(c) if &c == expected => {
                    self.advance();
                }
                _ => return false,
            }
        }
        true
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
        assert!(is_json(b"{\"x\":{}}"));
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

        // invalid
        assert!(!is_json(b"\"\\uGGGG\""));
        assert!(!is_json(b"\"\\u000G\""));
        assert!(!is_json(b"\"a\x00b\"")); // NUL
        assert!(!is_json(b"\"a\x01b\"")); // SOH
        assert!(!is_json(b"\"a\x1Fb\"")); // US (unit separator)
    }

    #[test]
    fn test_deep_nesting() {
        assert!(is_json(b"[[[[[[[[[[[]]]]]]]]]]]"));
        assert!(is_json(b"{\"a\":{\"b\":{\"c\":{\"d\":{}}}}}"));

        // invalid
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
        assert!(!is_json(b"true x")); // extra char after whitespace
        assert!(!is_json(b"1234 5678")); // two numbers = invalid
        assert!(!is_json(b"[]{}")); // multiple top-level values
    }
}
