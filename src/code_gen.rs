/// この関数を使ってreturnアドレスを保存する
struct ReturnAddrMarker(usize);
impl ReturnAddrMarker {
    fn incr(&mut self, d:usize) {
        self.0 += d;
    }
}

struct SedCode(String);

enum SedInstruction{
    Sed(SedCode),
    Call(CallFunc),
}

pub struct AstFunc{
    name: String, // 
    id: usize,
    return_addr_offset: ReturnAddrMarker,
    proc_contents: Vec<SedInstruction>
} 
// いくつFunction callがあるか数える関数

impl AstFunc{
    fn new(name:String) -> Self{
        AstFunc {
            name: name,
            id: 0,
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
    fn set_proc_contents(&mut self, proc_contents: Vec<SedInstruction>) {
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
    arg_pattern: String,
    arg_string:String,
    return_addr_marker: ReturnAddrMarker
}

impl CallFunc {
    fn new(func_name: &str, arg_pattern: &str, arg_string: &str) -> Self{   
        Self { 
            func_name:func_name.to_string(),
            arg_pattern: arg_pattern.to_string(),
            arg_string:arg_string.to_string(),
            return_addr_marker: ReturnAddrMarker(0)
        }
    }
}

/// return_dispatcher sectionを生成する
fn sedgen_return_dispatcher(func_table: &[AstFunc]) -> String
{
    let mut rstr = "
:return_dispatcher
x
".to_string();

    for i in func_table {
        for j in &i.proc_contents {
            if let SedInstruction::Call(f) = j {
                rstr.push_str(&format!("
/\\n:retlabel{}[^\\|]*|$/ {{
	s/\\(.*\\)\\n\\(.*\\)|$/\\1/
	x
	b retlabel{}
}}
",
                f.return_addr_marker.0,
                f.return_addr_marker.0
                ));
            }
        }
    }
    rstr
}

fn sedgen_func_call(func_call :&CallFunc, func_table:&[AstFunc]) -> Option<String> {
    let func_def = func_table.iter().find(|f|f.name == func_call.func_name)?;
// この正規表現の部分をうまくやれば引数を渡せる
// s/.*/:retlabel{}{}|/
    Some(
        format!("
# {}関数の呼び出し
s/{}/:retlabel{}{}|/
H
b func{}
:retlabel{}
",
        func_call.func_name,
        func_call.arg_pattern,
        func_call.return_addr_marker.0,
        func_call.arg_string,
        func_def.id ,
        func_call.return_addr_marker.0        
        )
    )
}

pub enum CompileErr {
    UndefinedFunction,
}

fn sedgen_func_def(func_def: &AstFunc, func_table:&[AstFunc]) -> Result<String, CompileErr> {
    let is_entry = func_def.name == "entry";
    
    let mut rstr = 
        if is_entry {
            "".to_string()
        }
        else 
        {
            format!(":func{}\n", func_def.id)
        };
    for i in &func_def.proc_contents {
        if let SedInstruction::Sed(sed) = i {
            rstr.push_str(&format!("{}\n", sed.0));
        }else if let SedInstruction::Call(call) = i {
            if let Some(code) = sedgen_func_call(call, func_table) {
                rstr.push_str(&code);
            }
            else
            {
                return Err(CompileErr::UndefinedFunction)
            }
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

pub fn sedgen_func_table(func_table:&[AstFunc]) -> Result<String, CompileErr>
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
fn assemble_funcs(func_table:&mut [AstFunc]){
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

pub fn build_ast_test() -> String{
    let mut entry = AstFunc::new("entry".to_string());
    let mut func_a = AstFunc::new("func_a".to_string());
    let mut func_b = AstFunc::new("func_b".to_string());

    entry.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/.*/hello/".to_string())),
            SedInstruction::Call(
                CallFunc::new("func_a", "\\(.*\\)", "-\\1")),
        ]
    );
    // 関数の内容を定義する
    func_a.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("g".to_string())),
            SedInstruction::Sed(SedCode("s/:retlabel[0-9]\\+-\\([^\\-]*\\)|$/\\1/".to_string())),
            SedInstruction::Call(
                CallFunc::new("func_b", "\\n\\(.*\\)", "-\\1-\\1")),
        ]
    );

    func_b.set_proc_contents(
        vec![
            SedInstruction::Sed(SedCode("s/:retlabel[0-9]\\+-\\([^\\-]*\\)-\\([^\\-]*\\)|$/\\1===\\2/".to_string())),
            //SedInstruction::Call(CallFunc::new("func_a","-arg1"))
        ]
    );

    let mut func_table = vec![entry, func_a, func_b];
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


