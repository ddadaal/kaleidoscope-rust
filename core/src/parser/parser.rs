use super::nodes::*;
use crate::lexer::Token::*;
use crate::lexer::*;
use crate::or_return;
use crate::util::buffer::Buffer;
use phf::phf_map;

static BINOP_PRECEDENCES: phf::Map<char, i8> = phf_map! {
    '<' => 10,
    '+' => 20,
    '-' => 20,
    '*' => 40,
};

fn get_binop_precedences(binop: char) -> ParseResult<i8> {
    BINOP_PRECEDENCES
        .get(&binop)
        .map(|x| *x)
        .ok_or(ParseError::new(Some(BinOp(binop)), "Unknown binop"))
}

#[derive(Debug)]
struct ParseError(Option<Token>, String);

impl ParseError {
    fn new(token: Option<Token>, message: &str) -> Self {
        ParseError(token, message.into())
    }
}

macro_rules! get_curr {
    ($s:expr,$err:tt) => {
        $s.curr().ok_or(ParseError::new(None, $err))?
    };
}

macro_rules! expect {
    ($s:expr,$expected:expr,$err:tt) => {{
        let token = $s.curr();
        if token != Some($expected) {
            return Err(ParseError::new(token.map(|x| x.clone()), $err));
        }
    }};
}

macro_rules! extract {
    ($s:expr,$expected:tt,$err:tt) => {{
        let token = $s.curr();
        if let Some($expected(inner)) = token {
            inner
        } else {
            return Err(ParseError::new(token.map(|x| x.clone()), $err));
        }
    }};
}

type ParseResult<T> = Result<T, ParseError>;

struct Parser<I: Iterator<Item = Token>> {
    buffer: Buffer<Token, I>,
}

impl<I: Iterator<Item = Token>> Parser<I> {
    pub fn new(lexer: I) -> Self {
        Parser {
            buffer: Buffer::new(lexer),
        }
    }

    /// program := []
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut program: Program = Vec::new();

        loop {
            let token = or_return!(self.buffer.curr(), Ok(program));
            let mut anonymous_fun_count = 0;
            let node: ASTNode = match token {
                Def => ASTNode::FunctionNode(self.parse_function()?),
                Extern => ASTNode::ExternNode(self.parse_extern()?),
                Delimiter => continue,
                _ => ASTNode::FunctionNode(Function {
                    prototype: Prototype {
                        name: format!("_anonymous_{}", {
                            anonymous_fun_count += 1;
                            anonymous_fun_count
                        }),
                        args: vec![],
                    },
                    body: self.parse_expression()?,
                }),
            };
            program.push(node);
            self.buffer.advance();
        }
    }

    #[inline]
    fn curr(&self) -> Option<&Token> {
        self.buffer.curr()
    }

    #[inline]
    fn peek(&self) -> Option<&Token> {
        self.buffer.peek()
    }

    #[inline]
    fn advance(&mut self) {
        self.buffer.advance()
    }

    fn parse_function(&mut self) -> ParseResult<Function> {
        self.buffer.advance(); // eat def
        println!("eat def");
        let prototype = self.parse_prototype()?;

        let body = self.parse_expression()?;
        Ok(Function { prototype, body })
    }

    fn parse_prototype(&mut self) -> ParseResult<Prototype> {
        // expect function name (identifier)
        let name = extract!(self, Identifier, "expect identifier in prototype").clone();

        // eat function name
        self.buffer.advance();

        // expect and eat (
        expect!(self, &OpeningParenthesis, "expect ( in prototype");
        self.buffer.advance();

        // read argument names
        let mut args = Vec::<String>::new();
        while let Identifier(arg_name) = get_curr!(self, "expect identifier or )") {
            args.push(arg_name.to_string());
            self.buffer.advance();
            println!("{:?}", args);
        }

        // expect )
        expect!(self, &ClosingParenthesis, "expect identifier or )");
        self.buffer.advance();

        Ok(Prototype { name, args })
    }

    fn parse_extern(&mut self) -> ParseResult<Prototype> {
        // eat extern
        self.buffer.advance();

        self.parse_prototype()
    }

    /// expression := primary binoprhs
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        let lhs = self.parse_primary()?;
        self.parse_bin_op_rhs(0, lhs)
    }

    /// binoprhs := ( + primary )*
    fn parse_bin_op_rhs(
        &mut self,
        min_expr_prec: i8,
        mut lhs: Expression,
    ) -> ParseResult<Expression> {
        loop {
            if let Some(BinOp(binop)) = self.curr() {
                let binop = *binop;
                let curr_prec = get_binop_precedences(binop)?;
                if curr_prec < min_expr_prec {
                    return Ok(lhs);
                }

                // eat binop
                self.advance();

                // parse next primary
                let mut rhs = self.parse_primary()?;

                // find if the next is still binop
                if let Some(BinOp(next_binop)) = self.curr() {
                    // if the next is still a binop, and it has higher precendence than curr
                    // than recursively call binop
                    if get_binop_precedences(*next_binop)? > curr_prec {
                        rhs = self.parse_bin_op_rhs(curr_prec + 1, rhs)?;
                    }
                }

                lhs = Expression::BinaryExpr(binop, Box::new(lhs), Box::new(rhs));
            } else {
                return Ok(lhs);
            }
        }
    }

    /// primary_expr     : [Identifier | Number | call_expr | parenthesis_expr];
    /// call_expr        : Ident OpeningParenthesis [expression Comma ?]* ClosingParenthesis;
    /// parenthesis_expr : OpeningParenthesis expression ClosingParenthesis;
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        let token = get_curr!(self, "expect a primary expression");
        match token {
            Identifier(_) => self.parse_identifier_expr(),
            Number(_) => self.parse_expression(),
            OpeningParenthesis => self.parse_parenthesis_expr(),
            _ => Err(ParseError::new(
                Some(token.clone()),
                "expect identifier, number or (",
            )),
        }
    }

    fn parse_number_expr(&mut self) -> ParseResult<Expression> {
        let number = extract!(self, Number, "expect a number");
        Ok(Expression::NumberExpr(*number))
    }

    /// parenthesis_expr : OpeningParenthesis expression ClosingParenthesis;
    fn parse_parenthesis_expr(&mut self) -> ParseResult<Expression> {
        // eat )
        self.buffer.advance();

        // get inner expression
        let expr = self.parse_expression()?;

        // eat )
        expect!(self, &ClosingParenthesis, "expect )");
        self.buffer.advance();

        Ok(expr)
    }

    /// identifier_expr : identifier
    ///                 : identifier ( expression* )
    fn parse_identifier_expr(&mut self) -> ParseResult<Expression> {
        // get identifier
        let identifier = extract!(self, Identifier, "expect identifier").clone();
        self.buffer.advance();

        // lookahead for whether its a call
        if self.peek() != Some(&OpeningParenthesis) {
            return Ok(Expression::VariableExpr(identifier));
        }

        // its a call
        // eat (
        self.advance();

        let mut args = Vec::<Expression>::new();

        // until a ) is reached
        while self.curr() != Some(&ClosingParenthesis) {
            let arg = self.parse_expression()?;
            args.push(arg);

            expect!(self, &Comma, "expect comma");
            self.advance();
        }

        // eat )
        self.advance();

        Ok(Expression::CallExpr(identifier, args))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn simple() {
        let program = "
         def fun1(a b)
         a+b*e-d
         ";
        let lexer = Lexer::new(program.chars());

        let tokens = lexer
            .take_while(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<Token>>();

        println!("{:?}", tokens);

        let mut parser = Parser::new(tokens.into_iter());

        let ast = parser.parse();

        println!("{:#?}", ast);
    }
}
