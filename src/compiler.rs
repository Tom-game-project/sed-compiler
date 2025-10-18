use std::{collections::HashMap};

use crate::code_gen::{SedInstruction};
use sed_compiler_frontend::parser::*;

/// 名前とインデックス(usize)を管理する構造体
#[derive(Debug)]
struct LocalNameRegistry {
    names: HashMap<String, usize>,
}

#[derive(Debug)]
struct LocalNameRegistryErr {
}

impl LocalNameRegistry {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    /// 名前を追加します。
    /// 成功した場合（新しい名前だった場合）、割り当てられたインデックスを Some(index) で返します。
    /// 失敗した場合（名前が重複していた場合）、None を返します。
    fn add_name(&mut self, name: &str) -> Option<usize> {
        use std::collections::hash_map::Entry;

        // 次に割り当てるインデックスは、現在の要素数
        let new_index = self.names.len();

        match self.names.entry(name.to_string()) {
            // Entry::Occupied は、キー（名前）がすでに存在することを意味します
            Entry::Occupied(_) => {
                // 名前が重複しているので None を返す
                None
            }
            // Entry::Vacant は、キーが存在しないことを意味します
            Entry::Vacant(entry) => {
                // 存在しないので、新しいインデックスを挿入します
                entry.insert(new_index);
                // 割り当てたインデックスを Some で返します
                Some(new_index)
            }
        }
    }

    /// 名前からインデックスを取得します
    fn get_index(&self, name: &str) -> Option<usize> {
        // get の結果は &usize なので、 copied() で usize に変換します
        self.names.get(name).copied()
    }

    fn merge(mut self, other: Self) -> Option<Self> {
        for (name, index) in other.names {
            self.add_name(&name);
        }
        Some(self)
    }
}


struct ArgNameRegistry {
    names: HashMap<String, usize>,
}

impl ArgNameRegistry {
    fn new() -> Self {
        Self {
            names: HashMap::new(),
        }
    }

    /// 名前を追加します。
    /// 成功した場合（新しい名前だった場合）、割り当てられたインデックスを Some(index) で返します。
    /// 失敗した場合（名前が重複していた場合）、None を返します。
    fn add_name(&mut self, name: &str) -> Option<usize> {
        use std::collections::hash_map::Entry;

        // 次に割り当てるインデックスは、現在の要素数
        let new_index = self.names.len();

        match self.names.entry(name.to_string()) {
            // Entry::Occupied は、キー（名前）がすでに存在することを意味します
            Entry::Occupied(_) => {
                // 名前が重複しているので None を返す
                None
            }
            // Entry::Vacant は、キーが存在しないことを意味します
            Entry::Vacant(entry) => {
                // 存在しないので、新しいインデックスを挿入します
                entry.insert(new_index);
                // 割り当てたインデックスを Some で返します
                Some(new_index)
            }
        }
    }

    /// 名前からインデックスを取得します
    fn get_index(&self, name: &str) -> Option<usize> {
        // get の結果は &usize なので、 copied() で usize に変換します
        self.names.get(name).copied()
    }

    fn merge(mut self, other: Self) -> Option<Self> {
        for (name, index) in other.names {
            self.add_name(&name);
        }
        Some(self)
    }
}

// 上 同じ実装が繰り返されてしまっているのでなんとかする


/// 名前をindexとして解決する
fn create_local_name_registry<'a>(expr: &Expr<'a>) -> Result<LocalNameRegistry, LocalNameRegistryErr>
{
    let mut name_reg = LocalNameRegistry::new();

    match expr {
        Expr::Then(a, b) => {
            if let Ok(a) = create_local_name_registry(&(*a).0) {
                if let Ok(b) = create_local_name_registry(&(*b).0) {
                    if let Some(r) = a.merge(b) {
                        name_reg = r;
                    } else {
                        return Err(LocalNameRegistryErr {  });
                    }
                } else {
                    return Err(LocalNameRegistryErr {  });
                }
            } else {
                return Err(LocalNameRegistryErr {  });
            }
        }
        Expr::If(_, a, b) => {
            if let Ok(a) = create_local_name_registry(&(*a).0) {
                if let Some(b_in) = &**b{
                    if let Ok(b) = create_local_name_registry(&b_in.0) {
                        if let Some(r) = a.merge(b) {
                            name_reg = r;
                        } else {
                            return Err(LocalNameRegistryErr {  });
                        }
                    } else {
                        return Err(LocalNameRegistryErr {  });
                    }
                } 
                else{ 
                    // else節がない時
                    name_reg = a;
                }
            } else {
                return Err(LocalNameRegistryErr {  });
            }
        }
        Expr::Let(a, _) => {
            name_reg.add_name(a);
        }
        _ => {
        }
    }
    Ok(name_reg)
}

fn build_ir<'a>(func: &Func<'a>) -> Vec<SedInstruction>
{
    match &func.body.0 {
        Expr::Error => {
            todo!()
        }
        Expr::If(cond, then, else_) => {
            
        }
        Expr::Then(a, b) => {
        }
        Expr::Let(a, b) => {
        }
        Expr::Sed(a) => {
        }
        Expr::Call(a, b) => {
        }
        Expr::Value(a) => {
        }
        Expr::Local(a) => {
        }
        Expr::Binary(a, op, b) => {
        }
        Expr::Neg(a) => {
        }

    }
    todo!()
}

/*
/// 
fn compiler_frontend(code: &str) -> Option<Vec<SedInstruction>>
{
    let (tokens, err) = lexer_parse(code);

    match tokens {
        Some(tokens) => {
            let parse_result = parser_parse(code, &tokens);
            match parse_result {
                Ok(a) => {
                    checking_syntax(a);
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
                        .eprint(Source::from(code))
                        .unwrap();
                    }
                    None
                }
            }

        }
        None => {
            println!("Some Error Occured");
            None
        }
    }
}
*/

#[cfg(test)]
mod compiler_test {
    use crate::compiler::create_local_name_registry;
    use sed_compiler_frontend::parser::*;
    use ariadne::{Color, Label, Report, ReportKind, Source};

    #[test]
    fn compiler_test00()
    {
        let code = r#"
pub fn mul a:bit32, b:bit32 -> bit32 {
    if is_empty(b) {
        let la = 0;
        0
    } else {
        let lb = 0;
        if ends_with_zero(b) {
            let lc = 0;
            mul(shift_left1(a), shift_right1(b))
        } else {
            let ld = 0;
            add(mul(shift_left1(a), shift_right1(b)))
        }
    }
}
"#;

        let (tokens, err) = lexer_parse(code);

        match tokens {
            Some(tokens) => {
                let parse_result = parser_parse(code, &tokens);
                match parse_result {
                    Ok(a) => {
                        for (func, _) in a {
                            let c = create_local_name_registry(&func.body.0);
                            println!("{:#?}", c.expect("failed to create_name_registry"));
                        }
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
                            .eprint(Source::from(code))
                            .unwrap();
                        }
                    }
                }

            }
            None => {
                println!("Some Error Occured");
            }
        }
    }
}
