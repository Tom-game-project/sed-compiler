use chumsky::prelude::*;

#[derive(Clone, Debug)]
pub enum Stmt <'src>{
    Expr,
    If(&'src str, Vec<Stmt<'src>>),
    Loop(Vec<Stmt<'src>>),
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<Stmt<'a>>> {
    let expr = just("expr"); // TODO

    let block = recursive(|block| {
        let indent = just(' ')
            .repeated()
            .configure(|cfg, parent_indent| cfg.exactly(*parent_indent));

        let expr_stmt = expr.then_ignore(text::newline()).to(Stmt::Expr);
        let control_flow = just("def:")
            .then(text::newline())
            .ignore_then(block)
            .map(Stmt::Loop);

        //let if_stmt = ;
        let stmt = expr_stmt.or(control_flow);

        text::whitespace()
            .count()
            .ignore_with_ctx(stmt.separated_by(indent).collect())
    });

    block.with_ctx(0)
}

fn aaa() {
    let stmts = parser().padded().parse(
        r#"
expr
expr
def:
    expr
    def:
        expr
        expr
    expr
expr
"#,
    );
    println!("{:#?}", stmts.output());
    println!("{:?}", stmts.errors().collect::<Vec<_>>());
}


#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works00() {
    //     let result = add(2, 2);
    //     assert_eq!(result, 4);
    // }

    #[test]
    fn it_works01() {
        aaa()
    }
}
