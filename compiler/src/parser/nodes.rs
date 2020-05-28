#[derive(Debug)]
pub enum ASTNode {
    EOF,
    Delimiter,
    ExternNode(Prototype),
    FunctionNode(Function),
}

/// definition : Def prototype expression;
#[derive(PartialEq, Clone, Debug)]
pub struct Function {
    pub prototype: Prototype,
    pub body: Expression,
}

/// prototype : Identifier ( [Identifier ,]* )
#[derive(PartialEq, Clone, Debug)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}

/// expression : [primaryexpr (Op primary_expr)*];
/// primaryexpr : identifierexpr
///             : numberexpr
///             : parenexpr
#[derive(PartialEq, Clone, Debug)]
pub enum Expression {
    NumberExpr(f64),
    VariableExpr(String),
    BinaryExpr(char, Box<Expression>, Box<Expression>),
    CallExpr(String, Vec<Expression>),
}
