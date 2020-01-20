use super::token::Token;
use crate::lexer::input::Input;
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
            '(' => Ok(Some(Token::OpeningParenthesis)),
            ')' => Ok(Some(Token::ClosingParenthesis)),
            ';' => Ok(Some(Token::Delimiter)),
            ',' => Ok(Some(Token::Comma)),
            '+' => Ok(Some(Token::Plus)),
            '-' => Ok(Some(Token::Minus)),
            '*' => Ok(Some(Token::Multiply)),
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
                    "def" => Token::Def,
                    "extern" => Token::Extern,
                    _ => Token::Identifier(ident),
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
                    .map(|x| Some(Token::Number(x)))
                    .map_err(|_| LexerError::NumberNotValid(val))
            }
            _ => Err(LexerError::NotRecognized(c)),
        }
    }
}

impl<I: Input> Iterator for Lexer<I> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read() {
            Ok(Some(x)) => Some(Ok(x)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_input_should_work() {}
}
