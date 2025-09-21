/// この関数を使ってreturnアドレスを保存する
struct ReturnAddrMarker(usize);
impl ReturnAddrMarker {
    pub fn incr(&mut self, d:usize) {
        self.0 += d;
    }
}

pub struct SedCode(pub String);

pub enum SedInstruction <'a>{
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

pub struct ArgVal {
    id: usize // 引数の識別、同一スコープ内で重複がないように設定する
}

impl ArgVal {
    pub fn new(id: usize) -> Self{
        Self { id }
    }
}

pub struct LocalVal {
    id: usize // 引数の識別、同一スコープ内で重複がないように設定する
}
impl LocalVal {
    pub fn new(id: usize) -> Self {
        Self {
            id
        }
    }
}

pub struct ConstVal {
    data: String
}
impl ConstVal {
    pub fn new(data: &str) -> Self {
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
    pub fn new(name:String, argc: usize, localc: usize, retc: usize) -> Self{
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
    pub fn set_proc_contents(&mut self, proc_contents: Vec<SedInstruction<'a>>) {
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

pub struct CallFunc {
    // 何を呼ぶか
    // return addr
    func_name: String,
    /// 呼び出しもとのローカル変数の個数
    localc: usize,
    return_addr_marker: ReturnAddrMarker
}

impl CallFunc {
    pub fn new(func_name: &str) -> Self{   
        Self { 
            func_name:func_name.to_string(),
            localc: 0,
            return_addr_marker: ReturnAddrMarker(0)
        }
    }

    fn set_localc(&mut self, localc: usize)
    {
        self.localc = localc
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
) -> Result<String, CompileErr>
{
    let func_def = 
        if let Some(a) = find_function_definition_by_name(&call_func.func_name, func_table) {
        a
    } else {
        return Err(CompileErr::UndefinedFunction);
    };
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
                                                        //    |
        rstr.push_str(&"~\\([^\\|]*\\)".repeat( //    |
                                                        //    |
                1                                       //    |
        ));                                             // <--+
        rstr.push_str("|\\n");
        rstr.push_str(&"~\\([^\\~;]*\\)".repeat(func_def.retc));
        rstr.push_str(";$/");
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
    Ok(rstr)
}

/// ローカル変数を返せる
fn sedgen_return_dispatcher(func_table: &[FuncDef]) -> Result<String, CompileErr>
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
                let code = sedgen_return_dispatcher_helper(
                        f, 
                        func_table
                )?;
                rstr.push_str(&code);
            }
        }
    }
    Ok(rstr)
}

fn sedgen_func_call(
    func_def :&FuncDef,
    return_addr_marker: &ReturnAddrMarker,
    stack_size:usize,
) -> Option<String> {
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

/// この関数を呼び出す前に必ずassemble_funcsを実行しfunc_tableの設定を終わらせる必要がある
pub fn sedgen_func_table(func_table:&[FuncDef]) -> Result<String, CompileErr>
{
    let mut rstr = "".to_string();
    for i in func_table{
        let code = sedgen_func_def(i, func_table)?;
        rstr.push_str(&code);
    }
    let code = sedgen_return_dispatcher(func_table)?;
    rstr.push_str(&code);
    rstr.push_str(":done");
    Ok(rstr)
}

/// return addrの決定
/// 関数を集めて、return アドレス(ラベル)を解決する
/// また、関数のラベルも解決する
pub fn assemble_funcs(func_table:&mut [FuncDef]){
    //let mut func_table = vec![func_a, func_b];
    // return addrの解決
    let mut pad = 0;
    let mut label_id = 0;
    for i in & mut *func_table{
        pad = i.set_return_addr_offset(pad);
        i.id = label_id;
        label_id += 1;
    }

    // ローカル変数の解決
    for i in func_table {
        for j in &mut *i.proc_contents {
            if let SedInstruction::Call(call_func) = j {
                call_func
                    .set_localc(i.localc + i.argc);
            }
        }
    }
}

