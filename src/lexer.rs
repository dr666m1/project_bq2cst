#[derive(PartialEq, Debug)]
enum TokenType {
    SEMICOLON,
    SHARP,
    EOF,
    ILLEGAL,
    SELECT,
    IDENT,
    CREATE,
    FROM,
    BRANKLINE,
    INTEGER,
    TEMPORAY,
    TABLE,
    AS,
    TYPE,
    LPAREN,
    RPAREN,
    FUNCTION,
    RETURNS,
}

#[derive(PartialEq, Debug)]
struct Token {
    token_type: TokenType,
    literal: String,
    line: usize,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
    line: usize,
}

impl Lexer {
    fn new(input: String) -> Lexer {
        let mut chars: Vec<char> = Vec::new();
        for i in input.chars() {
            chars.push(i)
        }
        let first_char = chars[0];
        Lexer {
            input: chars,
            position: 0,
            read_position: 1,
            ch: Some(first_char),
            line: 0,
        }
    }
    fn read_char(&mut self) {
        if self.input.len() <= self.read_position {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    fn read_identifier(&mut self) -> String {
        let first_position = self.position;
        while is_letter(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn next_token(&mut self) -> Token {
        self.skip_whitespaces();
        let ch = match self.ch {
            Some(ch) => ch,
            None => {
                return Token {
                    token_type: TokenType::EOF,
                    literal: "".to_string(),
                    line: self.line,
                }
            }
        };
        let token = match ch {
            ';' => Token {
                token_type: TokenType::SEMICOLON,
                literal: ch.to_string(),
                line: self.line,
            },
            '#' => Token {
                token_type: TokenType::SHARP,
                literal: ch.to_string(),
                line: self.line,
            },
            '(' => Token {
                token_type: TokenType::LPAREN,
                literal: ch.to_string(),
                line: self.line,
            },
            ')' => Token {
                token_type: TokenType::RPAREN,
                literal: ch.to_string(),
                line: self.line,
            },
            _ => {
                if ch.is_digit(10) {
                    let token_literal = self.read_number();
                    return Token {
                        token_type: TokenType::INTEGER,
                        literal: token_literal,
                        line: self.line,
                    }
                } else if is_letter(&self.ch) {
                    let token_literal = self.read_identifier();
                    return self.lookup_keyword(token_literal); // note: the ownerwhip moves
                } else {
                    Token {
                        token_type: TokenType::ILLEGAL,
                        literal: ch.to_string(),
                        line: self.line,
                    }
                }
            }
        };
        self.read_char();
        token
    }
    fn skip_whitespaces(&mut self) {
        while self.skip_whitespace() {
            self.read_char();
        }
    }
    fn skip_whitespace(&mut self) -> bool {
        let ch = match self.ch {
            Some(ch) => ch,
            None => return false,
        };
        match ch {
            '\n' => {
                self.line += 1;
                true
            }
            _ => ch.is_whitespace(),
        }
    }
    fn peek_char(&mut self) -> Option<char> {
        if self.input.len() <= self.read_position {
            None
        } else {
            Some(self.input[self.read_position])
        }
    }
    fn lookup_keyword(&self, keyword: String) -> Token {
        let s = keyword.to_ascii_uppercase();
        let s = s.as_str();
        match s {
            "SELECT" => Token {
                token_type: TokenType::SELECT,
                literal: keyword,
                line: self.line,
            },
            "FROM" => Token {
                token_type: TokenType::FROM,
                literal: keyword,
                line: self.line,
            },
            "CREATE" => Token {
                token_type: TokenType::CREATE,
                literal: keyword,
                line: self.line,
            },
            "TEMP" => Token {
                token_type: TokenType::TEMPORAY,
                literal: keyword,
                line: self.line,
            },
            "TEMPORAY" => Token {
                token_type: TokenType::TEMPORAY,
                literal: keyword,
                line: self.line,
            },
            "FUNCTION" => Token {
                token_type: TokenType::FUNCTION,
                literal: keyword,
                line: self.line,
            },
            "RETURNS" => Token {
                token_type: TokenType::RETURNS,
                literal: keyword,
                line: self.line,
            },
            "AS" => Token {
                token_type: TokenType::AS,
                literal: keyword,
                line: self.line,
            },
            "INT64" => Token {
                token_type: TokenType::TYPE,
                literal: keyword,
                line: self.line,
            },
            _ => Token {
                token_type: TokenType::IDENT,
                literal: keyword,
                line: self.line,
            },
        }
    }
    fn read_number(&mut self) -> String {
        let first_position = self.position;
        while is_digit(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
}

fn is_letter(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic() || ch.is_digit(10),
        None => false,
    }
}

fn is_digit(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_digit(10),
        None => false,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = "#standardSQL
            SELECT 10 From;
            CREATE TEMP FUNCTION RETURNS INT64 AS (0);"
            .to_string();
        let mut l = Lexer::new(input);
        let expected_tokens: Vec<Token> = vec![
            Token {
                token_type: TokenType::SHARP,
                literal: "#".to_string(),
                line: 0,
            },
            Token {
                token_type: TokenType::IDENT,
                literal: "standardSQL".to_string(),
                line: 0,
            },
            Token {
                token_type: TokenType::SELECT,
                literal: "SELECT".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::INTEGER,
                literal: "10".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::FROM,
                literal: "From".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::CREATE,
                literal: "CREATE".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::TEMPORAY,
                literal: "TEMP".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::FUNCTION,
                literal: "FUNCTION".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::RETURNS,
                literal: "RETURNS".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::TYPE,
                literal: "INT64".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::AS,
                literal: "AS".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::LPAREN,
                literal: "(".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::INTEGER,
                literal: "0".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::RPAREN,
                literal: ")".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::SEMICOLON,
                literal: ";".to_string(),
                line: 2,
            },
            Token {
                token_type: TokenType::EOF,
                literal: "".to_string(),
                line: 2,
            },
        ];
        for t in expected_tokens {
            assert_eq!(l.next_token(), t);
        }
    }

    #[test]
    fn test_surrogate() {
        let code = "𩸽";
        for c in code.chars() {
            assert_eq!(c, '𩸽'); // treated as one character
        }
    }

    #[test]
    fn test_chars2string() {
        let chars = vec!['#', ';', 'S', 'E', 'L', 'E', 'C', 'T'];
        let str: String = chars[0..2].into_iter().collect();
        assert_eq!(str, "#;".to_string());
    }

    #[test]
    fn test_is_letter() {
        assert!('a'.is_alphabetic());
        assert!('z'.is_alphabetic());
        assert!('𩸽'.is_alphabetic());
        assert!(!';'.is_alphabetic());
        assert!(';'.is_ascii());
        assert!('z'.is_ascii());
        assert!('0'.is_numeric());
        assert!('0'.is_ascii());
        assert!(!'0'.is_alphabetic());
        assert!(!'9'.is_alphabetic());
    }

    #[test]
    fn test_cr() {
        assert_eq!(
            "
",
            "\n"
        )
    }
}
