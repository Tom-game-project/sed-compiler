use std::{
    collections::HashSet,
    marker::PhantomData,
    vec
};

use crate::code_gen::{self, CallFunc, CompilerBuilder, ConstVal, FuncDef, IfProc, SedCode, SedInstruction};
use sed_compiler_frontend::parser::*;

use ariadne::{Color, Label, Report, ReportKind, Source};

#[derive(Debug)]
struct TypeArg;
#[derive(Debug)]
struct TypeLocal;

/// 名前とインデックス(usize)を管理する構造体
#[derive(Debug)]
struct NameRegistry<T> {
    names: Vec<String>,
    _seen: HashSet<String>,
    _arg_or_local: PhantomData<T>
}

#[derive(Debug)]
struct NameRegistryErr { 
}

impl<T> NameRegistry <T>{
    fn new() -> Self {
        Self {
            names: Vec::new(),
            _seen: HashSet::new(),
            _arg_or_local: PhantomData
        }
    }

    /// 名前を追加します。
    /// 成功した場合（新しい名前だった場合）、割り当てられたインデックスを Some(index) で返します。
    /// 失敗した場合（名前が重複していた場合）、None を返します。
    fn add_name(&mut self, name: &str) -> Option<usize> {
        // 次に割り当てるインデックスは、現在の要素数
        let r = self.names.len();
        if self._seen.insert(name.to_string()) {
            self.names.push(name.to_string());
        }
        else {
            return None;
        }
        Some(r)
    }

    /// 名前からインデックスを取得します
    fn get_index(&self, name: &str) -> Option<usize> {
        self.names.iter().position(|x| x == name)
    }

    fn merge(mut self, other: Self) -> Option<Self> {
        for name in other.names {
            self.add_name(&name);
        }
        Some(self)
    }
}

/// 名前をindexとして解決する
fn create_local_name_registry<'a>(expr: &Expr<'a>) -> Result<NameRegistry<TypeLocal>, NameRegistryErr>
{
    let mut name_reg = NameRegistry::new();

    match expr {
        Expr::Then(a, b) => {
            if let Ok(a) = create_local_name_registry(&(*a).0) {
                if let Ok(b) = create_local_name_registry(&(*b).0) {
                    if let Some(r) = a.merge(b) {
                        name_reg = r;
                    } else {
                        return Err(NameRegistryErr {  });
                    }
                } else {
                    return Err(NameRegistryErr {  });
                }
            } else {
                return Err(NameRegistryErr {  });
            }
        }
        Expr::If(_, a, b) => {
            if let Ok(a) = create_local_name_registry(&(*a).0) {
                if let Some(b_in) = &**b{
                    if let Ok(b) = create_local_name_registry(&b_in.0) {
                        if let Some(r) = a.merge(b) {
                            name_reg = r;
                        } else {
                            return Err(NameRegistryErr {  });
                        }
                    } else {
                        return Err(NameRegistryErr {  });
                    }
                } 
                else{ 
                    // else節がない時
                    name_reg = a;
                }
            } else {
                return Err(NameRegistryErr {  });
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

fn create_arg_name_registry<'a>(func: &Func<'a>) -> Result<NameRegistry<TypeArg>, NameRegistryErr>
{
    let mut name_reg = NameRegistry::new();
    for (arg, _) in &func.args {
        let r = name_reg.add_name(arg.name);
        if r.is_none() {
            return Err(NameRegistryErr { });
        }
    }
    Ok(name_reg)
}

fn build_func_ir<'a>(func: &Func<'a>) -> Result<FuncDef, BuildIRErr>
{
    let local_name_registry = if let Ok(a) = create_local_name_registry(&func.body.0){
        a
    }else {
        return Err(BuildIRErr { note: "failed to create local_name_registry".to_string() });
    };
    let arg_name_registry = if let Ok(a) = create_arg_name_registry(&func) {
        a
    }else {
        return Err(BuildIRErr { note: "failed to create arg_name_registry".to_string() });
    };
    let mut func_def = FuncDef::new(
        func.name,
        func.args.len(), 
        local_name_registry.names.len(),
        func.rtype.len());

    func_def.set_proc_contents(build_ir(&func.body.0, &arg_name_registry, &local_name_registry)?);
    Ok(func_def)
}

fn find_value_from_name_registry(
    arg_name_registry: &NameRegistry<TypeArg>,
    local_name_registry: &NameRegistry<TypeLocal>,
    name: &str
) -> Option<code_gen::Value>
{
    if let Some(index) = arg_name_registry.get_index(name) {
        Some(code_gen::Value::Arg(index))
    }
    else if let Some (index) = local_name_registry.get_index(name) {
        Some(code_gen::Value::Local(index))
    }
    else {
        None
    }
}

#[derive(Clone, Debug)]
struct BuildIRErr{
    note: String
}

fn build_ir<'a>(
    expr: &Expr<'a>,
    arg_name_registry: &NameRegistry<TypeArg>,
    local_name_registry: &NameRegistry<TypeLocal>
) -> Result<Vec<SedInstruction>, BuildIRErr>
{
    match &expr {
        Expr::Error => {
            return Err(BuildIRErr { note: "ast contains \"Error\"".to_string() })
        }
        Expr::If(cond, then, else_) => {
            let mut cond_ir =  build_ir(&(*cond).0, arg_name_registry, local_name_registry)?;
            let if_inst = SedInstruction::IfProc(
                IfProc::new(
                    build_ir(&(*then).0, &arg_name_registry, &local_name_registry)?,
                    if let Some((else_, span)) = &**else_ {
                        build_ir(else_, arg_name_registry, local_name_registry)?
                    }
                    else {
                        vec![]
                    }
            ));
            cond_ir.push(if_inst);
            return Ok(cond_ir);
        }
        Expr::Then(a, b) => {
            let mut a_ir = build_ir(&(*a).0, &arg_name_registry, &local_name_registry)?;
            let mut b_ir = build_ir(&(*b).0, &arg_name_registry, &local_name_registry)?;
            a_ir.append(&mut b_ir);
            return Ok(a_ir);
        }
        Expr::Let(a, b) => {
            if let Some(val_number) = find_value_from_name_registry(
                &arg_name_registry, 
                &local_name_registry, a) {
                let mut ir = build_ir(&(*b).0, &arg_name_registry, &local_name_registry)?;
                ir.push(
                    SedInstruction::Set(val_number)
                );
                return Ok(ir);
            } else {
                // error
                return Err(BuildIRErr { note: format!("could not find value \"{}\" from the registry.", a)})
            }
        }
        Expr::Sed(a) => {
            // 単純にプログラムを展開する
            let mut r_inst = vec![];
            for i in &a.code{
                if let Value::Str(sed_code) = i {
                    r_inst.push(SedInstruction::Sed(SedCode(sed_code.to_string())));
                } else {
                    // error
                }
            }
            return Ok(r_inst);
        }
        Expr::Call(a, b) => {
            let mut instructions = vec![];
            for (expr, _) in &(*b).0 {
                let mut inst = build_ir(&expr, &arg_name_registry, &local_name_registry)?;
                instructions.append(
                    &mut inst
                );
            }
            if let Expr::Local(name) = &(*a).0 {
                let call_name =  SedInstruction::Call(CallFunc::new(name));
                instructions.push(call_name);
            }
            else {
                return Err(BuildIRErr { note: "function name must be local".to_string() })
            }
            return Ok(instructions);
        }
        Expr::Value(a) => {
            let data_inst = match &a{
                Value::Null => {
                    SedInstruction::ConstVal(ConstVal::new("0"))
                }
                Value::Bool(b) => {
                    if *b {
                        SedInstruction::ConstVal(ConstVal::new("1"))
                    }
                    else {
                        SedInstruction::ConstVal(ConstVal::new("0"))
                    }
                }
                Value::Str(data) => {
                    SedInstruction::ConstVal(ConstVal::new(data))
                }
                Value::Func(name) => {
                    SedInstruction::Call(CallFunc::new(name))
                }
                Value::Int32(i) => {
                    SedInstruction::ConstVal(ConstVal::new(&format!("{:032b}", i)))
                }
                Value::Int64(i) => {
                    SedInstruction::ConstVal(ConstVal::new(&format!("{:064b}", i)))
                }
            };
            return Ok(vec![data_inst]);
        }
        Expr::Local(a) => {
            if let Some(val_number) = find_value_from_name_registry(
                &arg_name_registry, 
                &local_name_registry, a) {
                return Ok(vec! [SedInstruction::Val(val_number)]);
            } else {
                // localでかつこれに変数名引数名に該当しない場合は関数
                return Err(BuildIRErr { note: "context error".to_string() })
            }
        }
        Expr::Binary(lhs, op, rhs) => {
            match &op {
                BinaryOp::Add 
                    | BinaryOp::Sub
                    | BinaryOp::Mul 
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::NotEq 
                    | BinaryOp::Eq
                    => {
                    let mut lhs = build_ir(&(*lhs).0, &arg_name_registry, &local_name_registry)?;
                    let mut rhs = build_ir(&(*rhs).0, &arg_name_registry, &local_name_registry)?;
                    lhs.append(&mut rhs);
                    lhs.push(SedInstruction::Call(CallFunc::new(op_func_table(&op))));
                    return Ok(lhs);
                }
                //BinaryOp::Assign => {
                //    // 重要
                //    // 左の式は必ず、localでなければならない
                //    // 代入
                //    let mut rhs = build_ir(&(*rhs).0, &arg_name_registry, &local_name_registry)?;
                //    if let Expr::Local(a) = &(*lhs).0 {
                //        if let Some(name) = find_value_from_name_registry(&arg_name_registry, &local_name_registry, a) {
                //            rhs.push(SedInstruction::Set(name));
                //            return Ok(rhs);
                //        }
                //        else {
                //            return Err(BuildIRErr { note: "could not find value from the registry." })
                //        }
                //    }
                //    else {
                //        return Err(BuildIRErr { note: "invalid left expresion" });
                //    }
                //}
            }
        }
        Expr::Neg(a) => {
            return Err(BuildIRErr { note: "not yet".to_string() })
        }
        Expr::Return((a, span)) => {
            // 返り値の型が違うエラー
            let mut ir = vec![];
            for (expr, _span) in a {
                ir.append(&mut build_ir(
                        &expr,
                        &arg_name_registry,
                        &local_name_registry)?);
            }
            ir.push(SedInstruction::Ret);
            return Ok(ir);
        }
        Expr::Assign(lhs, rhs) => {
            let mut rhs_ir = build_ir(&(*rhs).0, &arg_name_registry, &local_name_registry)?;

            for (value, value_span) in (*lhs).0.iter().rev() {
                if let Expr::Local(a) = &value {
                        if let Some(name) = find_value_from_name_registry(&arg_name_registry, &local_name_registry, a) {
                            rhs_ir.push(SedInstruction::Set(name));
                        }
                        else {
                            return Err(BuildIRErr { note: format!("could not find value \"{}\" from the registry.", a)})
                        }
                } else {
                    // unreachable
                    return Err(BuildIRErr { note: "invalid left expresion".to_string() })
                }
            }
            return Ok(rhs_ir);
        }
    }
}

fn op_func_table<'a>(binop: &BinaryOp)-> &'a str
{
    // 組み込み関数の名前にする
    match binop {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "sub32",
        BinaryOp::Mul => "mul32",
        BinaryOp::Div => "div",
        BinaryOp::Eq => "eq",
        BinaryOp::NotEq => "neq",
        BinaryOp::Mod => "mod",
    }
}


/// 
pub fn compiler_frontend(code: &str) 
    -> Result<CompilerBuilder<code_gen::Unassembled>, BuildIRErr>
{
        let (tokens, err) = lexer_parse(code);
        let mut compile_builder = CompilerBuilder::new();

        match &tokens {
            Some(tokens) => {
                let parse_result = parser_parse(code, &tokens);
                println!("{:#?}", parse_result);
                match parse_result {
                    Ok(a) => {
                        for (func, _) in a {
                                match build_func_ir(&func) {
                                    Ok(instructions) => {
                                        println!("{:#?}", instructions);
                                        compile_builder = compile_builder.add_func(
                                            instructions
                                        )
                                    }
                                    Err(e) => {
                                        return Err(BuildIRErr{ note: format!("{}", e.note) });
                                    }
                                }
                            }
                        return Ok(compile_builder);
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
                        return Err(BuildIRErr { note: "failed while parsing".to_string() });
                    }
                }

            }
            None => {
                println!("Some Error Occured");

                return Err(BuildIRErr{ note: "failed while tokenize".to_string()});
            }
        }

}

#[cfg(test)]
mod compiler_test {
    use crate::compiler::{build_func_ir, create_arg_name_registry, create_local_name_registry};
    use sed_compiler_frontend::parser::*;
    use ariadne::{Color, Label, Report, ReportKind, Source};

    use super::{compiler_frontend};

    #[test]
    fn compiler_test00()
    {
        let code = r#"
pub fn mul a:bit32, b:bit32 -> bit32, bit32 {
    let r = 0;
    if is_empty(b) {
        r = 0;
    } else {
        if ends_with_zero(b) {
            r = mul(shift_left1(a), shift_right1(b));
        } else {
            r = add(a, mul(shift_left1(a), shift_right1(b)));
        }
    }
    return r;
}
"#;

        let (tokens, err) = lexer_parse(code);

        match tokens {
            Some(tokens) => {
                let parse_result = parser_parse(code, &tokens);
                println!("{:#?}", parse_result);
                match parse_result {
                    Ok(a) => {
                        for (func, _) in a {
                            let locals_name_dir = create_local_name_registry(&func.body.0);
                            let args_name_dir = create_arg_name_registry(&func);
                            println!("{:#?}\n{:#?}", locals_name_dir.expect("failed to create_name_registry"), args_name_dir);
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
    #[test]
    fn compiler_test01() {
        let code = r#"
pub fn mul a:bit32, b:bit32 -> bit32, bit32 {
    let r = 0;
    if is_empty(b) {
        r = 0;
    } else {
        if ends_with_zero(b) {
            r = mul(shift_left1(a), shift_right1(b));
        } else {
            r = add(mul(shift_left1(a), shift_right1(b)));
        }
    }
    return r;
}
"#;

        let (tokens, err) = lexer_parse(code);

        match tokens {
            Some(tokens) => {
                let parse_result = parser_parse(code, &tokens);
                println!("{:#?}", parse_result);
                match parse_result {
                    Ok(a) => {
                        for (func, _) in a {
                                let locals_name_dir = create_local_name_registry(&func.body.0).expect("ローカル変数の構成に失敗");
                                let args_name_dir = create_arg_name_registry(&func).expect("引数の構成に失敗");

                                match build_func_ir(&func) {
                                    Ok(instructions) => {
                                        println!("{:#?}", instructions);
                                    }
                                    Err(err) => {
                                        println!("{}", err.note);
                                    }
                                }
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

    #[test]
    fn compiler_test02() {
        use std::fs;
        let code = fs::read_to_string("soil/example.soil").expect("ファイルの読み込みに失敗しました");
        println!("start compiler_test02...");
        match compiler_frontend(&code){
            Ok(compiler_builder) => {
                let generated = compiler_builder
                    .assemble()
                    .generate();

                match generated {
                    Ok(generated_sed_code) => {
                        //let mut edi = 
                        //edi.push_str(&generated_sed_code);
                        fs::write("sed/c_example.sed", generated_sed_code).expect("書き込みに失敗しました");
                        // println!("{}", generated_sed_code);
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }
            }
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}
