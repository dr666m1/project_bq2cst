use crate::token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
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
            column: 0,
        }
    }
    fn read_char(&mut self) {
        if self.input.len() <= self.read_position {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
            if self.input[self.position] == '\n' {
                self.column = 0;
                self.line += 1;
            } else {
                self.column += 1;
            }
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    fn read_identifier(&mut self) -> String {
        let first_position = self.position;
        while is_letter_or_digit(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_parameter(&mut self) -> String {
        let first_position = self.position;
        while self.ch == Some('@') {
            self.read_char();
        }
        self.read_identifier();
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    pub fn next_token(&mut self) -> Option<token::Token> {
        self.skip_whitespace();
        let ch = match self.ch {
            Some(ch) => ch,
            None => {
                return None;
            }
        };
        let token = match ch {
            ',' => token::Token {
                literal: ch.to_string(),
                line: self.line,
                column: self.column,
            },
            ';' => token::Token {
                literal: ch.to_string(),
                line: self.line,
                column: self.column,
            },
            '.' => {
                let next_ch = self.peek_char(1).unwrap();
                if next_ch == '`' || next_ch.is_alphabetic() {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                } else {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_number(),
                    });
                }
            }
            '#' => {
                return Some(token::Token {
                    line: self.line,
                    column: self.column,
                    literal: self.read_comment(),
                });
            }
            //'(' => token::Token {
            //    literal: ch.to_string(),
            //    line: self.line,
            //    column: self.column,
            //},
            //')' => token::Token {
            //    literal: ch.to_string(),
            //    line: self.line,
            //    column: self.column,
            //},
            //'[' => token::Token {
            //    literal: ch.to_string(),
            //    line: self.line,
            //    column: self.column,
            //},
            //']' => token::Token {
            //    literal: ch.to_string(),
            //    line: self.line,
            //    column: self.column,
            //},
            // quotation
            '`' => {
                return Some(token::Token {
                    line: self.line,
                    column: self.column,
                    literal: self.read_quoted(self.ch),
                })
            }
            '"' => {
                if self.peek_char(1) == Some('"') && self.peek_char(2) == Some('"') {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_multiline_string(),
                    });
                } else {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_quoted(self.ch),
                    });
                }
            }
            '\'' => {
                if self.peek_char(1) == Some('\'') && self.peek_char(2) == Some('\'') {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_multiline_string(),
                    });
                } else {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_quoted(self.ch),
                    });
                }
            }
            // binary operator
            '+' => token::Token {
                literal: ch.to_string(),
                line: self.line,
                column: self.column,
            },
            '-' => {
                if self.peek_char(1) == Some('-') {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_comment(),
                    });
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '*' => token::Token {
                line: self.line,
                column: self.column,
                literal: ch.to_string(),
            },
            '/' => {
                if self.peek_char(1) == Some('*') {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_multiline_comment(),
                    });
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            '|' => {
                if self.peek_char(1) == Some('|') {
                    self.read_char();
                    token::Token {
                        line: self.line,
                        column: self.column - 1,
                        literal: "||".to_string(),
                    }
                } else {
                    token::Token {
                        line: self.line,
                        column: self.column,
                        literal: ch.to_string(),
                    }
                }
            }
            // parameter
            '?' => token::Token {
                line: self.line,
                column: self.column,
                literal: ch.to_string(),
            },
            '@' => {
                return Some(token::Token {
                    line: self.line,
                    column: self.column,
                    literal: self.read_parameter(),
                });
            }
            // other
            _ => {
                if ch.is_digit(10) {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_number(),
                    });
                } else if is_letter_or_digit(&self.ch) {
                    return Some(token::Token {
                        line: self.line,
                        column: self.column,
                        literal: self.read_identifier(),
                    }); // note: the ownerwhip moves
                } else {
                    token::Token {
                        literal: ch.to_string(),
                        line: self.line,
                        column: self.column,
                    }
                }
            }
        };
        self.read_char();
        Some(token)
    }
    fn read_multiline_string(&mut self) -> String {
        let first_position = self.position;
        let ch = self.ch;
        self.read_char(); // first ' -> secont '
        while !(self.ch == ch && self.peek_char(1) == ch && self.peek_char(2) == ch) {
            self.read_char();
        }
        self.read_char(); // first ' -> secont '
        self.read_char(); // second ' -> third '
        self.read_char(); // third ' ->  next_ch
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_quoted(&mut self, quote: Option<char>) -> String {
        let first_position = self.position;
        self.read_char();
        while self.ch != quote {
            if self.ch == Some('\\') {
                //self.read_escaped_char();
                panic!();
            } else {
                self.read_char();
            }
        }
        self.read_char();
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn skip_whitespace(&mut self) {
        while is_whitespace(&self.ch) {
            self.read_char();
        }
    }
    fn peek_char(&mut self, offset: usize) -> Option<char> {
        if self.input.len() <= self.read_position {
            None
        } else {
            Some(self.input[self.read_position - 1 + offset])
        }
    }
    fn read_number(&mut self) -> String {
        let first_position = self.position;
        while is_digit_or_period(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_comment(&mut self) -> String {
        let first_position = self.position;
        while !is_end_of_line(&self.ch) {
            self.read_char();
        }
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
    fn read_multiline_comment(&mut self) -> String {
        let first_position = self.position;
        while !(self.ch == Some('*') && self.peek_char(1) == Some('/')) {
            self.read_char();
        }
        self.read_char(); // * -> /
        self.read_char(); // / -> next_char
        self.input[first_position..self.position]
            .into_iter()
            .collect()
    }
}

fn is_letter_or_digit(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_alphabetic() || ch.is_digit(10) || ch == &'_',
        None => false,
    }
}

fn is_digit_or_period(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_digit(10) || ch == &'.' || ch == &'E' || ch == &'e',
        None => false,
    }
}

fn is_end_of_line(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch == &'\n',
        None => true, // EOF is treated as end of line
    }
}

fn is_whitespace(ch: &Option<char>) -> bool {
    match ch {
        Some(ch) => ch.is_whitespace(),
        None => false, // EOF is treated as end of line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_next_token() {
        let input = "#standardSQL
SELECT 10, 1.1, 'aaa' || \"bbb\", .9, 1-1+2/2*3, date '2000-01-01', timestamp '2000-01-01',col1,date_add(col1, interval 9 hour),.1E4,?,@@param,'''abc''',arr[offset(1)],ARRAY<INT64>[1],
From `data`; -- comment
-- 
/*
e
o
f
*/"
        .to_string();
        let mut l = Lexer::new(input);
        let expected_tokens: Vec<token::Token> = vec![
            // line 0
            token::Token {
                line: 0,
                column: 0,
                literal: "#standardSQL".to_string(),
            },
            // line 1
            token::Token {
                line: 1,
                column: 0,
                literal: "SELECT".to_string(),
            },
            token::Token {
                line: 1,
                column: 7,
                literal: "10".to_string(),
            },
            token::Token {
                line: 1,
                column: 9,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 11,
                literal: "1.1".to_string(),
            },
            token::Token {
                line: 1,
                column: 14,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 16,
                literal: "'aaa'".to_string(),
            },
            token::Token {
                line: 1,
                column: 22,
                literal: "||".to_string(),
            },
            token::Token {
                line: 1,
                column: 25,
                literal: "\"bbb\"".to_string(),
            },
            token::Token {
                line: 1,
                column: 30,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 32,
                literal: ".9".to_string(),
            },
            token::Token {
                line: 1,
                column: 34,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 36,
                literal: "1".to_string(),
            },
            token::Token {
                line: 1,
                column: 37,
                literal: "-".to_string(),
            },
            token::Token {
                line: 1,
                column: 38,
                literal: "1".to_string(),
            },
            token::Token {
                line: 1,
                column: 39,
                literal: "+".to_string(),
            },
            token::Token {
                line: 1,
                column: 40,
                literal: "2".to_string(),
            },
            token::Token {
                line: 1,
                column: 41,
                literal: "/".to_string(),
            },
            token::Token {
                line: 1,
                column: 42,
                literal: "2".to_string(),
            },
            token::Token {
                line: 1,
                column: 43,
                literal: "*".to_string(),
            },
            token::Token {
                line: 1,
                column: 44,
                literal: "3".to_string(),
            },
            token::Token {
                line: 1,
                column: 45,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 47,
                literal: "date".to_string(),
            },
            token::Token {
                line: 1,
                column: 52,
                literal: "'2000-01-01'".to_string(),
            },
            token::Token {
                line: 1,
                column: 64,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 66,
                literal: "timestamp".to_string(),
            },
            token::Token {
                line: 1,
                column: 76,
                literal: "'2000-01-01'".to_string(),
            },
            token::Token {
                line: 1,
                column: 88,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 89,
                literal: "col1".to_string(),
            },
            token::Token {
                line: 1,
                column: 93,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 94,
                literal: "date_add".to_string(),
            },
            token::Token {
                line: 1,
                column: 102,
                literal: "(".to_string(),
            },
            token::Token {
                line: 1,
                column: 103,
                literal: "col1".to_string(),
            },
            token::Token {
                line: 1,
                column: 107,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 109,
                literal: "interval".to_string(),
            },
            token::Token {
                line: 1,
                column: 118,
                literal: "9".to_string(),
            },
            token::Token {
                line: 1,
                column: 120,
                literal: "hour".to_string(),
            },
            token::Token {
                line: 1,
                column: 124,
                literal: ")".to_string(),
            },
            token::Token {
                line: 1,
                column: 125,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 126,
                literal: ".1E4".to_string(),
            },
            token::Token {
                line: 1,
                column: 130,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 131,
                literal: "?".to_string(),
            },
            token::Token {
                line: 1,
                column: 132,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 133,
                literal: "@@param".to_string(),
            },
            token::Token {
                line: 1,
                column: 140,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 141,
                literal: "'''abc'''".to_string(),
            },
            token::Token {
                line: 1,
                column: 150,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 151,
                literal: "arr".to_string(),
            },
            token::Token {
                line: 1,
                column: 154,
                literal: "[".to_string(),
            },
            token::Token {
                line: 1,
                column: 155,
                literal: "offset".to_string(),
            },
            token::Token {
                line: 1,
                column: 161,
                literal: "(".to_string(),
            },
            token::Token {
                line: 1,
                column: 162,
                literal: "1".to_string(),
            },
            token::Token {
                line: 1,
                column: 163,
                literal: ")".to_string(),
            },
            token::Token {
                line: 1,
                column: 164,
                literal: "]".to_string(),
            },
            token::Token {
                line: 1,
                column: 165,
                literal: ",".to_string(),
            },
            token::Token {
                line: 1,
                column: 166,
                literal: "ARRAY".to_string(),
            },
            token::Token {
                line: 1,
                column: 171,
                literal: "<".to_string(),
            },
            token::Token {
                line: 1,
                column: 172,
                literal: "INT64".to_string(),
            },
            token::Token {
                line: 1,
                column: 177,
                literal: ">".to_string(),
            },
            token::Token {
                line: 1,
                column: 178,
                literal: "[".to_string(),
            },
            token::Token {
                line: 1,
                column: 179,
                literal: "1".to_string(),
            },
            token::Token {
                line: 1,
                column: 180,
                literal: "]".to_string(),
            },
            token::Token {
                line: 1,
                column: 181,
                literal: ",".to_string(),
            },
            // line2
            token::Token {
                line: 2,
                column: 0,
                literal: "From".to_string(),
            },
            token::Token {
                line: 2,
                column: 5,
                literal: "`data`".to_string(),
            },
            token::Token {
                line: 2,
                column: 11,
                literal: ";".to_string(),
            },
            token::Token {
                line: 2,
                column: 13,
                literal: "-- comment".to_string(),
            },
            // line3
            token::Token {
                line: 3,
                column: 0,
                literal: "-- ".to_string(),
            },
            // line4
            token::Token {
                line: 4,
                column: 0,
                literal: "/*
e
o
f
*/"
                .to_string(),
            },
        ];
        for t in expected_tokens {
            assert_eq!(l.next_token().unwrap(), t);
        }
        assert_eq!(l.ch, None);
    }
}
