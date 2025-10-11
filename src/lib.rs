use chumsky::{input::ValueInput, prelude::*, primitive::select, text::ascii::ident};
use ariadne::{Color, Label, Report, ReportKind, Source};

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'src>{
    Error,
    Value(Value<'src>),
    Local(&'src str),
    Neg(Box<Spanned<Self>>),
    Let(&'src str, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Spanned<Vec<Spanned<Self>>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Assign,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'src> {
    Null,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Str(&'src str),
    Func(&'src str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arg<'src>{
    name: &'src str,
    type_: &'src str
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func<'src> {
    name: &'src str,
    args: Vec<(Arg<'src>, Span)>,
    rtype: &'src str,
    body: Spanned<Expr<'src>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src>{
    Fn,
    Let,
    Arrow,
    Colon,
    SemiColon,
    Comma,
    Op(BinaryOp),
    Bool(bool),
    I32(i32),
    Ctrl(char),
    Ident(&'src str),
}

fn lexer<'src>() 
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, Span>>> {

    let num = text::int(10)
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::I32);// TODO

    let minus_or_arrow = 
        just('-')
        .then(just('>').or_not())
        .to_slice()
        .map(|op| if op == "->" { Token::Arrow } else { Token::Op(BinaryOp::Sub)});

    let equal_or_assign = 
        just('=')
        .then(just('=').or_not())
        .to_slice()
        .map(|op| if op == "==" { Token::Op(BinaryOp::Eq)} else {Token::Op(BinaryOp::Assign)});

    let op = 
        choice((
            just('+').to(Token::Op(BinaryOp::Add)),
            just('*').to(Token::Op(BinaryOp::Mul)),
            just('/').to(Token::Op(BinaryOp::Div)),
            just('%').to(Token::Op(BinaryOp::Mod)),
            just("!=").to(Token::Op(BinaryOp::NotEq)),
        ));

    let ctrl = one_of("{}[]()").map(Token::Ctrl);
    let ident = 
        text::ascii::ident().map(|ident| match ident {
            "fn" => Token::Fn,
            "let" => Token::Let,
            _ => Token::Ident(ident),
        }).labelled("ident");

    let token = 
        minus_or_arrow
        .or(equal_or_assign)
        .or(
            choice((
                num.labelled("number"),
                ctrl.labelled("ctrl"),
                op.labelled("Operator"),
                just(':').to(Token::Colon).labelled("Colon"),
                just(';').to(Token::SemiColon).labelled("SemiColon"),
                just(',').to(Token::Comma).labelled("Comma"),
            ))
        )
        .or(ident);

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

fn args_parser<'tokens, 'src: 'tokens, I>() 
-> impl Parser<'tokens, I, Vec<Spanned<Arg<'src>>>, extra::Err<Rich<'tokens, Token<'src>, Span>>> 
    where I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
{
    let ident = select!{ Token::Ident(ident) => ident };
    let arg = group(
        (
            ident,
            just(Token::Colon).labelled("Colon"),
            ident,
        ))
        .map_with(
            |(name, _, type_), e| 
                (Arg{ name, type_ }, e.span())
            )
        .separated_by(
            just(Token::Comma)
            .labelled("Comma")
            .map_with(|tok, e| (tok, e.span())))
        .allow_trailing()
        .collect::<Vec<_>>();
    arg
}

fn func_parser<'tokens, 'src: 'tokens, I>() 
    -> impl Parser<'tokens, I, Vec<Spanned<Func<'src>>>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
    where I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
{
    let ident = select!{ Token::Ident(ident) => ident };
    let fn_header = 
        just(Token::Fn)
            .then(ident)
            .then(args_parser())
            .then_ignore(just(Token::Arrow).labelled("Arrow"))
            .then(ident);

    let func_def = fn_header
        .then(expr_parser().delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}'))))
        .map_with(|((((_, name), args), rtype), body), e| {
            (Func { name, args, rtype, body } ,e.span())
        });

    func_def
        .repeated()
        .collect::<Vec<_>>()
}

fn expr_parser<'tokens, 'src: 'tokens, I>() 
-> impl Parser<'tokens, I, Spanned<Expr<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
    where I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
{
    recursive(|expr|{
        let int = select! { Token::I32(i) => Expr::Value(Value::Int32(i))};
        let ident = select! { Token::Ident(i) => Expr::Local(i) };
        let atom = 
            int.map_with(|tok, e| (tok, e.span()))
            .or(
                expr.delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
            )
            .or(ident.map_with(|a, e| (a, e.span())));

        let unary = 
            just(Token::Op(BinaryOp::Sub))
            .repeated()
            .foldr(atom, |_op, rhs|{
                rhs
            });

        // let select_op = select! { Token::Op(i) => i };
        // Product ops (multiply and divide) have equal precedence
        let op = 
            just(Token::Op(BinaryOp::Mul)).to(BinaryOp::Mul)
            .or(just(Token::Op(BinaryOp::Div)).to(BinaryOp::Div));

        let product = unary
            .clone()
            .foldl_with(op.then(unary).repeated(), |a, (op, b), e| {
                (Expr::Binary(Box::new(a), op, Box::new(b)),e.span())
            });

        let op = 
            just(Token::Op(BinaryOp::Add)).to(BinaryOp::Add)
            .or(just(Token::Op(BinaryOp::Sub)).to(BinaryOp::Sub));
        let sum = product
            .clone()
            .foldl_with(op.then(product).repeated(), |a, (op, b), e| {
                (Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
            });

        let op = 
            just(Token::Op(BinaryOp::Eq)).to(BinaryOp::Eq)
            .or(just(Token::Op(BinaryOp::NotEq)).to(BinaryOp::NotEq));
        let compare = sum
            .clone()
        .foldl_with(op.then(sum).repeated(), |a, (op, b), e| {
            (Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
        });
        compare
    })
}

fn aaa() {
    let input = r#"
        fn name_hello a:num, b:num, c:string -> bool {
            let hello = 42 + 1;
        }

        fn name_world a:num, b:num, c:string -> bool {

        }
"#;
    // fn map lst:list<T>, func:<fn T -> U> -> list<U>
    println!("input: {}", input);
    let (tokens, err) = lexer().parse(input).into_output_errors();

    if let Some(tokens) = tokens {
        println!("{:#?}", tokens);
        let parse_result = func_parser().parse(
            tokens
                .as_slice()
                .map((input.len()..input.len()).into(), |(t, s)| (t, s))
        ).into_result();

        match parse_result {
            Ok(a) => {
                println!("{:#?}", a);
            }
            Err(errs) => {
                println!("{:#?}", errs);
                for err in errs { 
                    Report::build(ReportKind::Error, ((), err.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_code(3)
                    //.with_message(err.to_string())
                    .with_label(
                        Label::new(((), err.span().into_range()))
                            //.with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(input))
                    .unwrap();
                }
            }
        }
    }
    else 
    {
        println!("lexer error");
        println!("{:?}", err);
    }
}

fn bbb() {
    let input = r#"
42 + 1 + hello
"#;
    println!("input: {}", input);
    let (tokens, err) = lexer().parse(input).into_output_errors();

    if let Some(tokens) = tokens {
        println!("{:#?}", tokens);
        let parse_result = expr_parser().parse(
            tokens
                .as_slice()
                .map((input.len()..input.len()).into(), |(t, s)| (t, s))
        ).into_result();

        match parse_result {
            Ok(a) => {
                println!("{:#?}", a);
            }
            Err(errs) => {
                println!("{:#?}", errs);
                for err in errs { 
                    Report::build(ReportKind::Error, ((), err.span().into_range()))
                    .with_config(ariadne::Config::new().with_index_type(ariadne::IndexType::Byte))
                    .with_code(3)
                    //.with_message(err.to_string())
                    .with_label(
                        Label::new(((), err.span().into_range()))
                            //.with_message(err.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(input))
                    .unwrap();
                }
            }
        }
    }
    else 
    {
        println!("lexer error");
        println!("{:?}", err);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works00() {
        aaa()
    }

    #[test]
    fn it_works01() {
        bbb()
    }
}
