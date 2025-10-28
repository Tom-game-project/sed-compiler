use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// compiler state
pub struct Unassembled;
pub struct Assembled;

pub struct CompilerBuilder<State> {
    func_table: Vec<FuncDef>,
    _state: PhantomData<State>,
}

impl CompilerBuilder<Unassembled> {
    pub fn new() -> Self {
        Self {
            func_table: Vec::new(),
            _state: PhantomData,
        }
    }

    /// 関数定義を一つ追加する
    pub fn add_func(mut self, func: FuncDef) -> Self {
        self.func_table.push(func);
        self
    }

    /// 「組立」処理を実行し、状態を Assembled に遷移させる
    pub fn assemble(mut self) -> CompilerBuilder<Assembled> {
        // entry pointをリストの先頭に配置する
        if let Some(elem) = self.func_table.pop_if(|a|a.name == "entry") {
            self.func_table.insert(0, 
                elem
            );
        }
        // ID割り当て、オフセット計算、ラベル解決など
        assemble_funcs(&mut self.func_table);
        CompilerBuilder {
            func_table: self.func_table,
            _state: PhantomData,
        }
    }
}

// この型はすでにassembleを実行している状態のビルダー
impl CompilerBuilder<Assembled> {
    /// sedコードを生成する
    pub fn generate(self) -> Result<String, CompileErr> {
        println!("Generating sed code...");
        sedgen_func_table(&self.func_table)
    }
}

/// この関数を使ってreturnアドレスを保存する
#[derive(Debug)]
struct ReturnAddrMarker(usize);
impl ReturnAddrMarker {
    pub fn incr(&mut self, d: usize) {
        self.0 += d;
    }

    fn get_retlabel(&self) -> String {
        format!("retlabel{}", self.0)
    }
}

#[derive(Debug)]
pub struct SedCode(pub String);

#[derive(Debug)]
struct SedProgram(Vec<SedInstruction>);

impl Deref for SedProgram {
    type Target = Vec<SedInstruction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SedProgram {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub enum SedInstruction {
    /// 生のSedプログラム
    Sed(SedCode),
    Val(Value),
    /// スタックに定数を積む
    ConstVal(ConstVal),
    /// 関数をよびスタックを関数の引数分消費して
    /// 返り値をスタックに積む(返り値の個数だけスタックにpushされる)
    Call(CallFunc),
    /// スタックをpopしてそれをvalueにセットする
    Set(Value),
    /// func_def.retc分スタックを消費して値を返却する
    Ret,
    /// スタックのtopの値によって条件分岐
    IfProc(IfProc),
}

#[derive(Debug)]
pub enum Value {
    Arg(usize),
    Local(usize),
}

#[derive(Debug)]
pub struct IfProc {
    id: usize, // ラベルを決定するために使う
    then_proc: SedProgram,
    else_proc: SedProgram,
}

impl IfProc {
    pub fn new(then_proc: Vec<SedInstruction>, else_proc: Vec<SedInstruction>) -> Self {
        Self {
            id: 0,
            then_proc: SedProgram(then_proc),
            else_proc: SedProgram(else_proc),
        }
    }

    fn set_id(&mut self, id: usize) {
        self.id = id
    }
}

#[derive(Debug)]
pub struct ArgVal {
    id: usize, // 引数の識別、同一スコープ内で重複がないように設定する
}

impl ArgVal {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug)]
pub struct LocalVal {
    id: usize, // 引数の識別、同一スコープ内で重複がないように設定する
}
impl LocalVal {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug)]
pub struct ConstVal {
    data: String,
}
impl ConstVal {
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct FuncDef {
    name: String, //
    id: usize,
    argc: usize,   // 引数の個数
    localc: usize, // ローカル変数の個数
    retc: usize,   // 返り値の個数
    return_addr_offset: ReturnAddrMarker,
    proc_contents: SedProgram,
    arg_list: Vec<ArgVal>,
    local_list: Vec<LocalVal>,
}

impl FuncDef {
    pub fn new(name: &str, argc: usize, localc: usize, retc: usize) -> Self {
        FuncDef {
            name: name.to_string(),
            id: 0,
            argc,
            localc,
            retc,
            return_addr_offset: ReturnAddrMarker(0),
            proc_contents: SedProgram(vec![]),
            arg_list: (0..argc).map(ArgVal::new).collect(),
            local_list: (0..localc).map(LocalVal::new).collect(),
        }
    }

    /// 関数の内容がセットされ、更に,callにはreturn_addr_markerが0から1ずつインクリメントして設定される
    pub fn set_proc_contents(&mut self, proc_contents: Vec<SedInstruction>) -> usize {
        self.proc_contents = SedProgram(proc_contents);
        let counter = 0;
        self.setup_proc_contents(counter)
    }

    fn get_funclabel(&self) -> String {
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
    return_addr_marker: ReturnAddrMarker,
}

impl CallFunc {
    pub fn new(func_name: &str) -> Self {
        Self {
            func_name: func_name.to_string(),
            localc: 0,
            return_addr_marker: ReturnAddrMarker(0),
        }
    }
}

#[derive(Debug)]
pub enum CompileErr {
    UndefinedFunction(String),
    StackUnderFlow(String),
    PoppingValueFromEmptyStack,
}

// =========================================================================================
//                                 ここから 共通実装
// =========================================================================================

/// 命令リストが設定されたときの最初のセットアップ
/// returnアドレス解決のためのトレイト
/// proc_contentsのような構造を含む場合に必ず必要になってくるので、
trait ReturnAddrOffsetResolver {
    /// 命令列がセットされた直後にリターンアドレスをセットします。
    fn setup_proc_contents(&mut self, counter: usize) -> usize;
    /// 実行可能なプログラムを生成する前に、リターンアドレスの解決をします
    fn set_return_addr_offset(&mut self, offset: usize) -> usize;
}

impl ReturnAddrOffsetResolver for SedProgram {
    fn setup_proc_contents(&mut self, mut counter: usize) -> usize {
        for i in &mut **self {
            if let SedInstruction::Call(f) = i {
                f.return_addr_marker = ReturnAddrMarker(counter);
                counter += 1;
            } else if let SedInstruction::IfProc(if_proc) = i {
                counter = if_proc.setup_proc_contents(counter);
            }
        }
        counter
    }

    fn set_return_addr_offset(&mut self, offset: usize) -> usize {
        let mut counter = 0;
        for i in &mut **self {
            if let SedInstruction::Call(a) = i {
                a.return_addr_marker.incr(offset);
                counter += 1;
            } else if let SedInstruction::IfProc(if_proc) = i {
                counter += if_proc.set_return_addr_offset(offset);
            }
        }
        counter
    }
}

impl ReturnAddrOffsetResolver for FuncDef {
    fn setup_proc_contents(&mut self, counter: usize) -> usize {
        self.proc_contents.setup_proc_contents(counter)
    }

    fn set_return_addr_offset(&mut self, offset: usize) -> usize {
        self.return_addr_offset = ReturnAddrMarker(offset);
        self.proc_contents.set_return_addr_offset(offset)
    }
}

impl ReturnAddrOffsetResolver for IfProc {
    fn setup_proc_contents(&mut self, mut counter: usize) -> usize {
        counter = self.then_proc.setup_proc_contents(counter);
        counter = self.else_proc.setup_proc_contents(counter);
        counter
    }

    fn set_return_addr_offset(&mut self, offset: usize) -> usize {
        self.then_proc.set_return_addr_offset(offset)
            + self.else_proc.set_return_addr_offset(offset)
    }
}

/// 返り値を処理する関数
fn sedgen_return_dispatcher(func_table: &[FuncDef]) -> Result<String, CompileErr> {
    let mut rstr = "
:return_dispatcher
H
x
h
s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\1/
x
s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\2/
"
    .to_string();
    for i in func_table {
        rstr.push_str(&i.proc_contents.sedgen_return_dispatcher(func_table)?);
    }
    Ok(rstr)
}

/// return dispatcherコードの生成
trait SedgenReturnDispatcher {
    fn sedgen_return_dispatcher(&self, func_table: &[FuncDef]) -> Result<String, CompileErr>;
}

impl SedgenReturnDispatcher for SedProgram {
    fn sedgen_return_dispatcher(&self, func_table: &[FuncDef]) -> Result<String, CompileErr> {
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

impl SedgenReturnDispatcher for CallFunc {
    fn sedgen_return_dispatcher(&self, func_table: &[FuncDef]) -> Result<String, CompileErr> {
        let func_def = find_function_definition_by_name(&self.func_name, func_table)?;
        let mut rstr = "".to_string();
        let retlabel = self.return_addr_marker.get_retlabel();

        rstr.push_str(&format!("/^.*\\n:{}~[^\\|]*|.*$/ {{\n", retlabel));
        // s/.../.../形式のマッチ文開始
        {
            // pattern
            rstr.push_str(&format!("s/.*\\n:{}", retlabel));
            rstr.push_str(&"~[^\\~]*".repeat(func_def.argc));
            // 呼び出し元のローカル変数を復元する
            if 0 < self.localc {
                rstr.push_str(&format!(
                    "\\({}{}\\)",
                    "~[^\\~]*".repeat(self.localc - 1),
                    "~[^\\|]*".repeat(1)
                ));
            } else {
                rstr.push_str("\\(\\)");
            }
            rstr.push_str("|\\n");
            rstr.push_str(&format!("\\({}\\)", "~[^\\~;]*".repeat(func_def.retc)));
            rstr.push_str(";$/");
            rstr.push_str("\\1\\2");
            rstr.push_str("/\n");
        }
        rstr.push_str(&format!("b {}\n", retlabel));
        rstr.push_str("}\n");
        Ok(rstr)
    }
}

impl SedgenReturnDispatcher for IfProc {
    fn sedgen_return_dispatcher(&self, func_table: &[FuncDef]) -> Result<String, CompileErr> {
        let mut rstr: String = String::from("");
        rstr.push_str(&self.then_proc.sedgen_return_dispatcher(func_table)?);
        rstr.push_str(&self.else_proc.sedgen_return_dispatcher(func_table)?);
        Ok(rstr)
    }
}

trait SetLocalc {
    fn set_localc(&mut self, localc: usize);
}

impl SetLocalc for SedProgram {
    fn set_localc(&mut self, localc: usize) {
        for j in &mut **self {
            if let SedInstruction::Call(call_func) = j {
                call_func.set_localc(localc);
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
    fn set_localc(&mut self, localc: usize) {
        self.localc = localc
    }
}

impl SetLocalc for IfProc {
    fn set_localc(&mut self, localc: usize) {
        self.then_proc.set_localc(localc);
        self.else_proc.set_localc(localc);
    }
}

// 引数とローカル変数

/// |... ArgVal ...|... LocalVal...|[... stack zone ...]
///  <---------fixed size -------->| <--  flex size -->
/// 返り値
pub trait ResolvePopAndSetProc: Debug {
    fn resolve_pop_and_set_proc(&self, stack_size: usize, func_def: &FuncDef) -> String;
}

impl ResolvePopAndSetProc for ArgVal {
    fn resolve_pop_and_set_proc(&self, stack_size: usize, _func_def: &FuncDef) -> String {
        resolve_pop_and_set_proc(stack_size, self.id)
    }
}

impl ResolvePopAndSetProc for LocalVal {
    fn resolve_pop_and_set_proc(&self, stack_size: usize, func_def: &FuncDef) -> String {
        resolve_pop_and_set_proc(stack_size, func_def.argc + self.id)
    }
}

// =========================================================================================
//                                 ここまで 共通実装
// =========================================================================================

/// func_tableから名前の一致する関数を探し出す
fn find_function_definition_by_name<'a>(
    name: &str,
    func_table: &'a [FuncDef],
) -> Result<&'a FuncDef, CompileErr> {
    if let Some(func_def) = func_table.iter().find(|f| f.name == name.to_string()) {
        Ok(func_def)
    } else {
        Err(CompileErr::UndefinedFunction(format!("{}", name)))
    }
}

// ------------------------- resolve instructions -----------------------------

/// 生のsedプログラムを格納する
fn resolve_sed_instruction(rstr: &mut String, sed: &SedCode, stack_size: usize) -> usize {
    rstr.push_str(&format!("{}\n", sed.0));
    stack_size
}

/// 関数の呼び出し。
/// スタックトップから引数の個数分消費し、返り値分を積む
fn resolve_call_instruction(
    rstr: &mut String,
    func_call: &CallFunc,
    func_table: &[FuncDef],
    mut stack_size: usize,
) -> Result<usize, CompileErr> {
    // 関数の定義を見つけ出し関数の呼び方を決定する
    let func_def = find_function_definition_by_name(&func_call.func_name, func_table)?;
    if let Some(code) = sedgen_func_call(func_def, &func_call.return_addr_marker, stack_size) {
        rstr.push_str(&code);
        stack_size -= func_def.argc;
        stack_size += func_def.retc;
    } else {
        return Err(CompileErr::UndefinedFunction("unknown".to_string()));
    }
    Ok(stack_size)
}

fn resolve_stack_push_proc(stack_size: usize, offset: usize) -> String {
    format!(
        "s/{}/{}/\n",
        format!(
            "\\({}\\)\\(~[^\\~]*\\)\\({}\\)",
            "~[^\\~]*".repeat(offset),
            "~[^\\~]*".repeat(stack_size - offset - 1),
        ),
        "\\1\\2\\3\\2"
    )
}

/// 引数をスタックに積む
fn resolve_argval_instruction(rstr: &mut String, a: &ArgVal, stack_size: usize) -> usize {
    rstr.push_str(&resolve_stack_push_proc(stack_size, a.id));
    stack_size + 1
}

/// ローカル変数をスタックに積む
fn resolve_localval_instruction(
    rstr: &mut String,
    a: &LocalVal,
    func_def: &FuncDef,
    stack_size: usize,
) -> usize {
    rstr.push_str(&resolve_stack_push_proc(stack_size, func_def.argc + a.id));
    stack_size + 1
}

/// 定数をスタックに積む
fn resolve_constval_instruction(rstr: &mut String, a: &ConstVal, mut stack_size: usize) -> usize {
    rstr.push_str(&format!(
        "s/{}/{}/\n",
        format!("\\({}\\)", "~[^\\~]*".repeat(stack_size),),
        format!("\\1~{}", a.data)
    ));
    stack_size += 1;
    stack_size
}

fn resolve_pop_and_set_proc(stack_size: usize, offset: usize) -> String {
    format!(
        "s/{}/{}/\n",
        //\1      \2            \3
        format!(
            "\\({}\\)~[^\\~]*\\({}\\)\\(~[^\\~]*\\)",
            "~[^\\~]*".repeat(offset),
            "~[^\\~]*".repeat(stack_size - offset - 2)
        ),
        "\\1\\3\\2"
    )
}

/// スタックトップを消費してローカル変数または引数にセットする
///
fn resolve_set_instruction(
    rstr: &mut String,
    a: &dyn ResolvePopAndSetProc,
    func_def: &FuncDef,
    fixed_offset: usize,
    mut stack_size: usize,
) -> Result<usize, CompileErr> {
    // スタックの最上部を消費して、値をsetする
    // stack_sizeが
    // fixed_offsetだったらerror

    if stack_size <= fixed_offset {
        return Err(CompileErr::StackUnderFlow(format!(
            "stack_size: {}, fixed_offset: {}",
            stack_size, fixed_offset
        )));
    }
    rstr.push_str(&a.resolve_pop_and_set_proc(stack_size, func_def));
    stack_size -= 1;
    Ok(stack_size)
}

/// 返り値`return`の処理
fn resolve_ret_instructions(
    rstr: &mut String,
    func_def: &FuncDef,
    stack_size: usize,
    fixed_offset: usize,
) -> Result<usize, CompileErr> {
    if fixed_offset + func_def.retc > stack_size {
        println!("@ {}", func_def.name);
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
    mut stack_size: usize,
    func_table: &[FuncDef],
) -> Result<usize, CompileErr> {
    // if scope内では入る前のstack size以下になってはいけない
    stack_size = stack_size - 1;
    let then_stack_size = stack_size; // fixed
    let else_stack_size = stack_size; // fixed
    let mut then_code = String::new();
    let mut else_code = String::new();
    resolve_instructions(
        &mut then_code,
        func_def,
        &a.then_proc,
        then_stack_size,
        0,
        func_table,
    )?;
    resolve_instructions(
        &mut else_code,
        func_def,
        &a.else_proc,
        else_stack_size,
        0,
        func_table,
    )?;

    let reset_flag = format!("reset_flag{}", a.id);
    let else_label = format!("else{}", a.id);
    let then_label = format!("then{}", a.id);
    let endif_label = format!("endif{}", a.id);

    rstr.push_str(&format!(
        "
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
    fixed_offset: usize,
    mut stack_size: usize,
    func_table: &[FuncDef],
) -> Result<usize, CompileErr> {
    stack_size = stack_size + fixed_offset;
    for instruction in proc_contents {
        stack_size = match instruction {
            SedInstruction::Sed(sed) => resolve_sed_instruction(rstr, sed, stack_size),
            SedInstruction::Call(func_call) => {
                resolve_call_instruction(rstr, func_call, func_table, stack_size)?
            }
            SedInstruction::Val(a) => match *a {
                Value::Arg(index) => {
                    resolve_argval_instruction(rstr, &func_def.arg_list[index], stack_size)
                }
                Value::Local(index) => resolve_localval_instruction(
                    rstr,
                    &func_def.local_list[index],
                    func_def,
                    stack_size,
                ),
            },
            SedInstruction::ConstVal(a) => resolve_constval_instruction(rstr, a, stack_size),
            SedInstruction::Set(a) => resolve_set_instruction(
                rstr,
                match *a {
                    Value::Local(index) => &func_def.local_list[index],
                    Value::Arg(index) => &func_def.arg_list[index],
                },
                func_def,
                fixed_offset,
                stack_size,
            )?,
            SedInstruction::IfProc(a) => {
                resolve_if_instructions(rstr, a, func_def, stack_size, func_table)?
            }
            SedInstruction::Ret => {
                resolve_ret_instructions(rstr, func_def, stack_size, fixed_offset)?
            }
        };
    }

    Ok(stack_size)
}

// ------------------------- sedgen instructions -----------------------------

fn sedgen_func_call(
    func_def: &FuncDef,
    return_addr_marker: &ReturnAddrMarker,
    stack_size: usize,
) -> Option<String> {
    let retlabel = return_addr_marker.get_retlabel();
    let arg_pattern: String = format!(
        "\\({}\\)\\({}\\)",
        "~[^\\~]*".repeat(stack_size - func_def.argc),
        "~[^\\~]*".repeat(func_def.argc)
    );
    let arg_string = "\\2\\1";

    Some(format!(
        "
# function call: {}
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
    ))
}

fn sedgen_func_def(func_def: &FuncDef, func_table: &[FuncDef]) -> Result<String, CompileErr> {
    let is_entry = func_def.name == "entry";
    let fixed_offset = func_def.argc + func_def.localc;

    let mut rstr = if is_entry {
        // format!("{}\n", (0..fixed_offset).map(|_| "~init").collect::<String>())
        "".to_string()
    } else {
        let pattern = format!("\\({}\\)", "~[^\\~]*".repeat(func_def.argc));
        let args_out = "\\1";
        let locals_out = (0..func_def.localc).map(|_| "~init").collect::<String>();
        format!(
            ":{}\n
s/:retlabel[0-9]\\+{}[^\\|]*|$/{}{}/
s/\\n\\(.*\\)/\\1/
",
            func_def.get_funclabel(),
            pattern,
            args_out,
            locals_out
        )
    };

    let mut stack_size = 0;

    stack_size = resolve_instructions(
        &mut rstr,
        func_def,
        &func_def.proc_contents,
        fixed_offset,
        stack_size,
        func_table,
    )?;

    if is_entry {
        rstr.push_str("b done\n"); // entry return
    } else {
        rstr.push_str("b return_dispatcher\n"); // 最後は必ずreturn
    }
    Ok(rstr)
}

/// この関数を呼び出す前に必ずassemble_funcsを実行しfunc_tableの設定を終わらせる必要がある
fn sedgen_func_table(func_table: &[FuncDef]) -> Result<String, CompileErr> {
    let mut rstr = "".to_string();
    for i in func_table {
        let code = sedgen_func_def(i, func_table)?;
        rstr.push_str(&code);
    }
    let code = sedgen_return_dispatcher(func_table)?;
    rstr.push_str(&code);
    rstr.push_str(":done");
    Ok(rstr)
}

// ------------------------- resolve entry -----------------------------

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
fn assemble_funcs(func_table: &mut [FuncDef]) {
    let mut pad = 0;
    let mut label_id = 0;
    for i in &mut *func_table {

        // println!("set id {}", pad);
        pad += i.set_return_addr_offset(pad);

        i.id = label_id;
        label_id += 1;
    }
    // println!("{:#?}", func_table);

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
