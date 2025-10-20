use sed_compiler::{code_gen::*, embedded::{em_add, em_ends_with_zero, em_mul, em_sub32, em_twos_complement, em_zero_padding32}};

#[cfg(test)]
mod gen_test2 {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn gen_test00() {
        let mut file = File::create("./sed/mul.sed").expect("ファイルが開けませんでした");
        let a = gen_test_proc00();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test01() {
        let mut file = File::create("./sed/sub.sed").expect("ファイルが開けませんでした");
        let a = gen_test_proc01();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test02() {
        let mut file = File::create("./sed/sub.sed").expect("ファイルが開けませんでした");
        let a = gen_test_proc02();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }
}

fn gen_test_proc00() -> String {
    use sed_compiler::embedded::*;
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry", 0, 2, 1);
    let func_mul = em_mul();
    let func_add = em_add();
    let func_shift_left1 = em_shift_left1();
    let func_shift_right1 = em_shift_right1();
    let func_is_empty = em_is_empty();
    let func_ends_with_zero = em_ends_with_zero();

    entry.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
        SedInstruction::ConstVal(ConstVal::new("101101110")),
        SedInstruction::Set(Value::Local(0)),
        SedInstruction::ConstVal(ConstVal::new("11101110111")),
        SedInstruction::Set(Value::Local(1)),
        SedInstruction::Val(Value::Local(0)), // L0
        SedInstruction::Val(Value::Local(1)), // L0
        SedInstruction::Call(CallFunc::new("mul")),
        SedInstruction::Set(Value::Local(0)),
        SedInstruction::Val(Value::Local(0)),
        SedInstruction::Ret
    ]);

    let compile_result = CompilerBuilder::new()
        .add_func(entry)
        .add_func(func_mul)
        .add_func(func_add)
        .add_func(func_is_empty)
        .add_func(func_shift_left1)
        .add_func(func_shift_right1)
        .add_func(func_ends_with_zero)
        .assemble()
        .generate();

    match compile_result {
        Ok(code) => code,
        Err(e) => {
            println!("{:?}", e);
            "".to_string()
        }
    }
}

fn gen_test_proc01() -> String { 
    let mut entry = FuncDef::new("entry", 0, 2, 1);
    let twos_complement = em_twos_complement();
    let func_add = em_add();
    let func_zero_padding32 = em_zero_padding32();
    let mut func_sub32 = em_sub32();

    entry.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
        SedInstruction::ConstVal(ConstVal::new("10011011010111111")),
        SedInstruction::Set(Value::Local(0)),
        SedInstruction::ConstVal(ConstVal::new("11101101")),
        SedInstruction::Set(Value::Local(1)),
        SedInstruction::Val(Value::Local(0)), // L0
        SedInstruction::Val(Value::Local(1)), // L0
        SedInstruction::Call(CallFunc::new("sub32")),
        SedInstruction::Ret,
    ]);

    func_sub32.set_proc_contents(vec![
        SedInstruction::Val(Value::Arg(0)),
        SedInstruction::Call(CallFunc::new("zero_padding32")),
        SedInstruction::Val(Value::Arg(1)),
        SedInstruction::Call(CallFunc::new("zero_padding32")),
        SedInstruction::Call(CallFunc::new("twos_complement")),
        SedInstruction::Call(CallFunc::new("add")),
        SedInstruction::Call(CallFunc::new("zero_padding32")),
        SedInstruction::Ret,
    ]);

    let compile_result = CompilerBuilder::new()
        .add_func(entry)
        .add_func(func_add)
        .add_func(twos_complement)
        .add_func(func_zero_padding32)
        .add_func(func_sub32)
        .assemble()
        .generate();

    match compile_result {
        Ok(code) => code,
        Err(e) => {
            println!("{:?}", e);
            "".to_string()
        }
    }
}

fn gen_test_proc02() -> String {
    let mut entry = FuncDef::new("entry", 0, 2, 1);


    let compile_result = CompilerBuilder::new()
        .add_func(entry)
        .assemble().generate();

    match compile_result {
        Ok(code) => code,
        Err(e) => {
            println!("{:?}", e);
            "".to_string()
        }
    }
}
