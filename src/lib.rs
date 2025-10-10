use chumsky::{input::ValueInput, prelude::*};

#[derive(Clone, Debug)]
pub enum Stmt <'src>{
    Expr,
    //If(&'src str, Vec<Stmt<'src>>),
    Var(&'src str),
    Loop(Vec<Stmt<'src>>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arg<'src>{
    name: &'src str,
    type_: &'src str
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func<'src> {
    name: &'src str,
    args: Vec<Arg<'src>>,
    rtype: &'src str
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src>{
    Fn,
    Arrow,
    Colon,
    SemiColon,
    Comma,
    Op(char),
    Ctrl(char),
    Ident(&'src str),
}

fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token<'src>>, extra::Err<Rich<'src, char>>> {
    let minus_or_arrow = just('-')
        .then(just('>').or_not())
        .to_slice()
        .map(|op| if op == "->" { Token::Arrow } else { Token::Op('-')});
    let ctrl = one_of("").map(Token::Ctrl);
    let ident = 
        text::ascii::ident().map(|ident| match ident {
            "fn" => Token::Fn,
            _ => Token::Ident(ident),
        }).labelled("ident");

    let token = minus_or_arrow
        .or(ctrl)
        .or(just(':').to(Token::Colon).labelled("Colon"))
        .or(just(';').to(Token::SemiColon).labelled("SemiColon"))
        .or(just(',').to(Token::Comma).labelled("Comma"))
        .or(ident);

    token
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

fn args_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Vec<Arg<'src>>> 
    where I: ValueInput<'tokens, Token = Token<'src>>
{
    let ident = select!{ Token::Ident(ident) => ident };
    let arg = group(
        (
            ident,
            just(Token::Colon), 
            ident,
        ))
        .map(
            |(name, _, type_)| 
                Arg{ name, type_ }
            )
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>();
    arg
}

fn func_parser<'tokens, 'src: 'tokens, I>() -> impl Parser<'tokens, I, Vec<Func<'src>>>
    where I: ValueInput<'tokens, Token = Token<'src>>
{
    let ident = select!{ Token::Ident(ident) => ident };
    let fn_header = 
        just(Token::Fn)
            .then(ident)
            .then(args_parser())
            .then_ignore(just(Token::Arrow))
            .then(ident)
            .map(|(((_, name), args), rtype)| {
                Func { name, args, rtype, }
            });

    fn_header
        .repeated()
        .collect::<Vec<_>>()
}

// fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Stmt<'a>>> {
// }

fn aaa() {
    let input = r#"
        fn name_hello a:num,b:num -> bool 
"#;
    println!("input: {}", input);
    let (tokens, err) = lexer().parse(input).into_output_errors();

    if let Some(tokens) = tokens {
        let stmts = func_parser().parse(
            tokens
                .as_slice()
        );
        println!("{:#?}", stmts.output());
        println!("{:?}", stmts.errors().collect::<Vec<_>>());
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
}
