#[cfg(test)]
mod test00 {
    use sed_compiler_frontend::parser::{
        lexer_parse,
        func_parser,
    };
    use chumsky::prelude::*;
    use ariadne::{Color, Label, Report, ReportKind, Source};

    #[test]
    fn test00_aaa (){
        let input = r#"
pub fn name_hello a:num, b:num, c:string -> bool {
    let a = 42 + 1 + hello;
    let b = a * 2;
    a = a + b;
    let b = a * 2;
    b = c + 1;
}

fn name_world a:num, b:num, c:string -> bool {
    let a = 42 + 1 + hello;
    let b = (a + 1) * 2;
    a = a + b;
    let b = a * 2;
    if a == 1 {
        let b = a * 2;
        b
    } else {
        let b = a * 2;
        b + 1
    }
    b = c + 1
}

pub fn mul a:bit32, b:bit32 -> bit32 {
    if is_empty(b) {
        bit32(0)
    } else {
        if ends_with_zero(b) {
            mul(shift_left1(a), shift_right1(b))
        } else {
            add(mul(shift_left1(a), shift_right1(b)))
        }
    }
}
"#;
        // fn map lst:list<T>, func:<fn T -> U> -> list<U>
        println!("input: {}", input);
        let (tokens, err) = lexer_parse(input);

        if let Some(tokens) = tokens {
            // println!("{:#?}", tokens);
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

    fn test00_bbb() {
        let input = r#"
        "hello world"
"#;
        // fn map lst:list<T>, func:<fn T -> U> -> list<U>
        println!("input: {}", input);
        let (tokens, err) = lexer_parse(input);

        if let Some(tokens) = tokens {
            // println!("{:#?}", tokens);
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
}
