#[derive(PartialEq, Clone, Debug)]
pub enum Token {
    Def,
    Extern,
    Delimiter, //';' character
    OpeningParenthesis,
    ClosingParenthesis,
    Comma,
    Identifier(String),
    Number(f64),
    Plus,
    Minus,
    Multiply,
}
