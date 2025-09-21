/// この関数を使ってreturnアドレスを保存する
struct ReturnAddrMarker(usize);
impl ReturnAddrMarker {
    fn incr(&mut self, d:usize) {
        self.0 += d;
    }
}

struct SedCode(String);

enum SedInstruction <'a>{
    /// 生のSedプログラム
    Sed(SedCode),
    /// スタックに引数を積む
    ArgVal(&'a ArgVal),
    /// スタックにローカル引数を積む
    LocalVal(&'a LocalVal),
    /// スタックに定数を積む
    ConstVal(ConstVal),
    /// 関数をよびスタックを関数の引数分消費して
    /// 返り値をスタックに積む
    Call(CallFunc), // calling function
    Ret(&'a[&'a StackVal<'a>]) // return 
}

/// |... ArgVal ...|... LocalVal...|
///  <---------fixed size -------->
/// 返り値
enum StackVal <'a>{
    ArgVal(&'a ArgVal),
    LocalVal(&'a LocalVal)
}

struct ArgVal {
    id: usize // 引数の識別、同一スコープ内で重複がないように設定する
}

impl ArgVal {
    fn new(id: usize) -> Self{
        Self { id }
    }
}

struct LocalVal {
    id: usize // 引数の識別、同一スコープ内で重複がないように設定する
}
impl LocalVal {
    fn new(id: usize) -> Self {
        Self {
            id
        }
    }
}

struct ConstVal {
    data: String
}
impl ConstVal {
    fn new(data: &str) -> Self {
        Self { data: data.to_string() }
    }
}

pub struct FuncDef <'a>{
    name: String, // 
    id: usize,
    argc: usize,  // 引数の個数
    localc: usize,// ローカル変数の個数
    retc: usize,
    return_addr_offset: ReturnAddrMarker,
    proc_contents: Vec<SedInstruction<'a>>
} 
// いくつFunction callがあるか数える関数

impl <'a>FuncDef <'a>{
    fn new(name:String, argc: usize, localc: usize, retc: usize) -> Self{
        FuncDef {
            name: name,
            id: 0,
            argc,
            localc,
            retc,
            return_addr_offset: ReturnAddrMarker(0),
            proc_contents: vec![]
        }
    }

    fn count_function_call(&self) -> usize {
        self.proc_contents
            .iter()
            .filter(|i| 
            if let SedInstruction::Call(_f) = i {true} else {false})
            .count()
    }

    /// 関数の内容がセットされ、更に,callにはreturn_addr_markerが0から1ずつインクリメントして設定される
    fn set_proc_contents(&mut self, proc_contents: Vec<SedInstruction<'a>>) {
        self.proc_contents = proc_contents;

        let mut counter = 0;
        for i in &mut self.proc_contents {
            if let SedInstruction::Call(f) = i{
                f.return_addr_marker = ReturnAddrMarker(counter);
                counter += 1;
            }
        }
    }

    /// offsetを設定
    /// offset文だけすべてのReturnAddrMarkerを加算
    /// offset + self.count_function_callを返却
    fn set_return_addr_offset(&mut self, offset:usize) -> usize{
        self.return_addr_offset = ReturnAddrMarker(offset);
        for i in &mut self.proc_contents {
            if let SedInstruction::Call(f) = i {
                f.return_addr_marker.incr(offset);
            }
        }
        offset + self.count_function_call()
    }
}

struct CallFunc {
    // 何を呼ぶか
    // return addr
    func_name: String,
    /// 呼び出しもとのローカル変数の個数
    localc: usize,
    return_addr_marker: ReturnAddrMarker
}

impl CallFunc {
    fn new(func_name: &str, localc: usize) -> Self{   
        Self { 
            func_name:func_name.to_string(),
            localc,
            return_addr_marker: ReturnAddrMarker(0)
        }
    }
}

fn find_function_definition_by_name<'a>(name: &str, func_table:&'a [FuncDef]) -> Option<&'a FuncDef<'a>>
{
        func_table
            .iter()
            .find(|f|f.name == name.to_string())
}

fn sedgen_return_dispatcher_helper(
    call_func: &CallFunc,
    func_table:&[FuncDef]
) -> String
{
    let func_def = find_function_definition_by_name(&call_func.func_name, func_table)
        .expect("関数が存在しません");
    let mut rstr = "".to_string();
    let retlabel = format!("retlabel{}", 
        call_func.return_addr_marker.0
    );

    rstr.push_str(&format!("/^.*\\n:{}\\+[^\\|]*|.*$/ {{\n", retlabel));
    // s/.../.../形式のマッチ文開始
    {
        // pattern
        rstr.push_str(&format!("s/.*\\n:{}", retlabel));
        rstr.push_str(&"~[^\\~]*".repeat(func_def.argc));
        rstr.push_str(&"~\\([^\\~]*\\)".repeat( // <--+
                call_func.localc - 1                    //    |
        ));                                             //    |-[呼び出し元のローカル変数の個数]
                                                        //    | TODO: 
        rstr.push_str(&"~\\([^\\|]*\\)".repeat( //    | call_func.localcはCallFuncによって
                                                        //    | 適切に管理される必要がある
                1                                       //    |
        ));                                             // <--+
        rstr.push_str("|\\n");
        rstr.push_str(&"~\\([^\\~;]*\\)".repeat(func_def.retc));
        rstr.push_str(";$/");
        // TODO:
        rstr.push_str(
            &(0..
                call_func.localc // 呼び出しもとのローカル変数の個数
                + func_def.retc  // 返り値の個数
            )
            .map(|i| format!("~\\{}", i + 1))
            .collect::<Vec<String>>()
            .join(""));
        rstr.push_str("/\n");
    }
    rstr.push_str(&format!("b {}\n",retlabel));
    rstr.push_str("}\n");
    rstr
}

/// ローカル変数を返せる
fn sedgen_return_dispatcher(func_table: &[FuncDef]) -> String
{
    let mut rstr = "
:return_dispatcher
H
x
h
s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\1/
x
s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\2/
".to_string();
    for i in func_table {
        for j in &i.proc_contents {
            if let SedInstruction::Call(f) = j {
                rstr.push_str(&sedgen_return_dispatcher_helper(
                        f, 
                        func_table
                ));
            }
        }
    }
    rstr
}

fn sedgen_func_call(
    func_def :&FuncDef,
    return_addr_marker: &ReturnAddrMarker,
    stack_size:usize,
) -> Option<String> {
    // 呼び出している場所のスタックの状態はコンパイル時に判明するため TODO
    //let func_def = 
    //    func_table
    //    .iter()
    //    .find(|f|f.name == func_call.func_name)?; 
    // この正規表現の部分をうまくやれば引数を渡せる
    // s/.*/:retlabel{}{}|/
    let arg_pattern: String = 
        "~\\([^\\~]*\\)".repeat(stack_size);
    let arg_string: String = format!("{}{}",
            (0..func_def.argc)
                .map(|i| format!("~\\{}", 
                        stack_size 
                        - func_def.argc // argの始まりに合わせる 
                        + i 
                        + 1 // indexは1スタート
                ))
                .collect::<Vec<String>>()
                .join(""),
            (0..stack_size - func_def.argc) // 引数は消費され
                                            // スタックからpopされる
                .map(|i| format!("~\\{}", 
                        i
                        + 1 // indexは1スタート
                ))
                .collect::<Vec<String>>()
                .join(""),
        );
        //format!("-\\{}");
    Some(
        format!("
# {}関数の呼び出し
s/{}/:retlabel{}{}|/
H
b func{}
:retlabel{}
",
        func_def.name,
        arg_pattern,
        return_addr_marker.0,
        arg_string,
        func_def.id ,
        return_addr_marker.0        
        )
    )
}

pub enum CompileErr {
    UndefinedFunction,
}

fn sedgen_func_def(func_def: &FuncDef, func_table:&[FuncDef]) -> Result<String, CompileErr> {
    let is_entry = func_def.name == "entry";

    let arg_vals_len = func_def.argc;
    let local_vals_len = func_def.localc;
    let fixed_offset = arg_vals_len + local_vals_len;
    let mut stack_size = fixed_offset;
    
    let mut rstr = 
        if is_entry {
            "".to_string()
        }
        else 
        {
            let pattern = "~\\([^\\~]*\\)".repeat(func_def.argc);

            let out = (0..func_def.argc)
                .map(|i| format!("~\\{}", i 
                    + 1 // indexは1スタート
                    ))
                .collect::<String>();
            format!(":func{}\n
s/:retlabel[0-9]\\+{}[^\\|]*|$/{}/
s/\\n\\(.*\\)/\\1/
", func_def.id, pattern, out)
        };
    for i in &func_def.proc_contents {
        if let SedInstruction::Sed(sed) = i {
            rstr.push_str(&format!("{}\n", sed.0));
        }else if let SedInstruction::Call(func_call) = i {
            // 関数の定義を見つけ出し関数の呼び方を決定する
            let func_def = find_function_definition_by_name(
                &func_call.func_name,
                func_table
                ).expect("定義されていない関数");
            if let Some(code) = sedgen_func_call(
                func_def,
                &func_call.return_addr_marker,
                stack_size,
            )
            {
                rstr.push_str(&code);
                stack_size -= func_def.argc;
                stack_size += func_def.retc;
            }
            else
            {
                return Err(CompileErr::UndefinedFunction)
            }
        } else if let SedInstruction::ArgVal(a) = i {
                let mut next_pattern = 
                    (1..stack_size + 1)
                        .map(|d| format!("~\\{}", d))
                        .collect::<Vec<String>>();
                next_pattern.push(format!("~\\{}", 
                        a.id 
                        + 1 // index が1から始まる
                )); // スタックに引数を積む
                rstr.push_str(&format!("s/{}/{}/\n",
                    "~\\([^\\~]*\\)".repeat(stack_size),
                    next_pattern.join("")
                )
                );
                stack_size += 1;
        } else if let SedInstruction::LocalVal(a) = i {
                let mut next_pattern = 
                    (1..stack_size + 1)
                        .map(|d| format!("~\\{}", d))
                        .collect::<Vec<String>>();
                next_pattern.push(format!("~\\{}",
                        a.id 
                        + 1 // index が1から始まる
                        + arg_vals_len // 引数分のoffset
                )); // スタックにローカル変数を:world積む
                rstr.push_str(&format!("s/{}/{}/\n",
                    "~\\([^\\~]*\\)".repeat(stack_size),
                    next_pattern.join("")
                ));
                stack_size += 1;
        } else if let SedInstruction::ConstVal(a) = i {
                let mut next_pattern = 
                (1..stack_size + 1)
                    .map(|d| format!("~\\{}", d))
                    .collect::<Vec<String>>();
                next_pattern.push(format!("~{}",
                        a.data
                )); // 定数をスタックに積む
                rstr.push_str(&format!("s/{}/{}/\n",
                    "~\\([^\\~]*\\)".repeat(stack_size),
                    next_pattern.join("")
                ));
                stack_size += 1;
        } else if let SedInstruction::Ret(a) = i {
            //match  {
            //    
            //}
        }
    }

    if is_entry {
        rstr.push_str("b done\n"); // entry return
    }
    else
    {
        rstr.push_str("b return_dispatcher\n"); // 最後は必ずreturn
    }
    Ok(rstr)
}

pub fn sedgen_func_table(func_table:&[FuncDef]) -> Result<String, CompileErr>
{
    let mut rstr = "".to_string();
    for i in func_table{
        let code = sedgen_func_def(i, func_table)?;
        rstr.push_str(&code);
    }
    rstr.push_str(&sedgen_return_dispatcher(func_table));
    rstr.push_str(":done");
    Ok(rstr)
}

/// return addrの決定
/// 関数を集めて、return アドレス(ラベル)を解決する
/// また、関数のラベルも解決する
fn assemble_funcs(func_table:&mut [FuncDef]){
    //let mut func_table = vec![func_a, func_b];
    // return addrの決定
    // 関数の
    let mut pad = 0;
    let mut label_id = 0;
    for i in func_table{
        pad = i.set_return_addr_offset(pad);
        i.id = label_id;
        label_id += 1;
    }
}

// ======================================================================================

pub fn build_ast_test02() -> String{
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
            SedInstruction::Call(CallFunc::new("pow", 2)),
            SedInstruction::LocalVal(&local_vals[1]), // L1
            SedInstruction::ConstVal(ConstVal::new("2")),
            SedInstruction::Call(CallFunc::new("pow", 2)),
            SedInstruction::Call(CallFunc::new("add", 2)),
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
        println!("{}", code);
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
pub fn build_ast_test03() -> String
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
            SedInstruction::Call(CallFunc::new("add", 2)),
            SedInstruction::Call(CallFunc::new("add", 2)),
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
            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2/;".to_string())),
        ]
    );

    let mut func_table = vec![entry, func_add];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}

/// 関数がさらに関数を呼び出すようなものの例
pub fn build_ast_test04() -> String
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
            SedInstruction::Call(CallFunc::new("add3", 2))
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
            SedInstruction::Sed(SedCode("s/~\\([^\\~]*\\)~\\([^\\~]*\\)/~\\1\\2;/".to_string())),
            //SedInstruction::Call(
            //    CallFunc::new("func_b", "\\n\\(.*\\)", "-\\1-\\1")),
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
            SedInstruction::Call(CallFunc::new("add", 3)),
            SedInstruction::Call(CallFunc::new("add", 3)),
            SedInstruction::Sed(SedCode("s/~[^\\~]*~[^\\~]*~[^\\~]*~\\([^\\~]*\\)/~\\1;/".to_string())),
        ]
    );

    let mut func_table = vec![entry, func_add, func_add3];
    assemble_funcs(&mut func_table);
    if let Ok(code) = sedgen_func_table(&func_table) {
        println!("{}", code);
        code
    }
    else
    {
        println!("Compile err occured");
        "".to_string()
    }
}
