use chumsky::{input::ValueInput, prelude::*};


pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'src>{
    Error,
    Value(Value<'src>),
    Local(&'src str),
    Neg(Box<Spanned<Self>>),
    Let(&'src str, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    If (
        Box<Spanned<Self>>,        // cond
        Box<Spanned<Self>>,        // if
        Box<Option<Spanned<Self>>> // else
    ),
    Sed(Sed<'src>),
    Call(Box<Spanned<Self>>, Spanned<Vec<Spanned<Self>>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Return(Spanned<Vec<Spanned<Self>>>),
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
pub struct Sed <'src>{
    pub code: Vec<Value<'src>>
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value<'src> {
    Null,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Str(String),
    Func(&'src str),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arg<'src>{
    pub name: &'src str,
    pub type_: &'src str
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func<'src> {
    pub public: bool,
    pub name: &'src str,
    pub args: Vec<(Arg<'src>, Span)>,
    pub rtype: Vec<(&'src str, Span)>,
    pub body: Spanned<Expr<'src>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'src>{
    // 予約語
    Pub,
    Fn,
    Let,
    If,
    Else,
    Sed,
    Return,
    // 記号
    Arrow,
    Colon,
    SemiColon,
    Comma,
    // 特殊記号
    MbOpen, // ${
    MbClose, // }$
    Op(BinaryOp),
    Bool(bool),
    I32(i32),
    Ctrl(char),
    Ident(&'src str),
    Str(String),
}

fn lexer<'src>() 
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, Span>>> {
    let escape = 
        just('\\').ignore_then(choice((
            just('\\'),
            just('"'),
            just('n').to('\n'),
            just('t').to('\t'),
        )));

    let string_char = 
        none_of("\"\\");

    let string_contents = 
        string_char
        .or(escape)
        .repeated()
        .collect::<String>()
        .map(|tok|
           Token::Str(tok)
        );

    let string = 
        string_contents
        .delimited_by(just('"'), just('"'));

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

    let brace_open = 
        just('$')
        .or_not()
        .then(just('{'))
        .to_slice()
        .map(|tok| if tok == "${" {Token::MbOpen} else {Token::Ctrl('{')});

    let brace_close = 
        just('}')
        .then(just('$').or_not())
        .to_slice()
        .map(|tok| if tok == "}$" {Token::MbClose} else {Token::Ctrl('}')});
        
    let ctrl = 
        choice((
                brace_open,
                brace_close,
                one_of("[]()").map(Token::Ctrl),
        ));

    let ident = 
        text::ascii::ident().map(|ident| match ident {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "pub" => Token::Pub,
            "if" => Token::If,
            "else" => Token::Else,
            "sed" => Token::Sed,
            "return" => Token::Return,
            _ => Token::Ident(ident),
        })
        .labelled("ident");

    let token = 
        string
        .or(minus_or_arrow)
        .or(equal_or_assign)
        .or(
            choice((
                num.labelled("number"),
                ctrl.labelled("ctrl"),
                op.labelled("operator"),
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

pub fn func_parser<'tokens, 'src: 'tokens, I>() 
    -> impl Parser<'tokens, I, Vec<Spanned<Func<'src>>>, extra::Err<Rich<'tokens, Token<'src>, Span>>>
    where I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
{
    let ident = select!{ Token::Ident(ident) => ident };
    let fn_header = 
        just(Token::Fn)
            .then(ident)
            .then(args_parser())
            .then_ignore(just(Token::Arrow).labelled("Arrow"))
            .then(
                ident
                .map_with(|a, e| (a, e.span()))
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
            );

    let func_def = 
        just(Token::Pub)
        .or_not()
        .then(fn_header)
        .then(
            decl_parser()
            .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
            .recover_with(via_parser(nested_delimiters(
                    Token::Ctrl('{'),
                    Token::Ctrl('}'),
                    [
                        (Token::Ctrl('('), Token::Ctrl(')')),
                        (Token::Ctrl('['), Token::Ctrl(']')),
                    ],
                    |span| (Expr::Error, span),
            )))
        )
        .map_with(
            |((public, (((_, name), args), rtype)), body), e|{
            let public = match public {
                Some(_) => true,
                None => false, 
            };
            (Func { public, name, args, rtype, body }, e.span())
        }).labelled("function");

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

        let items = expr
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .collect::<Vec<_>>();

        let atom = 
            int.map_with(|tok, e| (tok, e.span()))
            .or(
                expr.delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
            )
            .or(ident.map_with(|a, e| (a, e.span())));

        let call = 
            atom
            .foldl_with(
                items
                    .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                    .map_with(|args, e| (args, e.span()))
                    .repeated(),
                |f, args, e| (
                    Expr::Call(Box::new(f), args), e.span()),
            );


        let unary = 
            just(Token::Op(BinaryOp::Sub))
            .repeated()
            .foldr(call, |_op, rhs|{
                rhs
            });

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

        let op = 
            just(Token::Op(BinaryOp::Assign)).to(BinaryOp::Assign);
        let assign = compare
            .clone()
            .foldl_with(op.then(compare).repeated(), |a, (op, b), e|{
                (Expr::Binary(Box::new(a), op, Box::new(b)), e.span())
            }) ;

        assign
    })
}

fn decl_parser<'tokens, 'src: 'tokens, I>() 
-> impl Parser<'tokens, I, Spanned<Expr<'src>>, extra::Err<Rich<'tokens, Token<'src>, Span>>> + Clone
    where I: ValueInput<'tokens, Token = Token<'src>, Span = Span>
{
    let ident = select! { Token::Ident(i) => i };

    recursive(|decl|{
        let r#let = 
            just(Token::Let)
            .ignore_then(ident)
            .then_ignore(just(Token::Op(BinaryOp::Assign)))
            .then(expr_parser())
            .then_ignore(just(Token::SemiColon))
            .then(decl.clone())
            .map_with(|((name, rhs), then), e| {
                (
                    Expr::Then(
                        Box::new((Expr::Let(name , Box::new(rhs)), e.span())),
                        Box::new(then)
                    ), e.span()
                )
            });

        let r#if = 
            just(Token::If)
            .ignore_then(
                expr_parser()
            )
            .then(
                decl.clone()
                .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
            )
            .then(
                just(Token::Else)
                .ignore_then(
                    decl.clone()
                    .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
                ).or_not()
            )
            .map_with(|((cond, then), else_), e| {
                (Expr::If(Box::new(cond), Box::new(then), Box::new(else_)), e.span())
            });

        let as_expr = 
            choice((
                expr_parser() 
                .then_ignore(
                    just(Token::SemiColon)
                ),
                r#if
            ));

        // sed_compiler用拡張
        let sed_str = select! { Token::Str(i) => Value::Str(i) };
        let raw_sed = 
            sed_str
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let sed = 
            just(Token::Sed)
            .ignore_then(
                raw_sed.
                delimited_by(just(Token::MbOpen), just(Token::MbClose))
                .map_with(|code,e| {
                    (Expr::Sed(Sed { code }), e.span()) 
                })
            );

        // sed_compiler用拡張 ここまで

        // return 
        let r#return = just(Token::Return)
            .ignore_then(
                expr_parser()
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>()
            )
            .then_ignore(just(Token::SemiColon))
            .map_with(|a, e| 
                (Expr::Return((a, e.span())) ,e.span())
            );

        choice((
            r#let,
            as_expr.clone()
            .then(decl)
            .map_with(|(lhs, rhs), e|
                (Expr::Then(Box::new(lhs), Box::new(rhs)), e.span())
            ),
            as_expr, // あとに何も続かない場合
            expr_parser(), // 式単体 返り値とするような場合
            sed, // sed命令文単体
            r#return
        ))
    })
}

pub fn lexer_parse(input: &str) -> (Option<Vec<(Token<'_>, SimpleSpan)>>, Vec<Rich<'_, char>>){
    lexer().parse(input).into_output_errors()
}

pub fn parser_parse<'a>(input: &str, tokens: &'a Vec<(Token<'a>, SimpleSpan)>) 
-> Result<Vec<(Func<'a>, SimpleSpan)>, Vec<Rich<'a, Token<'a>>>>
{
    func_parser().parse(
        tokens
            .as_slice()
            .map((input.len()..input.len()).into(), |(t, s)| (t, s))
    ).into_result()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne::{Color, Label, Report, ReportKind, Source};

    fn aaa() {
        use std::fs;
        let code = fs::read_to_string("example.soil")
            .expect("ファイルの読み込みに失敗しました");

        // fn map lst:list<T>, func:<fn T -> U> -> list<U>
        println!("input: {}", code);
        let (tokens, err) = lexer_parse(&code);

        if let Some(tokens) = tokens {
            println!("{:#?}", tokens);
            let parse_result = parser_parse(&code, &tokens);

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
                        .eprint(Source::from(&code))
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
        "hello world"
        "this is Tom"
    "#;
        println!("input: {}", input);
        let (tokens, err) = lexer().parse(input).into_output_errors();

        if let Some(tokens) = tokens {
            println!("{:#?}", tokens);
            let parse_result = decl_parser().parse(
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

    #[test]
    fn it_works00() {
        aaa()
    }

    #[test]
    fn it_works01() {
        bbb()
    }
}
