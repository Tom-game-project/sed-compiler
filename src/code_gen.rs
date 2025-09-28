use std::ops::{Deref, DerefMut};

/// この関数を使ってreturnアドレスを保存する
#[derive(Debug)]
struct ReturnAddrMarker(usize);
impl ReturnAddrMarker {
    pub fn incr(&mut self, d:usize) {
        self.0 += d;
    }

    fn get_retlabel(&self) -> String {
        format!("retlabel{}", self.0)
    }
}

#[derive(Debug)]
pub struct SedCode(pub String);

#[derive(Debug)]
struct SedProgram<'a>(Vec<SedInstruction<'a>>);

impl<'a> Deref for SedProgram<'a> {
    type Target = Vec<SedInstruction<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for SedProgram<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
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
    Set(&'a SedValue<'a>),
    /// func_def.retc分スタックを消費して値を返却する
    Ret,
    /// スタックのtopの値によって条件分岐
    IfProc(IfProc<'a>)
}

/// |... ArgVal ...|... LocalVal...|[... stack zone ...]
///  <---------fixed size -------->| <--  flex size -->
/// 返り値
#[derive(Debug)]
pub enum SedValue <'a>{
    ArgVal(&'a ArgVal),
    LocalVal(&'a LocalVal)
}

#[derive(Debug)]
pub struct IfProc <'a>{
    id:usize, // ラベルを決定するために使う
    then_proc: SedProgram<'a>,
    else_proc: SedProgram<'a>,
}

impl<'a> IfProc<'a> {
    pub fn new(
        then_proc:Vec<SedInstruction<'a>>,
        else_proc:Vec<SedInstruction<'a>>,
    ) -> Self {
        Self { id: 0, then_proc: SedProgram(then_proc), else_proc: SedProgram(else_proc) }
    }

    fn set_id(&mut self, id:usize) {
        self.id = id
    }

    fn init_return_addr_offset(&mut self, offset:usize) -> usize
    {
        let mut counter = offset;
        for i in &mut *self.then_proc {
            if let SedInstruction::Call(a) = i {
                a.return_addr_marker = ReturnAddrMarker(counter);
                counter += 1;
            }
            else if let SedInstruction::IfProc(if_proc) = i{
                counter = if_proc.init_return_addr_offset(counter);
            }
        }
        for i in &mut *self.else_proc {
            if let SedInstruction::Call(a) = i {
                a.return_addr_marker = ReturnAddrMarker(counter);
                counter += 1;
            }
            else if let SedInstruction::IfProc(if_proc) = i{
                counter = if_proc.init_return_addr_offset(counter);
            }
        }
        counter
    }
}

#[derive(Debug)]
pub struct ArgVal {
    id: usize // 引数の識別、同一スコープ内で重複がないように設定する
}

impl ArgVal {
    pub fn new(id: usize) -> Self{
        Self { id }
    }
}


#[derive(Debug)]
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

#[derive(Debug)]
pub struct ConstVal {
    data: String
}
impl ConstVal {
    pub fn new(data: &str) -> Self {
        Self { data: data.to_string() }
    }
}

#[derive(Debug)]
pub struct FuncDef <'a>{
    name: String, // 
    id: usize,
    argc: usize,  // 引数の個数
    localc: usize,// ローカル変数の個数
    retc: usize,  // 返り値の個数
    return_addr_offset: ReturnAddrMarker,
    proc_contents: SedProgram<'a>
}

impl <'a>FuncDef <'a>{
    pub fn new(name:String, argc: usize, localc: usize, retc: usize) -> Self{
        FuncDef {
            name: name,
            id: 0,
            argc,
            localc,
            retc,
            return_addr_offset: ReturnAddrMarker(0),
            proc_contents: SedProgram(vec![])
        }
    }

    /// 関数の内容がセットされ、更に,callにはreturn_addr_markerが0から1ずつインクリメントして設定される
    pub fn set_proc_contents(&mut self, proc_contents: Vec<SedInstruction<'a>>) -> usize {
        self.proc_contents = SedProgram(proc_contents);
        let mut counter = 0;
        for i in &mut *self.proc_contents {
            if let SedInstruction::Call(f) = i{
                f.return_addr_marker = ReturnAddrMarker(counter);
                counter += 1;
            }
            else if let SedInstruction::IfProc(if_proc) = i {
                counter = if_proc.init_return_addr_offset(counter);
            }
        }
        counter
    }

    /// offsetを設定
    /// offset分だけすべてのReturnAddrMarkerを加算
    /// offset + self.count_function_callを返却
    fn get_funclabel(&self)  -> String {
        format!("func{}", self.id)
    }
}

#[derive(Debug)]
pub struct CallFunc {
    // 何を呼ぶか
    // return addr
    func_name: String,
    /// 呼び出しもとのローカル変数の個数
    /// 関数の引数とローカル変数を足したもの
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

}


// =========================================================================================
//                                 ここから 共通実装
// =========================================================================================


/// returnアドレス解決のためのトレイト
/// proc_contentsのような構造を含む場合に必ず必要になってくるので、
/// 今後はよりデータに近い形に実装を変更していきたい
trait SetReturnAddrOffset {
    fn set_return_addr_offset(&mut self, offset: usize) -> usize;
}

impl<'a> SetReturnAddrOffset for SedProgram <'a>{
    fn set_return_addr_offset(&mut self, offset: usize) -> usize {
        let mut counter = 0;
        for i in &mut *self.0{
            if let SedInstruction::Call(a) = i {
                a.return_addr_marker.incr(offset);
                counter += 1;
            }
            else if let SedInstruction::IfProc(if_proc) = i {
                counter += if_proc.set_return_addr_offset(offset);
            }
        }
        counter
    }
}

impl<'a> SetReturnAddrOffset for FuncDef<'a> {
    fn set_return_addr_offset(&mut self, offset: usize) -> usize {
        self.return_addr_offset = ReturnAddrMarker(offset);
        self.proc_contents.set_return_addr_offset(offset)
    }
}

impl<'a> SetReturnAddrOffset for IfProc<'a> {
    fn set_return_addr_offset(&mut self, offset:usize) -> usize {
        self.then_proc.set_return_addr_offset(offset) + self.else_proc.set_return_addr_offset(offset)
    }
}


/// 返り値を処理する関数
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
        rstr.push_str(&i.proc_contents.sedgen_return_dispatcher(func_table)?);
    }
    Ok(rstr)
}

/// return dispatcherコードの生成
trait SedgenReturnDispatcher {
    fn sedgen_return_dispatcher(
        &self,
        func_table:&[FuncDef]
    ) -> Result<String, CompileErr>;
}

impl<'a> SedgenReturnDispatcher for SedProgram<'a> {
    fn sedgen_return_dispatcher(
            &self,
            func_table:&[FuncDef]
        ) -> Result<String, CompileErr> {
        let mut rstr = String::from("");
        for j in &**self {
            if let SedInstruction::Call(f) = j {
                rstr.push_str(&f.sedgen_return_dispatcher(func_table)?);
            } else if let SedInstruction::IfProc(if_proc) = j {
                rstr.push_str(&if_proc.sedgen_return_dispatcher(func_table)?);
            }
        }
        Ok(rstr)
    }
}

impl SedgenReturnDispatcher for CallFunc{
    fn sedgen_return_dispatcher(
        &self,
        func_table:&[FuncDef]
    ) -> Result<String, CompileErr> {
        let func_def = 
        if let Some(a) = find_function_definition_by_name(&self.func_name, func_table) {
            a
        } else {
            return Err(CompileErr::UndefinedFunction(self.func_name.clone()));
        };
        let mut rstr = "".to_string();
        let retlabel = self.return_addr_marker.get_retlabel();

        rstr.push_str(&format!("/^.*\\n:{}\\+[^\\|]*|.*$/ {{\n", retlabel));
        // s/.../.../形式のマッチ文開始
        {
            // pattern
            rstr.push_str(&format!("s/.*\\n:{}", retlabel));
            rstr.push_str(&"~[^\\~]*".repeat(func_def.argc));
            // 呼び出し元のローカル変数を復元する
            if 0 < self.localc  {
                rstr.push_str(&format!(
                    "\\({}{}\\)",
                    "~[^\\~]*".repeat(
                        self.localc - 1
                    ),
                    "~[^\\|]*".repeat(
                        1
                    )
                ));
            }
            else {
                rstr.push_str("\\(\\)");
            }
            rstr.push_str("|\\n");
            //rstr.push_str(&"~\\([^\\~;]*\\)".repeat(func_def.retc));
            rstr.push_str(
                &format!("\\({}\\)", "~[^\\~;]*".repeat(func_def.retc))
            );
            rstr.push_str(";$/");
            rstr.push_str("\\1\\2");
            rstr.push_str("/\n");
        }
        rstr.push_str(&format!("b {}\n",retlabel));
        rstr.push_str("}\n");
        Ok(rstr)

    }
}

impl<'a> SedgenReturnDispatcher for IfProc<'a>{
    fn sedgen_return_dispatcher(&self, func_table: &[FuncDef]) -> Result<String, CompileErr> {
        let mut rstr:String = String::from("");
        rstr.push_str(&self.then_proc.sedgen_return_dispatcher(func_table)?);
        rstr.push_str(&self.else_proc.sedgen_return_dispatcher(func_table)?);
        Ok(rstr)
    }
}

trait SetLocalc {
    fn set_localc(&mut self, localc: usize);
}


impl<'a> SetLocalc for SedProgram <'a>{
    fn set_localc(&mut self, localc: usize) {
        for j in &mut **self {
            if let SedInstruction::Call(call_func) = j {
                call_func
                    .set_localc(localc);
            } else if let SedInstruction::IfProc(if_proc) = j {
                if_proc.set_localc(localc);
            }
        }
    }
}

impl SetLocalc for CallFunc {
    /// この関数を呼び出しているスコープにおいて
    /// どれだけのローカル変数が使用されているかをセットする
    /// 主にassemble_funcs
    fn set_localc(&mut self, localc: usize)
    {
        self.localc = localc
    }
}

impl<'a> SetLocalc for IfProc <'a>{
    fn set_localc(&mut self, localc: usize) {
        self.then_proc.set_localc(localc);
        self.else_proc.set_localc(localc);
    }
}


// =========================================================================================
//                                 ここまで 共通実装
// =========================================================================================

/// func_tableから名前の一致する関数を探し出す
fn find_function_definition_by_name<'a>(name: &str, func_table:&'a [FuncDef]) -> Option<&'a FuncDef<'a>>
{
        func_table
            .iter()
            .find(|f|f.name == name.to_string())
}


fn sedgen_func_call(
    func_def :&FuncDef,
    return_addr_marker: &ReturnAddrMarker,
    stack_size:usize,
) -> Option<String> {
    let retlabel = return_addr_marker.get_retlabel();
    let arg_pattern: String = format!(
            "\\({}\\)\\({}\\)",
            "~[^\\~]*".repeat(stack_size - func_def.argc),
            "~[^\\~]*".repeat(func_def.argc)
        );
    let arg_string = "\\2\\1";

    Some(
        format!("
# {}関数の呼び出し
s/{}/:{}{}|/
H
b {}
:{}
",
        func_def.name,
        arg_pattern,
        retlabel,
        arg_string,
        func_def.get_funclabel(),
        retlabel
        )
    )
}

#[derive(Debug)]
pub enum CompileErr {
    UndefinedFunction(String),
    StackUnderFlow(String),
    PoppingValueFromEmptyStack,
}

// ------------------------- resolve instructions -----------------------------

/// 生のsedプログラムを格納する
fn resolve_sed_instruction(
    rstr: &mut String, 
    sed: &SedCode,
    stack_size:usize
) -> usize
{
    rstr.push_str(&format!("{}\n", sed.0));
    stack_size
}

/// 関数の呼び出し。
/// スタックトップから引数の個数分消費し、返り値分を積む
fn resolve_call_instruction(
    rstr: &mut String,
    func_call: &CallFunc,
    func_table:&[FuncDef], 
    mut stack_size: usize
) -> Result<usize, CompileErr>
{
    // 関数の定義を見つけ出し関数の呼び方を決定する
    let func_def = 
        if let Some(a) = find_function_definition_by_name(&func_call.func_name, func_table) {
            a
        } else {
            return Err(CompileErr::UndefinedFunction(func_call.func_name.clone()));
        };
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
        return Err(CompileErr::UndefinedFunction("unknown".to_string()))
    }
    Ok(stack_size)
}

/// 引数をスタックに積む
fn resolve_argval_instruction(
    rstr: &mut String,
    a: &ArgVal,
    mut stack_size:usize
) -> usize
{
    rstr.push_str(&format!("s/{}/{}/\n",
        format!(
            "\\({}\\)\\(~[^\\~]*\\)\\({}\\)", 
            "~[^\\~]*".repeat(a.id),
            "~[^\\~]*".repeat(stack_size - a.id - 1),
        ),
        "\\1\\2\\3\\2"
    ));
    // rstr.push_str(&format!("# DEBUG from arg stack_size {}\n", stack_size));
    stack_size += 1;
    stack_size
}

/// ローカル変数をスタックに積む
fn resolve_localval_instruction(
    rstr: &mut String,
    a: &LocalVal,
    func_def: &FuncDef,
    mut stack_size:usize
) -> usize
{ 
    rstr.push_str(&format!(
        "s/{}/{}/\n",
            format!(
                "\\({}\\)\\(~[^\\~]*\\)\\({}\\)", 
                "~[^\\~]*".repeat(func_def.argc + a.id),
                "~[^\\~]*".repeat(stack_size - (func_def.argc + a.id) - 1),
            ),
        "\\1\\2\\3\\2"
    ));
    // rstr.push_str(&format!("# DEBUG from local stack_size {}\n", stack_size));
    stack_size += 1;
    stack_size
}

/// 定数をスタックに積む
fn resolve_constval_instruction(
    rstr: &mut String,
    a: &ConstVal,
    mut stack_size:usize
    ) -> usize
{
    rstr.push_str(&format!("s/{}/{}/\n",
        format!(
            "\\({}\\)", 
            "~[^\\~]*".repeat(stack_size),
        ),
        format!("\\1~{}", a.data)
    ));
    // rstr.push_str(&format!("# DEBUG from const stack_size {}\n", stack_size));
    stack_size += 1;
    stack_size
}


/// スタックトップを消費してローカル変数または引数にセットする
/// 
fn resolve_set_instruction(
    rstr: &mut String,
    a: &SedValue,
    func_def: &FuncDef,
    fixed_offset:usize,
    mut stack_size:usize) -> Result<usize, CompileErr>
{ 
    // スタックの最上部を消費して、値をsetする
    // stack_sizeが
    // fixed_offsetだったらerror

    if stack_size < fixed_offset {
        return Err(CompileErr::StackUnderFlow(format!("stack_size: {}, fixed_offset: {}", stack_size, fixed_offset)));
    }
    rstr.push_str(&format!("# stack_size: {}, fixed_offset: {}\n", stack_size, fixed_offset));
    match a {
        // TODO index関係のエラー処理
        SedValue::ArgVal(arg_a) => {
            rstr.push_str(&format!("s/{}/{}/\n",
                           //\1      \2            \3
                    format!("\\({}\\)~[^\\~]*\\({}\\)\\(~[^\\~]*\\)",
                        "~[^\\~]*".repeat(arg_a.id),
                        "~[^\\~]*".repeat(stack_size - arg_a.id - 2)
                    ),
                    "\\1\\3\\2"
                )
            );
        }
        SedValue::LocalVal(loc_a) => {
            rstr.push_str(&format!("s/{}/{}/\n",
                           //\1              \2      \3
                    format!("\\({}\\)~[^\\~]*\\({}\\)\\(~[^\\~]*\\)",
                        "~[^\\~]*".repeat(func_def.argc + loc_a.id),
                        "~[^\\~]*".repeat(stack_size - (func_def.argc + loc_a.id) - 2)
                    ),
                    "\\1\\3\\2"
                )
            );
        }
    }
    stack_size -= 1;
    Ok(stack_size)
}

/// 返り値`return`の処理
fn resolve_ret_instructions(
    rstr: &mut String,
    func_def: &FuncDef,
    stack_size:usize,
    fixed_offset:usize,
) -> Result<usize, CompileErr>
{
    if fixed_offset + func_def.retc > stack_size {
        return Err(CompileErr::PoppingValueFromEmptyStack);
    }
    let arg_pattern: String = format!(
            "{}\\({}\\)",
            "~[^\\~]*".repeat(stack_size - func_def.retc),
            "~[^\\~]*".repeat(func_def.retc)
        );
    let arg_string = "\\1;";
    rstr.push_str(&format!("s/{}/{}/\n", arg_pattern, arg_string));
    rstr.push_str("b return_dispatcher\n"); // 最後は必ずreturn
    Ok(0)
}

fn resolve_if_instructions(
    rstr: &mut String,
    a: &IfProc,
    func_def: &FuncDef,
    mut stack_size:usize,
    func_table:&[FuncDef]
) -> Result<usize, CompileErr> {
    // if scope内では入る前のstack size以下になってはいけない
    stack_size = stack_size - 1;
    let then_stack_size = stack_size; // fixed
    let else_stack_size = stack_size; // fixed
    let mut then_code = String::new();
    let mut else_code = String::new();
    resolve_instructions(&mut then_code, func_def, &a.then_proc, then_stack_size, 0, func_table)?;
    resolve_instructions(&mut else_code, func_def, &a.else_proc, else_stack_size, 0, func_table)?;

    let reset_flag = format!("reset_flag{}", a.id);
    let else_label = format!("else{}", a.id);
    let then_label = format!("then{}", a.id);
    let endif_label = format!("endif{}", a.id);

    rstr.push_str(&format!("
t{reset_flag}
:{reset_flag}

s/\\(.*\\)~[0]\\+$/\\1/

t {else_label}
b {then_label}

:{then_label}
s/\\(.*\\)~\\([^\\~]*\\)\\+$/\\1/
{then_code}

b {endif_label}
:{else_label}
{else_code}
b {endif_label}
:{endif_label}
",
));
    Ok(stack_size)
}

fn resolve_instructions(
    rstr: &mut String,
    func_def: &FuncDef,
    proc_contents: &[SedInstruction], // 命令列
    fixed_offset:usize,
    mut stack_size:usize,
    func_table:&[FuncDef]
) -> Result<usize, CompileErr>
{
    stack_size = stack_size + fixed_offset;
    for instruction in proc_contents {
        stack_size = match instruction {
            SedInstruction::Sed(sed) => resolve_sed_instruction(rstr, sed, stack_size),
            SedInstruction::Call(func_call) => resolve_call_instruction(rstr, func_call, func_table, stack_size)?,
            SedInstruction::ArgVal(a)=> resolve_argval_instruction(rstr, a, stack_size),
            SedInstruction::LocalVal(a) => resolve_localval_instruction(rstr, a, func_def, stack_size),
            SedInstruction::ConstVal(a)=> resolve_constval_instruction(rstr, a, stack_size),
            SedInstruction::Set(a) => resolve_set_instruction(rstr, a, func_def, fixed_offset, stack_size)?,
            SedInstruction::Ret => resolve_ret_instructions(rstr, func_def, stack_size, fixed_offset)?,
            SedInstruction::IfProc(a) => resolve_if_instructions(rstr, a, func_def, stack_size, func_table)?,
        };
    }

    Ok(stack_size)
}

fn sedgen_func_def(
    func_def: &FuncDef,
    func_table:&[FuncDef]
) -> Result<String, CompileErr> {
    let is_entry = func_def.name == "entry";
    let fixed_offset = func_def.argc + func_def.localc;
    
    let mut rstr = 
        if is_entry {
            "".to_string()
        }
        else 
        {   
            let pattern = format!("\\({}\\)", "~[^\\~]*".repeat(func_def.argc));
            let args_out = "\\1";
            let locals_out = (0..func_def.localc)
                .map(|_| "~init")
                .collect::<String>();
            format!(":{}\n
s/:retlabel[0-9]\\+{}[^\\|]*|$/{}{}/
s/\\n\\(.*\\)/\\1/
", func_def.get_funclabel(), pattern, args_out, locals_out)
        };

    let mut stack_size = 0;

    stack_size = resolve_instructions(
        &mut rstr,
        func_def,
        &func_def.proc_contents,
        fixed_offset, 
        stack_size,
        func_table
    )?;

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

/// ifを表現するラベルに割り当てる名前を解決する関数
fn resolve_if_label(proc_contents: &mut Vec<SedInstruction>, mut min_id: usize) -> usize {
    for j in &mut *proc_contents {
        if let SedInstruction::IfProc(a) = j {
            a.set_id(min_id);
            min_id += 1;
            min_id = resolve_if_label(&mut a.then_proc, min_id);
            min_id = resolve_if_label(&mut a.else_proc, min_id);
        }
    }
    min_id
}

/// return addrの決定
/// 関数を集めて、
/// return アドレス(ラベル)、
/// 関数のラベルも解決する
pub fn assemble_funcs(func_table:&mut [FuncDef]){
    let mut pad = 0;
    let mut label_id = 0;
    for i in & mut *func_table{
        pad = i.set_return_addr_offset(pad);
        i.id = label_id;
        label_id += 1;
    }

    // ローカル変数の解決
    for i in &mut *func_table {
        i.proc_contents.set_localc(i.localc + i.argc);
    }

    // ifスコープのラベル解決
    let mut if_min_id = 0;

    for i in &mut *func_table {
        if_min_id = resolve_if_label(&mut i.proc_contents, if_min_id);
    }
}

