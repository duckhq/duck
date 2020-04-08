use crate::builds::BuildStatus;
use crate::DuckResult;
use std::fmt::Display;

mod lexer;
mod parser;

pub fn parse<T: Into<String>>(expression: T) -> DuckResult<Expression> {
    match parse_expression(expression) {
        Ok(expression) => Ok(expression),
        Err(e) => Err(format_err!("Error parsing expression: {}", e)),
    }
}

fn parse_expression<T: Into<String>>(expression: T) -> DuckResult<Expression> {
    let expression = expression.into();
    parser::parse(&mut lexer::tokenize(&expression[..])?)
}

///////////////////////////////////////////////////////////
// Visitor

pub trait Visitor<TContext, TResult> {
    fn or(&self, ctx: &TContext, left: &Expression, right: &Expression) -> DuckResult<TResult>;
    fn and(&self, ctx: &TContext, left: &Expression, right: &Expression) -> DuckResult<TResult>;
    fn not(&self, ctx: &TContext, exp: &Expression) -> DuckResult<TResult>;
    fn constant(&self, ctx: &TContext, constant: &Constant) -> DuckResult<TResult>;
    fn property(&self, ctx: &TContext, property: &Property) -> DuckResult<TResult>;
    fn scope(&self, ctx: &TContext, exp: &Expression) -> DuckResult<TResult>;
    fn relational(
        &self,
        ctx: &TContext,
        left: &Expression,
        right: &Expression,
        op: &Operator,
    ) -> DuckResult<TResult>;
}

///////////////////////////////////////////////////////////
// AST

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Constant(Constant),
    Property(Property),
    Relational(Box<Expression>, Box<Expression>, Operator),
    Scope(Box<Expression>),
}

impl Expression {
    pub fn accept<TContext, TResult>(
        &self,
        ctx: &TContext,
        visitor: &dyn Visitor<TContext, TResult>,
    ) -> DuckResult<TResult> {
        match self {
            Expression::And(lhs, rhs) => visitor.and(ctx, lhs, rhs),
            Expression::Or(lhs, rhs) => visitor.or(ctx, lhs, rhs),
            Expression::Not(expression) => visitor.not(ctx, expression),
            Expression::Constant(constant) => visitor.constant(ctx, constant),
            Expression::Property(property) => visitor.property(ctx, property),
            Expression::Relational(lhs, rhs, op) => visitor.relational(ctx, lhs, rhs, op),
            Expression::Scope(expression) => visitor.scope(ctx, expression),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Constant {
    Boolean(bool),
    Integer(i64),
    String(String),
    Status(BuildStatus),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    EqualTo,
    NotEqualTo,
    GreaterThan,
    GreaterThanOrEqualTo,
    LessThan,
    LessThanOrEqualTo,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::EqualTo => write!(f, "=="),
            Operator::NotEqualTo => write!(f, "!="),
            Operator::GreaterThan => write!(f, ">"),
            Operator::GreaterThanOrEqualTo => write!(f, ">="),
            Operator::LessThan => write!(f, "<"),
            Operator::LessThanOrEqualTo => write!(f, "<="),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Property {
    Branch,
    Status,
    Project,
    Definition,
    Build,
    Collector,
    Provider,
}
