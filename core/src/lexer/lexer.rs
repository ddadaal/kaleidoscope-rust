use super::token::Token;
use crate::lexer::input::Input;
use crate::lexer::token::Token::*;
use crate::{or_break, or_return};

#[derive(Debug, PartialEq)]
pub enum LexerError {
    NumberNotValid(String),
    NotRecognized(char),
}

struct Lexer<I: Input> {
    /// The source of input
    input: I,
}

pub type LexerResult = Result<Token, LexerError>;

impl<I: Input> Lexer<I> {
    pub fn new(input: I) -> Self {
        Lexer { input }
    }
    pub fn read(&mut self) -> LexerResult {
        // Read a char
        // If no more input, return Ok(None)
        let mut c = or_return!(self.input.curr_char(), Ok(EOF));
        // Skip whitespaces
        while c.is_whitespace() {
            c = or_return!(self.input.advance(), Ok(EOF));
        }
        // eat current.
        self.input.advance();
        // Now iter is at the next char
        match c {
            // Simple cases
            '(' => Ok(OpeningParenthesis),
            ')' => Ok(ClosingParenthesis),
            ';' => Ok(Delimiter),
            ',' => Ok(Comma),
            '+' => Ok(Plus),
            '-' => Ok(Minus),
            '*' => Ok(Multiply),
            // handle comment by getting until the eol
            '#' => {
                loop {
                    c = or_break!(self.input.curr_char());
                    // Got \n, line has end
                    // Eat \n and continue parsing
                    self.input.advance();
                    if c == '\n' {
                        break;
                    }
                }
                self.read()
            }
            // Get a letter, it may be a identifier, or a keyword
            _ if c.is_alphabetic() => {
                let mut ident = c.to_string();
                // Collect all alphanumeric chars
                while {
                    self.input.curr_char().map_or(false, |x| {
                        c = x;
                        x.is_alphanumeric()
                    })
                } {
                    ident.push(c);
                    self.input.advance();
                }
                Ok(match ident.as_ref() {
                    "def" => Def,
                    "extern" => Extern,
                    _ => Identifier(ident),
                })
            }
            // Get a digit, it may be a digit.
            _ if c.is_ascii_digit() || c == '.' => {
                let mut val = c.to_string();
                // Collect all numbers and at most one dot (.).
                while {
                    self.input.curr_char().map_or(false, |x| {
                        c = x;
                        c == '.' || c.is_ascii_digit()
                    })
                } {
                    self.input.advance();
                    val.push(c);
                }
                val.parse::<f64>()
                    .map(|x| Number(x))
                    .map_err(|_| LexerError::NumberNotValid(val))
            }
            _ => Err(LexerError::NotRecognized(c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::input::StringInput;

    #[test]
    fn identifier() {
        assert_eq!(read_all("ident"), vec![Identifier("ident".into())]);
        assert_eq!(read_all("ident123"), vec![Identifier("ident123".into())]);
        assert_eq!(
            read_all("ident123 als"),
            vec!["ident123", "als"]
                .into_iter()
                .map(|x| Identifier(x.to_string()))
                .collect::<Vec<Token>>()
        );
    }

    #[test]
    fn keywords_and_symbols() {
        assert_eq!(
            read_all("def extern ; ( ) , + - *"),
            vec![
                Def,
                Extern,
                Delimiter,
                OpeningParenthesis,
                ClosingParenthesis,
                Comma,
                Plus,
                Minus,
                Multiply
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            read_all("123 12 .4 1234. 12345.6"),
            vec![123.0, 12.0, 0.4, 1234.0, 12345.6]
                .into_iter()
                .map(|x| Number(x))
                .collect::<Vec<Token>>()
        );
    }

    #[test]
    fn complete_program() {
        assert_eq!(
            read_all(
                "
                extern sin(a)
            def aFunction(a, b)
                a+4*b-3.2;
            "
            ),
            vec![
                Extern,
                Identifier("sin".into()),
                OpeningParenthesis,
                Identifier("a".into()),
                ClosingParenthesis,
                Def,
                Identifier("aFunction".into()),
                OpeningParenthesis,
                Identifier("a".into()),
                Comma,
                Identifier("b".into()),
                ClosingParenthesis,
                Identifier("a".into()),
                Plus,
                Number(4.0),
                Multiply,
                Identifier("b".into()),
                Minus,
                Number(3.2),
                Delimiter,
            ]
        );
    }

    #[test]
    fn malformed_numbers() {
        expect_err("1.4.2", LexerError::NumberNotValid("1.4.2".into()));
        // expect_err(".4.2", LexerError::NumberNotValid(".4.2".into()));
    }

    fn read_all(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(StringInput::new(input));
        let mut result: Vec<Token> = Vec::new();
        while let Ok(res) = lexer.read() {
            match res {
                EOF => break,
                _ => result.push(res),
            }
        }

        if let Err(err) = lexer.read() {
            print!("Err: {:?}", err);
        }

        result
    }

    fn expect_err(input: &str, err: LexerError) {
        let mut lexer = Lexer::new(StringInput::new(input));
        loop {
            let result = lexer.read();
            if result.is_err() {
                assert_eq!(Err(err), result);
                break;
            }
        }
    }
}
