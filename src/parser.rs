use crate::{
    lexer::{Ctrl, Kw, Op, Token},
    Span, Spanned,
};
use chumsky::{input::SpannedInput, prelude::*};

#[derive(Clone, Debug)]
pub enum Statement<'src> {
    Expr(Spanned<Expr<'src>>),
    Function {
        name: Spanned<&'src str>,
        parameters: Spanned<Vec<Spanned<&'src str>>>,
        body: Spanned<Vec<Spanned<Statement<'src>>>>,
    },
    Return(Spanned<Expr<'src>>),
    Break,
    Continue,
    Print(Spanned<Expr<'src>>),
    Loop(Spanned<Vec<Spanned<Statement<'src>>>>),
}

#[derive(Clone, Debug)]
pub enum Expr<'src> {
    Variable(Spanned<&'src str>),
    Boolean(Spanned<bool>),
    Integer(Spanned<i32>),
    Null,
    Binary(
        Box<Spanned<Expr<'src>>>,
        Spanned<BinaryOp>,
        Box<Spanned<Expr<'src>>>,
    ),
    Unary(Spanned<UnaryOp>, Box<Spanned<Expr<'src>>>),
    Call(Box<Spanned<Expr<'src>>>, Spanned<Vec<Spanned<Expr<'src>>>>),
}

#[derive(Clone, Copy, Debug)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Equals,
    NotEquals,
    Plus,
    Minus,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Equals => write!(f, "=="),
            BinaryOp::NotEquals => write!(f, "!="),
            BinaryOp::Plus => write!(f, "+"),
            BinaryOp::Minus => write!(f, "-"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum UnaryOp {
    Negate,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
        }
    }
}

type ParserInput<'tokens, 'src> = SpannedInput<Token<'src>, Span, &'tokens [(Token<'src>, Span)]>;

type ParserError<'tokens, 'src> = extra::Err<Rich<'tokens, Token<'src>, Span>>;

pub fn parser<'tokens, 'src: 'tokens>() -> impl Parser<
    'tokens,
    ParserInput<'tokens, 'src>,
    Vec<Spanned<Statement<'src>>>,
    ParserError<'tokens, 'src>,
> {
    statement_parser().repeated().collect().then_ignore(end())
}

fn statement_parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Statement<'src>>, ParserError<'tokens, 'src>>
{
    recursive(|statement| {
        let expr = expr_parser()
            .then_ignore(just(Token::Ctrl(Ctrl::SemiColon)))
            .map_with(|expr, e| (Statement::Expr(expr), e.span()))
            .boxed();

        let function = just(Token::Kw(Kw::Func))
            .ignore_then(ident())
            .then(
                ident()
                    .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                    .collect()
                    .delimited_by(just(Token::Ctrl(Ctrl::Pipe)), just(Token::Ctrl(Ctrl::Pipe)))
                    .map_with(|parameters, e| (parameters, e.span())),
            )
            .then(
                statement
                    .clone()
                    .repeated()
                    .collect()
                    .then_ignore(just(Token::Kw(Kw::End)))
                    .map_with(|body, e| (body, e.span())),
            )
            .map_with(|((name, parameters), body), e| {
                (
                    Statement::Function {
                        name,
                        parameters,
                        body,
                    },
                    e.span(),
                )
            })
            .boxed();

        let return_ = just(Token::Kw(Kw::Return))
            .ignore_then(expr_parser())
            .then_ignore(just(Token::Ctrl(Ctrl::SemiColon)))
            .map_with(|expr, e| (Statement::Return(expr), e.span()))
            .boxed();

        let break_ = just(Token::Kw(Kw::Break))
            .then_ignore(just(Token::Ctrl(Ctrl::SemiColon)))
            .map_with(|_, e| (Statement::Break, e.span()))
            .boxed();

        let continue_ = just(Token::Kw(Kw::Continue))
            .then_ignore(just(Token::Ctrl(Ctrl::SemiColon)))
            .map_with(|_, e| (Statement::Continue, e.span()))
            .boxed();

        let print = just(Token::Kw(Kw::Print))
            .ignore_then(expr_parser())
            .then_ignore(just(Token::Ctrl(Ctrl::SemiColon)))
            .map_with(|expr, e| (Statement::Print(expr), e.span()))
            .boxed();

        let loop_ = just(Token::Kw(Kw::Loop))
            .ignore_then(
                statement
                    .repeated()
                    .collect()
                    .then_ignore(just(Token::Kw(Kw::End)))
                    .map_with(|body, e| (body, e.span())),
            )
            .map_with(|body, e| (Statement::Loop(body), e.span()))
            .boxed();

        choice((expr, function, return_, break_, continue_, print, loop_)).boxed()
    })
}

fn expr_parser<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<Expr<'src>>, ParserError<'tokens, 'src>>
{
    recursive(|expression| {
        let variable = select! {
            Token::Variable(name) => name
        }
        .map_with(|variable, e| (Expr::Variable((variable, e.span())), e.span()));

        let boolean = select! {
            Token::Boolean(b) => b
        }
        .map_with(|boolean, e| (Expr::Boolean((boolean, e.span())), e.span()));

        let integer = select! {
            Token::Integer(n) => n
        }
        .map_with(|integer, e| (Expr::Integer((integer, e.span())), e.span()));

        let null = just(Token::Null)
            .ignored()
            .map_with(|_, e| (Expr::Null, e.span()))
            .boxed();

        let parenthesized_expr = expression.clone().delimited_by(
            just(Token::Ctrl(Ctrl::LeftParen)),
            just(Token::Ctrl(Ctrl::RightParen)),
        );

        let atom = choice((variable, boolean, integer, null, parenthesized_expr)).boxed();

        let call = atom
            .foldl(
                expression
                    .separated_by(just(Token::Ctrl(Ctrl::Comma)))
                    .collect()
                    .delimited_by(
                        just(Token::Ctrl(Ctrl::LeftParen)),
                        just(Token::Ctrl(Ctrl::RightParen)),
                    )
                    .map_with(|arguments, e| (arguments, e.span()))
                    .repeated(),
                |expr: (_, SimpleSpan), arguments: (_, SimpleSpan)| {
                    let span = expr.1.start..arguments.1.end;

                    (Expr::Call(Box::new(expr), arguments), span.into())
                },
            )
            .boxed();

        let negate_op = just(Token::Op(Op::Minus))
            .to(UnaryOp::Negate)
            .map_with(|op, e| (op, e.span()));

        let negate = negate_op
            .repeated()
            .foldr(call, |op: (_, SimpleSpan), expr| {
                let span = op.1.start..expr.1.end;

                (Expr::Unary(op, Box::new(expr)), SimpleSpan::from(span))
            })
            .boxed();

        let factor_op = choice((
            just(Token::Op(Op::Multiply)).to(BinaryOp::Multiply),
            just(Token::Op(Op::Divide)).to(BinaryOp::Divide),
        ))
        .map_with(|t, e| (t, e.span()));

        let factor = negate
            .clone()
            .foldl(factor_op.then(negate).repeated(), |lhs, (op, rhs)| {
                let span = lhs.1.start..rhs.1.end;

                (Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span.into())
            })
            .boxed();

        let sum_op = choice((
            just(Token::Op(Op::Plus)).to(BinaryOp::Plus),
            just(Token::Op(Op::Minus)).to(BinaryOp::Minus),
        ))
        .map_with(|t, e| (t, e.span()));

        let sum = factor
            .clone()
            .foldl(sum_op.then(factor).repeated(), |lhs, (op, rhs)| {
                let span = lhs.1.start..rhs.1.end;

                (Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span.into())
            })
            .boxed();

        let equality_op = choice((
            just(Token::Op(Op::Equals)).to(BinaryOp::Equals),
            just(Token::Op(Op::NotEquals)).to(BinaryOp::NotEquals),
        ))
        .map_with(|t, e| (t, e.span()));

        sum.clone()
            .foldl(equality_op.then(sum).repeated(), |lhs, (op, rhs)| {
                let span = lhs.1.start..rhs.1.end;

                (Expr::Binary(Box::new(lhs), op, Box::new(rhs)), span.into())
            })
            .boxed()
    })
}

fn ident<'tokens, 'src: 'tokens>(
) -> impl Parser<'tokens, ParserInput<'tokens, 'src>, Spanned<&'src str>, ParserError<'tokens, 'src>>
{
    select! {
        Token::Variable(name) => name
    }
    .map_with(|name, e| (name, e.span()))
}
