//use sed_practice::code_gen::*;
//
//#[cfg(test)]
//mod gen_test {
//    use super::*;
//    use std::fs::File;
//    use std::io::Write;
//
//    #[test]
//    fn gen_test00() {
//        let mut file = File::create("./sed/labo6.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc00();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//    }
//
//    #[test]
//    fn gen_test01() {
//        let mut file = File::create("./sed/labo6.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc01();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//    }
//
//    #[test]
//    fn gen_test02() {
//        let mut file = File::create("./sed/labo6.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc02();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//    }
//
//    #[test]
//    fn gen_test03() {
//        let mut file = File::create("./sed/strjoin.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc03();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//    }
//
//    #[test]
//    fn gen_test04() {
//        let mut file = File::create("./sed/num_add.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc04();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//    }
//
//    #[test]
//    fn gen_test05() {
//        let mut file = File::create("./sed/if_example.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc05();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//
//    }
//
//    #[test]
//    fn gen_test06() {
//        let mut file = File::create("./sed/mul.sed")
//            .expect("ファイルが開けませんでした");  
//        let a = gen_test_proc06();
//        file.write_all(a.as_bytes())
//            .expect("書き込みに失敗しました");
//
//    }
//}
//
//pub fn gen_test_proc00() -> String {
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_pow = FuncDef::new("pow".to_string(), 2, 1, 1); 
//    let mut func_add = FuncDef::new("add".to_string(), 2, 1, 1);
//
//    let local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::ConstVal(ConstVal::new("2")),
//            SedInstruction::Call(CallFunc::new("pow")),
//            SedInstruction::LocalVal(1), // L1
//            SedInstruction::ConstVal(ConstVal::new("2")),
//            SedInstruction::Call(CallFunc::new("pow")),
//            SedInstruction::Call(CallFunc::new("add")),
//        ]
//    );
//
//    // 関数の内容を定義する
//    func_pow.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("# 関数の例".to_string())),
//            SedInstruction::Sed(SedCode("s/:retlabel[0-9]\\+-\\([^\\-~]*\\)[^\\|]*|$/~\\1/".to_string())),
//
//            SedInstruction::Sed(SedCode("s/\\n~\\(.*\\)/\\1funca/".to_string())),
//            //SedInstruction::Call(
//            //    CallFunc::new("func_b", "\\n\\(.*\\)", "-\\1-\\1")),
//        ]
//    );
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_pow)
//        .add_func(func_add)
//        .assemble()
//        .generate();
//    if let Ok(code) = compile_result {
//        //println!("{}", code);
//        code
//    }
//    else
//    {
//        println!("Compile err occured");
//        "".to_string()
//    }
//
//}
//
//
///// let l0 = hello;
///// let l1 = Tom;
///// add("hello", add("world", "Tom"))
///// ---
///// local.get l0
///// const "world"
///// local.get l1
///// add
///// add
///// 
//pub fn gen_test_proc01() -> String
//{
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_add = FuncDef::new("add".to_string(), 2, 1, 1);
//
//    let entry_local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/.*/~hello~Tom/".to_string())), //ローカル変数の初期化
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::ConstVal(ConstVal::new("world")),
//            SedInstruction::LocalVal(1), // L1
//            SedInstruction::Call(CallFunc::new("add")),
//            SedInstruction::Call(CallFunc::new("add")),
//        ]
//    );
//    // 関数の内容を定義する
//
//    func_add.set_proc_contents(
//        vec![ // 引数のセットが終わった状態からスタート
//            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
//        ]
//    );
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_add)
//        .assemble()
//        .generate();
//    if let Ok(code) = compile_result {
//        // println!("{}", code);
//        code
//    }
//    else
//    {
//        println!("Compile err occured");
//        "".to_string()
//    }
//}
//
///// 関数がさらに関数を呼び出すようなものの例
//pub fn gen_test_proc02() -> String
//{
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_add = FuncDef::new("add".to_string(), 2, 1, 1);
//    let mut func_add3 =  FuncDef::new("add3".to_string(), 3, 0, 1);
//
//    let entry_local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/.*/~hello~Tom/".to_string())), //ローカル変数の初期化
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::ConstVal(ConstVal::new("world")),
//            SedInstruction::LocalVal(1), // L1
//            SedInstruction::Call(CallFunc::new("add3"))
//        ]
//    );
//
//    func_add.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
//        ]
//    );
//
//    let add3_arg_vals: Vec<ArgVal> = vec![
//        ArgVal::new(0),
//        ArgVal::new(1),
//        ArgVal::new(2),
//    ]; // entryの引数
//
//    func_add3.set_proc_contents(
//        vec![
//            SedInstruction::ArgVal(0), // L0
//            SedInstruction::ArgVal(1),
//            SedInstruction::ArgVal(2), // L1
//            SedInstruction::Call(CallFunc::new("add")),
//            SedInstruction::Call(CallFunc::new("add")),
//            SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())), // return処理
//            //                            |<--------+------------>|
//            //                                      +-- func_def.argc + func_def.localc
//        ]
//    );
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_add)
//        .add_func(func_add3)
//        .assemble()
//        .generate();
//    if let Ok(code) = compile_result {
//        // println!("{}", code);
//        code
//    }
//    else
//    {
//        println!("Compile err occured");
//        "".to_string()
//    }
//}
//
//
///// 関数がさらに関数を呼び出すようなものの例
//pub fn gen_test_proc03() -> String
//{
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_strjoin = FuncDef::new("strjoin".to_string(), 2, 1, 1);
//    let mut func_strjoin3 =  FuncDef::new("strjoin3".to_string(), 3, 0, 1);
//
//    // ======================== func entry ========================
//
//    let entry_local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
//            SedInstruction::ConstVal(ConstVal::new("hello")),
//            SedInstruction::Set(&entry_local_vals[0]),
//            SedInstruction::ConstVal(ConstVal::new("Tom")),
//            SedInstruction::Set(&entry_local_vals[1]),
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::ConstVal(ConstVal::new("world")),
//            SedInstruction::LocalVal(1), // L1
//            SedInstruction::Call(CallFunc::new("strjoin3")),
//            SedInstruction::Set(&entry_local_vals[0]),
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::Set(&entry_local_vals[1]),
//        ]
//    );
//
//    // ======================== func add ========================
//
//    let func_add_local_vals:Vec<LocalVal> = vec![
//        LocalVal::new(0)
//    ];
//    // 関数の内容を定義する
//
//    func_strjoin.set_proc_contents(
//        vec![
//            // 引数の受取部分
//            // local変数は未初期化
//
//            SedInstruction::ConstVal(ConstVal::new("hello hello hello hello")),
//            SedInstruction::Set(&func_add_local_vals[0]), // 関数内のローカル変数に値を代入する
//
//            //                             |<---------argc----------->|<---localc--->|
//            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
//        ]
//    );
//
//    // ======================== func add3 ========================
//    let add3_arg_vals: Vec<ArgVal> = vec![
//        ArgVal::new(0),
//        ArgVal::new(1),
//        ArgVal::new(2),
//    ]; // entryの引数
//
//    func_strjoin3.set_proc_contents(
//        vec![
//            SedInstruction::ArgVal(0), // L0
//            SedInstruction::ArgVal(1),
//            SedInstruction::ArgVal(2), // L1
//            SedInstruction::Call(CallFunc::new("strjoin")),
//            SedInstruction::Call(CallFunc::new("strjoin")),
//            //SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())), // return処理
//            //                            |<--------+------------>|
//            //                                      +-- func_def.argc + func_deflocalc>
//            SedInstruction::Ret,
//        ]
//    );
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_strjoin)
//        .add_func(func_strjoin3)
//        .assemble()
//        .generate();
//    if let Ok(code) = compile_result{
//        // println!("{}", code);
//        code
//    }
//    else
//    {
//        println!("Compile err occured");
//        "".to_string()
//    }
//}
//
//pub fn gen_test_proc04() -> String 
//{
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_add = FuncDef::new("add".to_string(), 2, 0, 1);
//    let mut func_add3 =  FuncDef::new("add3".to_string(), 3, 0, 1);
//
//    // ======================== func entry ========================
//
//    let entry_local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
//            SedInstruction::ConstVal(ConstVal::new("101101110")),
//            SedInstruction::Set(&entry_local_vals[0]),
//            SedInstruction::ConstVal(ConstVal::new("11101110111")),
//            SedInstruction::Set(&entry_local_vals[1]),
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::ConstVal(ConstVal::new("111")),
//            SedInstruction::LocalVal(1), // L1
//            SedInstruction::Call(CallFunc::new("add3")),
//            SedInstruction::Set(&entry_local_vals[0]),
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::Set(&entry_local_vals[1]),
//        ]
//    );
//
//    // ======================== func add ========================
//    // 関数の内容を定義する
//
//    func_add.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("# 入力をaddloopの形式に変換".to_string())),
//            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/add 0;;\\1;\\2;/".to_string())),
//            SedInstruction::Sed(SedCode("b addloop".to_string())),
//            SedInstruction::Sed(SedCode(":addloop".to_string())),
//            SedInstruction::Sed(SedCode("s/add 1;\\([01]*\\);;;/1\\1/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 0;\\([01]*\\);;;/\\1/".to_string())),
//            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);\\([01]*\\);;/add \\1;\\2;\\3;0;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);;\\([01]*\\);/add \\1;\\2;0;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);\\([01]*\\)\\([01]\\);\\([01]*\\)\\([01]\\);/add \\1\\4\\6;\\2;\\3;\\5;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 000;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 001;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 010;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 011;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 100;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 101;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 110;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 111;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("t addloop".to_string())),
//            SedInstruction::Sed(SedCode("s/\\(.*\\)/~\\1;/".to_string())),
//        ]
//    );
//
//    // ======================== func add3 ========================
//    let add3_arg_vals: Vec<ArgVal> = vec![
//        ArgVal::new(0),
//        ArgVal::new(1),
//        ArgVal::new(2),
//    ]; // entryの引数
//
//    func_add3.set_proc_contents(
//        vec![
//            SedInstruction::ArgVal(0), // L0
//            SedInstruction::ArgVal(1),
//            SedInstruction::ArgVal(2), // L1
//            SedInstruction::Call(CallFunc::new("add")),
//            SedInstruction::Call(CallFunc::new("add")),
//            SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())), // return処理
//            //                            |<--------+------------>|
//            //                                      +-- func_def.argc + func_deflocalc>
//        ]
//    );
//
//    // ===========================================================
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_add)
//        .add_func(func_add3)
//        .assemble()
//        .generate();
//    if let Ok(code) = compile_result {
//        // println!("{}", code);
//        code
//    }
//    else
//    {
//        println!("Compile err occured");
//        "".to_string()
//    }
//}
//
//fn gen_test_proc05() -> String {
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_if_test = FuncDef::new("if_test".to_string(), 1, 1, 1);
//
//    // ======================== func entry ========================
//
//    let entry_local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
//            SedInstruction::ConstVal(ConstVal::new("101101110")),
//            SedInstruction::Set(&entry_local_vals[0]),
//            SedInstruction::ConstVal(ConstVal::new("11101110111")),
//            SedInstruction::Set(&entry_local_vals[1]),
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::Call(CallFunc::new("if_test")),
//            SedInstruction::Set(&entry_local_vals[0])
//        ]
//    );
//
//    // ======================== func add ========================
//    let func_if_test_arg_vals: Vec<ArgVal> = vec![
//        ArgVal::new(0),
//    ]; // entryの引数
//
//    let func_if_test_local_vals:Vec<LocalVal> = vec![
//        LocalVal::new(0)
//    ];
//
//    // 関数の内容を定義する
//
//    func_if_test.set_proc_contents(
//        vec![
//            //SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)/\\1/".to_string())),
//            SedInstruction::ConstVal(ConstVal::new("1")), // else節に入る
//            //SedInstruction::ConstVal(ConstVal::new("0")),
//            SedInstruction::IfProc(IfProc::new(
//                vec![
//                    SedInstruction::ConstVal(ConstVal::new("Hello")),
//                    SedInstruction::Set(&func_if_test_local_vals[0]),
//                ],
//                vec![
//                    SedInstruction::ConstVal(ConstVal::new("World")),
//                    SedInstruction::Set(&func_if_test_local_vals[0]),
//                ]
//            )),
//            SedInstruction::LocalVal(0),
//            SedInstruction::Ret,
//        ]
//    );
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_if_test)
//        .assemble()
//        .generate();
//    if let Ok(code) = compile_result {
//        // println!("{}", code);
//        code
//    }
//    else
//    {
//        println!("Compile err occured");
//        "".to_string()
//    }
//
//}
//
//
///// mul関数のテスト
///// 再帰的に定義したmul関数をコンパイル
//pub fn gen_test_proc06() -> String 
//{
//    // それぞれの関数のローカル変数の個数は後で適当なものに置き換える
//    let mut entry = FuncDef::new("entry".to_string(), 0, 2, 1);
//    let mut func_mul = FuncDef::new("mul".to_string(), 2, 1, 1);
//    let mut func_add = FuncDef::new("add".to_string(), 2, 0, 1);
//    let mut func_shift_left1 = FuncDef::new("shift_left1".to_string(), 1, 0, 1);
//    let mut func_shift_right1 = FuncDef::new("shift_right1".to_string(), 1, 0, 1);
//    let mut func_is_empty = FuncDef::new("is_empty".to_string(), 1, 0, 1);
//    let mut func_ends_with_zero = FuncDef::new("ends_with_zero".to_string(), 1, 0, 1);
//
//    // ======================== func entry ========================
//
//    // let entry_arg_vals: Vec<ArgVal> = vec![]; // entryの引数
//    let entry_local_vals = vec![
//        LocalVal::new(0), // L0
//        LocalVal::new(1), // L1
//    ]; // entryのローカル変数
//
//
//    entry.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
//            SedInstruction::ConstVal(ConstVal::new("101101110")),
//            SedInstruction::Set(&entry_local_vals[0]),
//            SedInstruction::ConstVal(ConstVal::new("11101110111")),
//            SedInstruction::Set(&entry_local_vals[1]),
//            SedInstruction::LocalVal(0), // L0
//            SedInstruction::LocalVal(1), // L0
//            SedInstruction::Call(CallFunc::new("mul")),
//            SedInstruction::Set(&entry_local_vals[0])
//        ]
//    );
//
//    // ======================== func ends_with_zero ========================
//    func_ends_with_zero.set_proc_contents(vec![
//        SedInstruction::Sed(SedCode("s/.*0$/~1;/ ".to_string())),
//        SedInstruction::Sed(SedCode("s/.*1$/~0;/ ".to_string())),
//    ]);
//    // ======================== func is_empty ========================
//    func_is_empty.set_proc_contents(vec![
//        SedInstruction::Sed(SedCode("s/~$/T/   ".to_string())),
//        SedInstruction::Sed(SedCode("s/~.*$/F/ ".to_string())),
//        SedInstruction::Sed(SedCode("s/T/~1;/  ".to_string())),
//        SedInstruction::Sed(SedCode("s/F/~0;/  ".to_string())),
//    ]);
//    // ======================== func shift_left1 ========================
//    func_shift_left1.set_proc_contents(vec![
//        SedInstruction::Sed(SedCode("s/\\(~[01]*\\)/\\10;/".to_string())),
//    ]);
//    // ======================== func shift_right1 ========================
//    func_shift_right1.set_proc_contents(vec![
//        SedInstruction::Sed(SedCode("s/\\(~[01]*\\)[01]/\\1;/".to_string())),
//    ]);
//    // ======================== func add ========================
//    // 関数の内容を定義する
//
//    func_add.set_proc_contents(
//        vec![
//            SedInstruction::Sed(SedCode("# 入力をaddloopの形式に変換".to_string())),
//            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/add 0;;\\1;\\2;/".to_string())),
//            SedInstruction::Sed(SedCode("b addloop".to_string())),
//            SedInstruction::Sed(SedCode(":addloop".to_string())),
//            SedInstruction::Sed(SedCode("s/add 1;\\([01]*\\);;;/1\\1/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 0;\\([01]*\\);;;/\\1/".to_string())),
//            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);\\([01]*\\);;/add \\1;\\2;\\3;0;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);;\\([01]*\\);/add \\1;\\2;0;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add \\([01]\\);\\([01]*\\);\\([01]*\\)\\([01]\\);\\([01]*\\)\\([01]\\);/add \\1\\4\\6;\\2;\\3;\\5;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 000;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 001;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 010;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 011;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 100;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 0;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 101;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 110;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;0\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("s/add 111;\\([01]*\\);\\([01]*\\);\\([01]*\\);/add 1;1\\1;\\2;\\3;/".to_string())),
//            SedInstruction::Sed(SedCode("t addloop".to_string())),
//            SedInstruction::Sed(SedCode("s/\\(.*\\)/~\\1;/".to_string())),
//        ]
//    );
//
//    // ======================== func mul ========================
//    let func_if_test_arg_vals: Vec<ArgVal> = vec![
//        ArgVal::new(0), // a
//        ArgVal::new(1), // b
//    ]; // entryの引数
//
//    let func_if_test_local_vals:Vec<LocalVal> = vec![
//        LocalVal::new(0)
//    ];
//
//    func_mul.set_proc_contents(
//        vec![
//            SedInstruction::ArgVal(1), // b
//            SedInstruction::Call(CallFunc::new("is_empty")),
//            SedInstruction::IfProc(IfProc::new(
//                vec![
//                    SedInstruction::ConstVal(ConstVal::new("0")),
//                    SedInstruction::Set(&func_if_test_local_vals[0]), // rstr
//                ],
//                vec![
//                    SedInstruction::ArgVal(1),
//                    SedInstruction::Call(CallFunc::new("ends_with_zero")),
//                    SedInstruction::IfProc(IfProc::new(
//                        vec![
//                            // rstr = mul(shift_left1(a), shift_right1(b))
//                            SedInstruction::ArgVal(0), // a
//                            SedInstruction::Call(CallFunc::new("shift_left1")),
//                            SedInstruction::ArgVal(1), // b
//                            SedInstruction::Call(CallFunc::new("shift_right1")),
//                            SedInstruction::Call(CallFunc::new("mul")),
//                            SedInstruction::Set(&func_if_test_local_vals[0]), // rstr
//                        ],
//                        vec![
//                            // rstr = add(a, mul(shift_left1(a), shift_right1(b)))
//                            SedInstruction::ArgVal(0), // a
//                            SedInstruction::Call(CallFunc::new("shift_left1")),
//                            SedInstruction::ArgVal(1), // b
//                            SedInstruction::Call(CallFunc::new("shift_right1")),
//                            SedInstruction::Call(CallFunc::new("mul")),
//                            SedInstruction::ArgVal(0), // a
//                            SedInstruction::Call(CallFunc::new("add")),
//                            SedInstruction::Set(&func_if_test_local_vals[0]),  // rstr
//                        ]
//                    )),
//                ]
//            )),
//            // return rstr;
//            SedInstruction::LocalVal(0), 
//            SedInstruction::Ret,
//        ]
//    );
//
//    let compile_result = CompilerBuilder::new()
//        .add_func(entry)
//        .add_func(func_mul)
//        .add_func(func_add)
//        .add_func(func_is_empty)
//        .add_func(func_shift_left1)
//        .add_func(func_shift_right1)
//        .add_func(func_ends_with_zero)
//        .assemble()
//        .generate();
//
//    match compile_result {
//        Ok(code) => code,
//        Err(e) => {
//            println!("{:?}", e);
//            "".to_string()
//        }
//    }
//}
//
