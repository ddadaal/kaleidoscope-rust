use super::token::Token;
use crate::lexer::token::Token::*;
use crate::or_return;
use crate::util::buffer::Buffer;

#[derive(Debug, PartialEq)]
pub enum LexerError {
    NumberNotValid(String),
    NotRecognized(char),
}

pub struct Lexer<I: Iterator<Item = char>> {
    /// The source of input
    buffer: Buffer<char, I>,
}

pub type LexerResult = Result<Token, LexerError>;

impl<I: Iterator<Item = char>> Lexer<I> {
    pub fn new(char_iter: I) -> Self {
        Lexer {
            buffer: Buffer::new(char_iter),
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Read a char
        // If no more input, return None
        let mut c: char = *or_return!(self.buffer.curr(), None);

        // Skip whitespaces
        while c.is_whitespace() {
            self.buffer.advance();
            c = *or_return!(self.buffer.curr(), None);
        }

        // eat current
        self.buffer.advance();

        // handle comment by getting until eol
        if c == '#' {
            while {
                self.buffer.curr().map_or(false, |x| {
                    c = *x;
                    c != '\n'
                })
            } {
                self.buffer.advance();
            }
            return self.next();
        }

        // handle all other case
        Some(match c {
            // Simple cases
            '(' => Ok(OpeningParenthesis),
            ')' => Ok(ClosingParenthesis),
            ';' => Ok(Delimiter),
            ',' => Ok(Comma),
            '+' | '-' | '*' => Ok(BinOp(c)),
            // Get a letter, it may be a identifier, or a keyword
            _ if c.is_alphabetic() => {
                let mut ident = c.to_string();
                // Collect all alphanumeric chars
                while {
                    self.buffer.curr().map_or(false, |x| {
                        c = *x;
                        x.is_alphanumeric()
                    })
                } {
                    ident.push(c);
                    self.buffer.advance();
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
                    self.buffer.curr().map_or(false, |x| {
                        c = *x;
                        c == '.' || c.is_ascii_digit()
                    })
                } {
                    self.buffer.advance();
                    val.push(c);
                }
                val.parse::<f64>()
                    .map(|x| Number(x))
                    .map_err(|_| LexerError::NumberNotValid(val))
            }
            _ => Err(LexerError::NotRecognized(c)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! tokens {
        ( $( $x:expr),*) => {
            tokens!($($x,)*)
        };
        ( $( $x:expr,)* ) => {
            {
                let mut temp_vec = Vec::new();
                $(
                    temp_vec.push(Ok($x));
                )*
                temp_vec
            }
        };
    }

    #[test]
    fn identifier() {
        assert_eq!(read_all("ident"), tokens![Identifier("ident".into())]);
        assert_eq!(read_all("ident123"), tokens![Identifier("ident123".into())]);
        assert_eq!(
            read_all("ident123 als"),
            tokens![Identifier("ident123".into()), Identifier("als".into())]
        );
    }

    #[test]
    fn keywords_and_symbols() {
        assert_eq!(
            read_all("def extern ; ( ) , + - *"),
            tokens![
                Def,
                Extern,
                Delimiter,
                OpeningParenthesis,
                ClosingParenthesis,
                Comma,
                BinOp('+'),
                BinOp('-'),
                BinOp('*'),
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            read_all("123 12 .4 1234. 12345.6"),
            tokens![
                Number(123.0),
                Number(12.0),
                Number(0.4),
                Number(1234.0),
                Number(12345.6),
            ]
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
            tokens![
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
                BinOp('+'),
                Number(4.0),
                BinOp('*'),
                Identifier("b".into()),
                BinOp('-'),
                Number(3.2),
                Delimiter,
            ]
        );
    }

    #[test]
    fn malformed_numbers() {
        assert_eq!(
            read_all("1.4.2"),
            vec![Err(LexerError::NumberNotValid("1.4.2".into()))]
        );
        assert_eq!(
            read_all(".4.2"),
            vec![Err(LexerError::NumberNotValid(".4.2".into()))]
        );
    }

    #[test]
    fn comments() {
        assert_eq!(
            read_all(
                "
            123 #12312321ojff
            def
            "
            ),
            tokens![Number(123.0), Def,]
        );
        assert_eq!(read_all("123 #12312321ojff"), tokens![Number(123.0),]);
    }

    fn read_all(input: &str) -> Vec<Result<Token, LexerError>> {
        Lexer::new(input.chars()).collect()
    }
}
