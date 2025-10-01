use sed_practice::code_gen::*;

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
        let mut file = File::create("./sed/mul.sed").expect("ファイルが開けませんでした");
        let a = gen_test_proc01();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }
}

fn gen_test_proc00() -> String {
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_mul = FuncDef::new("mul".to_string(), 2, 1, 1);
    let mut func_add = FuncDef::new("add".to_string(), 2, 0, 1);
    let mut func_shift_left1 = FuncDef::new("shift_left1".to_string(), 1, 0, 1);
    let mut func_shift_right1 = FuncDef::new("shift_right1".to_string(), 1, 0, 1);
    let mut func_is_empty = FuncDef::new("is_empty".to_string(), 1, 0, 1);
    let mut func_ends_with_zero = FuncDef::new("ends_with_zero".to_string(), 1, 0, 1);

    // ======================== func entry ========================

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
    ]);

    // ======================== func ends_with_zero ========================
    func_ends_with_zero.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("s/.*0$/~1;/ ".to_string())),
        SedInstruction::Sed(SedCode("s/.*1$/~0;/ ".to_string())),
    ]);
    // ======================== func is_empty ========================
    func_is_empty.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("s/~$/T/   ".to_string())),
        SedInstruction::Sed(SedCode("s/~.*$/F/ ".to_string())),
        SedInstruction::Sed(SedCode("s/T/~1;/  ".to_string())),
        SedInstruction::Sed(SedCode("s/F/~0;/  ".to_string())),
    ]);
    // ======================== func shift_left1 ========================
    func_shift_left1.set_proc_contents(vec![SedInstruction::Sed(SedCode(
        "s/\\(~[01]*\\)/\\10;/".to_string(),
    ))]);
    // ======================== func shift_right1 ========================
    func_shift_right1.set_proc_contents(vec![SedInstruction::Sed(SedCode(
        "s/\\(~[01]*\\)[01]/\\1;/".to_string(),
    ))]);
    // ======================== func add ========================
    // 関数の内容を定義する

    func_add.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("# 入力をaddloopの形式に変換".to_string())),
            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/add 0;;\\1;\\2;/".to_string())),
            SedInstruction::Sed(SedCode("b addloop".to_string())),
            SedInstruction::Sed(SedCode(":addloop".to_string())),
            SedInstruction::Sed(SedCode("s/add 1;\\([01]*\\);;;/1\\1/".to_string())),
            SedInstruction::Sed(SedCode("s/add 0;\\([01]*\\);;;/\\1/".to_string())),
            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);\\([01]*\\);;/add \\1;\\2;\\3;0;/".to_string())),
            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);;\\([01]*\\);/add \\1;\\2;0;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);\\([01]*\\)\\([01]\\);\\([01]*\\)\\([01]\\);/add \\1\\4\\6;\\2;\\3;\\5;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 000;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;0\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 001;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 010;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 011;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 100;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 101;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 110;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("s/add 111;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;1\\1;\\2;\\3;/".to_string())),
            SedInstruction::Sed(SedCode("t addloop".to_string())),
            SedInstruction::Sed(SedCode("s/\\(.*\\)/~\\1;/".to_string())),
        ]
    );

    // ======================== func mul ========================
    func_mul.set_proc_contents(vec![
        SedInstruction::Val(Value::Arg(1)),
        SedInstruction::Call(CallFunc::new("is_empty")),
        SedInstruction::IfProc(IfProc::new(
            vec![
                SedInstruction::ConstVal(ConstVal::new("0")),
                SedInstruction::Set(Value::Local(0)), // rstr
            ],
            vec![
                SedInstruction::Val(Value::Arg(1)),
                SedInstruction::Call(CallFunc::new("ends_with_zero")),
                SedInstruction::IfProc(IfProc::new(
                    vec![
                        // rstr = mul(shift_left1(a), shift_right1(b))
                        SedInstruction::Val(Value::Arg(0)), // a
                        SedInstruction::Call(CallFunc::new("shift_left1")),
                        SedInstruction::Val(Value::Arg(1)), // b
                        SedInstruction::Call(CallFunc::new("shift_right1")),
                        SedInstruction::Call(CallFunc::new("mul")),
                        SedInstruction::Set(Value::Local(0)), // rstr
                    ],
                    vec![
                        // rstr = add(a, mul(shift_left1(a), shift_right1(b)))
                        SedInstruction::Val(Value::Arg(0)), // a
                        SedInstruction::Call(CallFunc::new("shift_left1")),
                        SedInstruction::Val(Value::Arg(1)), // b
                        SedInstruction::Call(CallFunc::new("shift_right1")),
                        SedInstruction::Call(CallFunc::new("mul")),
                        SedInstruction::Val(Value::Arg(0)), // a
                        SedInstruction::Call(CallFunc::new("add")),
                        SedInstruction::Set(Value::Local(0)), // rstr
                    ],
                )),
            ],
        )),
        // return rstr;
        SedInstruction::Val(Value::Local(0)),
        SedInstruction::Ret,
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
    use sed_practice::embedded::*;
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
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
