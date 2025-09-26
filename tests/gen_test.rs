use sed_practice::code_gen::*;


pub fn gen_test_proc00() -> String{
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_pow = FuncDef::new("pow".to_string(), 2, 1, 1); 
    let mut func_add = FuncDef::new("add".to_string(), 2, 1, 1);

    let arg_vals: Vec<ArgVal> = vec![]; // entryの引数
    let local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ]; // entryのローカル変数

    entry.set_proc_contents(
        vec![
            SedInstruction::LocalVal(&local_vals[0]), // L0
            SedInstruction::ConstVal(ConstVal::new("2")),
            SedInstruction::Call(CallFunc::new("pow")),
            SedInstruction::LocalVal(&local_vals[1]), // L1
            SedInstruction::ConstVal(ConstVal::new("2")),
            SedInstruction::Call(CallFunc::new("pow")),
            SedInstruction::Call(CallFunc::new("add")),
        ]
    );

    // 関数の内容を定義する
    func_pow.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("# 関数の例".to_string())),
            SedInstruction::Sed(SedCode("s/:retlabel[0-9]\\+-\\([^\\-~]*\\)[^\\|]*|$/~\\1/".to_string())),

            SedInstruction::Sed(SedCode("s/\\n~\\(.*\\)/\\1funca/".to_string())),
            //SedInstruction::Call(
            //    CallFunc::new("func_b", "\\n\\(.*\\)", "-\\1-\\1")),
        ]
    );

    let mut func_table = vec![entry, func_pow];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        //println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }

}


/// let l0 = hello;
/// let l1 = Tom;
/// add("hello", add("world", "Tom"))
/// ---
/// local.get l0
/// const "world"
/// local.get l1
/// add
/// add
/// 
pub fn gen_test_proc01() -> String
{
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_add = FuncDef::new("add".to_string(), 2, 1, 1);

    let entry_arg_vals: Vec<ArgVal> = vec![]; // entryの引数
    let entry_local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ]; // entryのローカル変数

    entry.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/.*/~hello~Tom/".to_string())), //ローカル変数の初期化
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::ConstVal(ConstVal::new("world")),
            SedInstruction::LocalVal(&entry_local_vals[1]), // L1
            SedInstruction::Call(CallFunc::new("add")),
            SedInstruction::Call(CallFunc::new("add")),
        ]
    );

    let func_add_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
    ]; // entryの引数

    let func_add_local_vals:Vec<LocalVal> = vec![];
    // 関数の内容を定義する

    func_add.set_proc_contents(
        vec![ // 引数のセットが終わった状態からスタート
            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
        ]
    );

    let mut func_table = vec![entry, func_add];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        // println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}

/// 関数がさらに関数を呼び出すようなものの例
pub fn gen_test_proc02() -> String
{
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_add = FuncDef::new("add".to_string(), 2, 1, 1);
    let mut func_add3 =  FuncDef::new("add3".to_string(), 3, 0, 1);

    let entry_arg_vals: Vec<ArgVal> = vec![]; // entryの引数
    let entry_local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ]; // entryのローカル変数

    entry.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/.*/~hello~Tom/".to_string())), //ローカル変数の初期化
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::ConstVal(ConstVal::new("world")),
            SedInstruction::LocalVal(&entry_local_vals[1]), // L1
            SedInstruction::Call(CallFunc::new("add3"))
        ]
    );

    let func_add_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
    ]; // entryの引数

    let func_add_local_vals:Vec<LocalVal> = vec![];
    // 関数の内容を定義する

    func_add.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
        ]
    );

    let add3_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
        ArgVal::new(2),
    ]; // entryの引数

    func_add3.set_proc_contents(
        vec![
            SedInstruction::ArgVal(&add3_arg_vals[0]), // L0
            SedInstruction::ArgVal(&add3_arg_vals[1]),
            SedInstruction::ArgVal(&add3_arg_vals[2]), // L1
            SedInstruction::Call(CallFunc::new("add")),
            SedInstruction::Call(CallFunc::new("add")),
            SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())), // return処理
            //                            |<--------+------------>|
            //                                      +-- func_def.argc + func_def.localc
        ]
    );

    let mut func_table = vec![entry, func_add, func_add3];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        // println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}


/// 関数がさらに関数を呼び出すようなものの例
pub fn gen_test_proc03() -> String
{
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_strjoin = FuncDef::new("strjoin".to_string(), 2, 1, 1);
    let mut func_strjoin3 =  FuncDef::new("strjoin3".to_string(), 3, 0, 1);

    // ======================== func entry ========================

    let entry_arg_vals: Vec<ArgVal> = vec![]; // entryの引数
    let entry_local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ]; // entryのローカル変数

    let sed_values: Vec<SedValue> = vec![
        SedValue::LocalVal(&entry_local_vals[0]),
        SedValue::LocalVal(&entry_local_vals[1]),
    ];

    entry.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
            SedInstruction::ConstVal(ConstVal::new("hello")),
            SedInstruction::Set(&sed_values[0]),
            SedInstruction::ConstVal(ConstVal::new("Tom")),
            SedInstruction::Set(&sed_values[1]),
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::ConstVal(ConstVal::new("world")),
            SedInstruction::LocalVal(&entry_local_vals[1]), // L1
            SedInstruction::Call(CallFunc::new("strjoin3")),
            SedInstruction::Set(&sed_values[0]),
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::Set(&sed_values[1]),
        ]
    );

    // ======================== func add ========================
    let func_add_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
    ]; // entryの引数

    let func_add_local_vals:Vec<LocalVal> = vec![
        LocalVal::new(0)
    ];

    let sed_values_add = vec![
        SedValue::LocalVal(&func_add_local_vals[0]),
    ];
    // 関数の内容を定義する

    func_strjoin.set_proc_contents(
        vec![
            // 引数の受取部分
            // local変数は未初期化

            SedInstruction::ConstVal(ConstVal::new("hello hello hello hello")),
            SedInstruction::Set(&sed_values_add[0]), // 関数内のローカル変数に値を代入する

            //                             |<---------argc----------->|<---localc--->|
            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
        ]
    );

    // ======================== func add3 ========================
    let add3_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
        ArgVal::new(2),
    ]; // entryの引数

    func_strjoin3.set_proc_contents(
        vec![
            SedInstruction::ArgVal(&add3_arg_vals[0]), // L0
            SedInstruction::ArgVal(&add3_arg_vals[1]),
            SedInstruction::ArgVal(&add3_arg_vals[2]), // L1
            SedInstruction::Call(CallFunc::new("strjoin")),
            SedInstruction::Call(CallFunc::new("strjoin")),
            //SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())), // return処理
            //                            |<--------+------------>|
            //                                      +-- func_def.argc + func_deflocalc>
            SedInstruction::Ret,
        ]
    );

    let mut func_table = vec![entry, func_strjoin, func_strjoin3];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        // println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}

pub fn gen_test_proc04() -> String 
{
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_add = FuncDef::new("add".to_string(), 2, 0, 1);
    let mut func_shift1 = FuncDef::new("shift1".to_string(), 1, 0, 1);
    let mut func_add3 =  FuncDef::new("add3".to_string(), 3, 0, 1);

    // ======================== func shift1 ========================

    // ======================== func entry ========================

    let entry_arg_vals: Vec<ArgVal> = vec![]; // entryの引数
    let entry_local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ]; // entryのローカル変数

    let sed_values: Vec<SedValue> = vec![
        SedValue::LocalVal(&entry_local_vals[0]),
        SedValue::LocalVal(&entry_local_vals[1]),
    ];

    entry.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
            SedInstruction::ConstVal(ConstVal::new("101101110")),
            SedInstruction::Set(&sed_values[0]),
            SedInstruction::ConstVal(ConstVal::new("11101110111")),
            SedInstruction::Set(&sed_values[1]),
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::ConstVal(ConstVal::new("111")),
            SedInstruction::LocalVal(&entry_local_vals[1]), // L1
            SedInstruction::Call(CallFunc::new("add3")),
            SedInstruction::Set(&sed_values[0]),
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::Set(&sed_values[1]),
        ]
    );

    // ======================== func add ========================
    let func_add_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
    ]; // entryの引数

    let func_add_local_vals:Vec<LocalVal> = vec![
        LocalVal::new(0)
    ];

    let sed_values_add = vec![
        SedValue::LocalVal(&func_add_local_vals[0]),
    ];
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

    // ======================== func add3 ========================
    let add3_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
        ArgVal::new(1),
        ArgVal::new(2),
    ]; // entryの引数

    func_add3.set_proc_contents(
        vec![
            SedInstruction::ArgVal(&add3_arg_vals[0]), // L0
            SedInstruction::ArgVal(&add3_arg_vals[1]),
            SedInstruction::ArgVal(&add3_arg_vals[2]), // L1
            SedInstruction::Call(CallFunc::new("add")),
            SedInstruction::Call(CallFunc::new("add")),
            SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())), // return処理
            //                            |<--------+------------>|
            //                                      +-- func_def.argc + func_deflocalc>
        ]
    );

    // ===========================================================

    let mut func_table = vec![entry, func_add, func_add3];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        // println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}

/// if分岐テスト
pub fn gen_test_proc05() -> String 
{
    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
    let mut func_if_test = FuncDef::new("if_test".to_string(), 1, 1, 1);

    // ======================== func entry ========================

    let entry_arg_vals: Vec<ArgVal> = vec![]; // entryの引数
    let entry_local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ]; // entryのローカル変数

    let sed_values: Vec<SedValue> = vec![
        SedValue::LocalVal(&entry_local_vals[0]),
        SedValue::LocalVal(&entry_local_vals[1]),
    ];

    entry.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
            SedInstruction::ConstVal(ConstVal::new("101101110")),
            SedInstruction::Set(&sed_values[0]),
            SedInstruction::ConstVal(ConstVal::new("11101110111")),
            SedInstruction::Set(&sed_values[1]),
            SedInstruction::LocalVal(&entry_local_vals[0]), // L0
            SedInstruction::Call(CallFunc::new("if_test")),
            SedInstruction::Set(&sed_values[0])
        ]
    );

    // ======================== func add ========================
    let func_if_test_arg_vals: Vec<ArgVal> = vec![
        ArgVal::new(0),
    ]; // entryの引数

    let func_if_test_local_vals:Vec<LocalVal> = vec![
        LocalVal::new(0)
    ];

    let func_if_test_sed_values: Vec<SedValue> = vec![
        SedValue::LocalVal(&func_if_test_local_vals[0]),
    ];

    // 関数の内容を定義する

    func_if_test.set_proc_contents(
        vec![
            //SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)/\\1/".to_string())),
            SedInstruction::ConstVal(ConstVal::new("1")), // else節に入る
            //SedInstruction::ConstVal(ConstVal::new("0")),
            SedInstruction::IfProc(IfProc::new(
                    vec![
                        SedInstruction::ConstVal(ConstVal::new("Hello")),
                        SedInstruction::Set(&func_if_test_sed_values[0]),
                    ],
                    vec![
                        SedInstruction::ConstVal(ConstVal::new("World")),
                        SedInstruction::Set(&func_if_test_sed_values[0]),
                    ]
            )),
            SedInstruction::LocalVal(&func_if_test_local_vals[0]),
            SedInstruction::Ret,
        ]
    );

    let mut func_table = vec![entry, func_if_test];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        // println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}

#[cfg(test)]
mod gen_test {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn gen_test00() {
        let mut file = File::create("./sed/labo6.sed")
            .expect("ファイルが開けませんでした");  
        let a = gen_test_proc00();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test01() {
        let mut file = File::create("./sed/labo6.sed")
            .expect("ファイルが開けませんでした");  
        let a = gen_test_proc01();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test02() {
        let mut file = File::create("./sed/labo6.sed")
            .expect("ファイルが開けませんでした");  
        let a = gen_test_proc02();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test03() {
        let mut file = File::create("./sed/strjoin.sed")
            .expect("ファイルが開けませんでした");  
        let a = gen_test_proc03();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test04() {
        let mut file = File::create("./sed/num_add.sed")
            .expect("ファイルが開けませんでした");  
        let a = gen_test_proc04();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");
    }

    #[test]
    fn gen_test05() {
        let mut file = File::create("./sed/if_example.sed")
            .expect("ファイルが開けませんでした");  
        let a = gen_test_proc05();
        file.write_all(a.as_bytes())
            .expect("書き込みに失敗しました");

    }
}
