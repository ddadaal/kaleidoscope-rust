use super::input::StringInput;
use super::token::Token;
use crate::lexer::input::Input;
use crate::lexer::token::Token::*;
use crate::{or_break, or_return};
pub enum LexerError {
    NumberNotValid(String),
    NotRecognized(char),
}

struct Lexer<I: Input> {
    /// The source of input
    input: I,
}

pub type LexerResult = Result<Option<Token>, LexerError>;

impl<I: Input> Lexer<I> {
    pub fn new(input: I) -> Self {
        Lexer { input }
    }
    pub fn read(&mut self) -> LexerResult {
        // Read a char
        // If no more input, return Ok(None)
        let mut c = or_return!(self.input.curr_char(), Ok(None));

        // Skip whitespaces
        while c.is_whitespace() {
            c = or_return!(self.input.advance(), Ok(None));
        }
        match c {
            // Simple cases
            '(' => Ok(Some(OpeningParenthesis)),
            ')' => Ok(Some(ClosingParenthesis)),
            ';' => Ok(Some(Delimiter)),
            ',' => Ok(Some(Comma)),
            '+' => Ok(Some(Plus)),
            '-' => Ok(Some(Minus)),
            '*' => Ok(Some(Multiply)),
            // handle comment by getting until the eol
            '#' => {
                loop {
                    c = or_break!(self.input.advance());
                    // Got \n, line has end
                    // Eat \n and continue parsing
                    if c == '\n' {
                        self.input.advance();
                        break;
                    }
                }
                self.read()
            }
            // Get a letter, it may be a identifier, or a keyword
            _ if c.is_alphabetic() => {
                let mut ident = c.to_string();
                // Collect all alphanumeric chars
                loop {
                    c = or_break!(self.input.advance());
                    if c.is_alphanumeric() {
                        ident.push(c);
                    } else {
                        break;
                    }
                }
                Ok(Some(match ident.as_ref() {
                    "def" => Def,
                    "extern" => Extern,
                    _ => Identifier(ident),
                }))
            }
            // Get a digit, it may be a digit.
            _ if c.is_ascii_digit() => {
                let mut val = c.to_string();
                let mut dot_exists = false;
                // Collect all numbers and at most one dot (.).
                loop {
                    c = or_break!(self.input.advance());
                    if c == '.' {
                        if dot_exists {
                            return Err(LexerError::NumberNotValid(val));
                        }
                        dot_exists = true;
                        val.push(c);
                    } else if c.is_ascii_digit() {
                        val.push(c);
                    } else {
                        break;
                    }
                }
                val.parse::<f64>()
                    .map(|x| Some(Number(x)))
                    .map_err(|_| LexerError::NumberNotValid(val))
            }
            _ => Err(LexerError::NotRecognized(c)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            read_all("123 .4 1234. 12345.6"),
            vec![123.0, 0.4, 1234.0, 12345.6]
                .into_iter()
                .map(|x| Number(x))
                .collect::<Vec<Token>>()
        );
    }

    fn read_all(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(StringInput::new(input));
        let mut result: Vec<Token> = Vec::new();
        while let Ok(res) = lexer.read() {
            match res {
                Some(x) => result.push(x),
                None => break,
            }
        }

        result
    }
}
