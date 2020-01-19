use super::token::Token;
use crate::lexer::input::Input;
use std::str::CharIndices;

pub enum LexerError {
    NumberNotValid(String),
    NotRecognized(char),
}

struct Lexer<I: Input> {
    /// The source of input
    input: I,
}

impl<I: Input> Iterator for Lexer<I> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Read a char
        let mut c = match self.input.curr_char() {
            Some(x) => x,
            None => return None,
        };

        // Skip whitespaces
        while c.is_whitespace() {
            self.input.advance();
            c = match self.input.curr_char() {
                Some(x) => x,
                None => return None,
            };
        }

        match c {
            // Simple cases
            '(' => Some(Ok(Token::OpeningParenthesis)),
            ')' => Some(Ok(Token::ClosingParenthesis)),
            ';' => Some(Ok(Token::Delimiter)),
            ',' => Some(Ok(Token::Comma)),
            '+' => Some(Ok(Token::Plus)),
            '-' => Some(Ok(Token::Minus)),
            '*' => Some(Ok(Token::Multiply)),
            // handle comment by getting until the eol
            '#' => {
                loop {
                    self.input.advance();

                    c = match self.input.curr_char() {
                        Some(x) => x,
                        None => break,
                    };

                    // Got \n, line has end
                    // Eat \n and continue parsing
                    if c == '\n' {
                        self.input.advance();
                        break;
                    }
                }
                self.next()
            }

            // Get a letter, it may be a identifier, or a keyword
            _ if c.is_alphabetic() => {
                let mut ident = c.to_string();

                // Collect all alphanumeric chars
                loop {
                    self.input.advance();

                    c = match self.input.curr_char() {
                        Some(x) => x,
                        // The input ends.
                        None => break,
                    };
                    if c.is_alphanumeric() {
                        ident.push(c);
                    } else {
                        break;
                    }
                }

                Some(Ok(match ident.as_ref() {
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
                    self.input.advance();

                    c = match self.input.curr_char() {
                        Some(x) => x,
                        // The input ends.
                        None => break,
                    };

                    if c == '.' {
                        if dot_exists {
                            return Some(Err(LexerError::NumberNotValid(val)));
                        }
                        dot_exists = true;
                        val.push(c);
                    } else if c.is_ascii_digit() {
                        val.push(c);
                    } else {
                        break;
                    }
                }

                Some(
                    val.parse::<f64>()
                        .map(|x| Token::Number(x))
                        .map_err(|_| LexerError::NumberNotValid(val)),
                )
            }
            _ => Some(Err(LexerError::NotRecognized(c))),
        }
    }
}

impl<I: Input> Lexer<I> {
    pub fn new(input: I) -> Self {
        Lexer { input }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_input_should_work() {

    }
}

