/// code_gen.rs 
/// 
/// -- ja --
/// code_genにおいて、内部の中間表現(IR)はwasmやforthのようなスタック指向のランタイムのように記述できます。
/// 具体的にはSoilIR構造体がその役割を果たします。
/// 関数や条件分岐にはsedのラベルを使います。ラベルの重複を防ぐために、IRのツリーを再帰的に探索して解決します。
/// 具体的には`CompilerBuilder<Unassembled>::assemble`が諸々の解決をします。
/// -- en --
/// In the code generation phase, the internal IR is modeled after a stack-oriented runtime like WebAssembly or Forth.
/// Specifically, the `SoilIR` struct fulfills this role.
///
/// `sed` labels are used for functions and conditional branches. To prevent label collisions, the IR tree is traversed recursively to resolve them.
/// This resolution is specifically handled by `CompilerBuilder<Unassembled>::assemble`.
///
/// ## Structure of the Generated Sed Script
/// ```txt
/// +--------------------------+
/// | entry function section   |
/// | junp to :done            |
/// +--------------------------+
/// | def func0                |
/// | -(return addr resolver)- |
/// +--------------------------+
/// | def func1                |
/// | -(return addr resolver)- |
/// +--------------------------+
/// | def func2                |
/// | -(return addr resolver)- |
/// +--------------------------+
///             :
/// |           :              |
/// +---------:done------------+
/// :done
/// ```
///

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

// CompilerBuilder state
pub struct Unassembled;
pub struct Assembled;
pub struct Linked;

/// 
pub struct CompilerBuilder<State> {
    func_table: Vec<FuncDef>,
    consumed_table: ConsumedTable,
    _state: PhantomData<State>,
}

impl Default for CompilerBuilder<Unassembled> {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerBuilder<Unassembled> {
    pub fn new() -> Self {
        Self {
            func_table: Vec::new(),
            consumed_table: ConsumedTable {
                func_label_id: 0,
                if_id: 0,
            },
            _state: PhantomData,
        }
    }

    /// add function definition
    pub fn add_func(mut self, func: FuncDef) -> Self {
        self.func_table.push(func);
        self
    }

    /// resolve and setting SoilIR status.
    /// then this function returns "Assembled" CompilerBuilder.
    pub fn assemble(mut self) -> CompilerBuilder<Assembled> {
        // "entry" is a special entry function.
        // move the "entry" function to the top of the functions list.
        if let Some(elem) = self.func_table.pop_if(|a| a.name == "entry") {
            self.func_table.insert(0, elem);
        }

        // resolve return addr
        // Assign an unique id to the function
        let mut pad = 0;
        let mut label_id = 0;
        for i in &mut *self.func_table {
            pad += i.set_return_addr_offset(pad);

            i.id = label_id;
            label_id += 1;
        }

        // resolve local variables
        for i in &mut *self.func_table {
            i.proc_contents.set_localc(i.localc + i.argc);
        }

        // resolve if scope labels
        let mut if_min_id = 0;

        for i in &mut *self.func_table {
            if_min_id = resolve_if_label(&mut i.proc_contents, if_min_id);
        }

        let consumed = ConsumedTable {
            func_label_id: pad,
            if_id: if_min_id,
        };

        CompilerBuilder {
            func_table: self.func_table,
            consumed_table: consumed,
            _state: PhantomData,
        }
    }
}

impl CompilerBuilder<Assembled> {
    /// generate sed script
    pub fn generate(&self) -> Result<String, CompileErr> {
        let mut rstr = "".to_string();
        let call_tree = 
            create_return_dispatcher_btree_map(&self.func_table)?;
        for i in &self.func_table {
            let code = sedgen_func_def(i, &self.func_table, &call_tree)?;
            rstr.push_str(&code);
        }
        rstr.push_str(":done");
        Ok(rstr)
    }

    /// show IR table
    pub fn resolved_show_table(&self) {
        println!("{:#?}", self.func_table);
    }
}

/// This structure is stored in `return_addr`.
///
/// `ReturnAddrMarker` represents the address to which the function will return.
/// It holds a unique identifier.
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
struct SoilIR(Vec<SoilIRInstruction>);

impl Deref for SoilIR {
    type Target = Vec<SoilIRInstruction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SoilIR {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Sed-Compiler Intermediate Representation(IR)
#[derive(Debug)]
pub enum SoilIRInstruction {
    /// Raw Sed Script
    Sed(SedCode),
    /// push Value to stack
    Val(Value),
    /// push ConstVal to stack
    ConstVal(ConstVal),
    /// Call the function and consume the stack by the number of function arguments for computation
    /// Push the return value onto the stack
    Call(CallFunc),
    /// pop stack and set value
    Set(Value),
    /// Consumes the stack by the number specified in func_def.retc and returns the value.
    Ret,
    /// Conditional branching based on the value at the top of the stack
    /// 0 -> else
    /// otherwise -> then
    IfProc(IfProc),
}

#[derive(Debug)]
pub enum Value {
    Arg(usize),
    Local(usize),
}

#[derive(Debug)]
pub struct IfProc {
    id: usize, // unique if label
    then_proc: SoilIR,
    else_proc: SoilIR,
}

impl IfProc {
    pub fn new(then_proc: Vec<SoilIRInstruction>, else_proc: Vec<SoilIRInstruction>) -> Self {
        Self {
            id: 0,
            then_proc: SoilIR(then_proc),
            else_proc: SoilIR(else_proc),
        }
    }

    fn set_id(&mut self, id: usize) {
        self.id = id
    }
}

#[derive(Debug)]
pub struct ArgVal {
    id: usize, // 引数の識別、同一スコープ内で重複がないように設定する
               // A unique integer within the same scope that does not duplicate
}

impl ArgVal {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug)]
pub struct LocalVal {
    id: usize, // 引数の識別、同一スコープ内で重複がないように設定する
               // A unique integer within the same scope that does not duplicate
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
    name: String, // function name
    id: usize,
    argc: usize,   // the number of arguments
    localc: usize, // the number of local variables
    retc: usize,   // the number of return variables
    return_addr_offset: ReturnAddrMarker,
    proc_contents: SoilIR,
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
            proc_contents: SoilIR(vec![]),
            arg_list: (0..argc).map(ArgVal::new).collect(),
            local_list: (0..localc).map(LocalVal::new).collect(),
        }
    }

    /// 関数の内容がセットされ、更に,callにはreturn_addr_markerが0から1ずつインクリメントして設定される
    /// setting function contents.
    pub fn set_proc_contents(&mut self, proc_contents: Vec<SoilIRInstruction>) -> usize {
        self.proc_contents = SoilIR(proc_contents);
        self.setup_proc_contents(0)
    }

    fn get_funclabel(&self) -> String {
        format!("func{}", self.id)
    }
}

#[derive(Debug)]
pub struct CallFunc {
    func_name: String,
    /// 呼び出しもとのローカル変数の個数
    /// 関数の引数とローカル変数を足したもの
    localc: usize, // 
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
    PoppingValueFromEmptyStack(String),
    Fatal,
}

// =========================================================================================
//                                      traits
// =========================================================================================

// returnアドレス解決のためのトレイト
/// the trait for resolving return address
trait ReturnAddrOffsetResolver {
    // 中間表現SoilIRが設定された直後にリターンアドレスをセットします。
    // 関数の内部実装でcallが使われている部分を0からカウントしナンバリングします。
    /// Assigns return addresses right after the SoilIR is generated.
    /// Each `call` instruction in the function is assigned a sequential index, starting from `counter`.
    fn setup_proc_contents(&mut self, counter: usize) -> usize;
    // 実行可能なプログラムを生成するために、リターンアドレスの解決をします
    // この関数を呼び出すとすべての関数呼び出しが連番になり、数字の重複がなくなります。
    /// Resolves return addresses for executable generation.
    /// After calling this, all function calls are sequentially indexed with no overlap.
    fn set_return_addr_offset(&mut self, offset: usize) -> usize;
}

impl ReturnAddrOffsetResolver for SoilIR {
    fn setup_proc_contents(&mut self, mut counter: usize) -> usize {
        for i in &mut **self {
            if let SoilIRInstruction::Call(f) = i {
                f.return_addr_marker = ReturnAddrMarker(counter);
                counter += 1;
            } else if let SoilIRInstruction::IfProc(if_proc) = i {
                counter = if_proc.setup_proc_contents(counter);
            }
        }
        counter
    }

    fn set_return_addr_offset(&mut self, offset: usize) -> usize {
        let mut counter = 0;
        for i in &mut **self {
            if let SoilIRInstruction::Call(a) = i {
                a.return_addr_marker.incr(offset);
                counter += 1;
            } else if let SoilIRInstruction::IfProc(if_proc) = i {
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

/// Returns a map organized as follows:
///
/// * **Key**: Function name (`String`)
/// * **Value**: List of call sites (`Vec<ReturnAddrResolveCode>`)
fn create_return_dispatcher_btree_map(
    func_table: &[FuncDef],
) -> Result<BTreeMap<String, Vec<ReturnAddrResolveCode>>, CompileErr> {
    let mut rdic: BTreeMap<String, Vec<ReturnAddrResolveCode>> = BTreeMap::new();
    for i in func_table {
        // ある関数以下での呼び出しをカウント
        // 呼び出されている関数から、呼び出し元をリストアップしたい
        for j in i.proc_contents.sedgen_return_dispatcher(func_table)? {
            if let Some(rdic_mut) = &mut rdic.get_mut(&j.func_name) {
                rdic_mut.push(j);
            } else {
                rdic.insert(j.func_name.clone(), vec![j]);
            }
        }
    }
    Ok(rdic)
}

/// ```txt
/// |            :             |
/// +--------------------------+
/// | def funcn                |
/// | -(return addr resolver)- | <- this is the trait for generating `return addr resolver`
/// +--------------------------+
/// |            :             |
/// ```
trait SedgenReturnDispatcher {
    fn sedgen_return_dispatcher(
        &self,
        func_table: &[FuncDef],
    ) -> Result<Vec<ReturnAddrResolveCode>, CompileErr>;
}

#[derive(Debug)]
struct ReturnAddrResolveCode {
    func_name: String,
    code: String,
}

impl SedgenReturnDispatcher for SoilIR {
    fn sedgen_return_dispatcher(
        &self,
        func_table: &[FuncDef],
    ) -> Result<Vec<ReturnAddrResolveCode>, CompileErr> {
        let mut rvec = Vec::new();
        for j in &**self {
            if let SoilIRInstruction::Call(f) = j {
                rvec.append(&mut f.sedgen_return_dispatcher(func_table)?);
            } else if let SoilIRInstruction::IfProc(if_proc) = j {
                rvec.append(&mut if_proc.sedgen_return_dispatcher(func_table)?);
            }
        }
        Ok(rvec)
    }
}

impl SedgenReturnDispatcher for CallFunc {
    fn sedgen_return_dispatcher(
        &self,
        func_table: &[FuncDef],
    ) -> Result<Vec<ReturnAddrResolveCode>, CompileErr> {
        let func_def = find_function_definition_by_name(&self.func_name, func_table)?;
        let mut rstr = "".to_string();
        let retlabel = self.return_addr_marker.get_retlabel();

        rstr.push_str(&format!("/^.*\\n:{}~[^\\|]*|.*$/ {{\n", retlabel));
        // s/.../.../形式のマッチ文開始
        {
            // pattern
            rstr.push_str(&format!("s/.*\\n:{}", retlabel));
            rstr.push_str(&"~[^\\~]*".repeat(func_def.argc));
            // -- ja --
            // 呼び出し元のローカル変数を復元する
            // -- en --
            // restore the caller's local variables
            if 0 < self.localc {
                rstr.push_str(&format!(
                    "\\({}{}\\)",
                    "~[^\\~]*".repeat(self.localc - 1),
                    "~[^\\|]*"
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
        Ok(vec![ReturnAddrResolveCode {
            func_name: self.func_name.to_string(),
            code: rstr,
        }])
    }
}

impl SedgenReturnDispatcher for IfProc {
    fn sedgen_return_dispatcher(
        &self,
        func_table: &[FuncDef],
    ) -> Result<Vec<ReturnAddrResolveCode>, CompileErr> {
        let mut rvec = Vec::new();
        rvec.append(&mut self.then_proc.sedgen_return_dispatcher(func_table)?);
        rvec.append(&mut self.else_proc.sedgen_return_dispatcher(func_table)?);
        Ok(rvec)
    }
}

/// 引数とローカル変数のスタックフレーム内で占有する
trait SetLocalc {
    fn set_localc(&mut self, localc: usize);
}

impl SetLocalc for SoilIR {
    fn set_localc(&mut self, localc: usize) {
        for j in &mut **self {
            if let SoilIRInstruction::Call(call_func) = j {
                call_func.set_localc(localc);
            } else if let SoilIRInstruction::IfProc(if_proc) = j {
                if_proc.set_localc(localc);
            }
        }
    }
}

impl SetLocalc for CallFunc {
    /// この関数を呼び出しているスコープにおいて
    /// どれだけのローカル変数が使用されているかをセットする
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


/// |... ArgVal ...|... LocalVal...|[... stack zone ...]
///  <---------fixed size -------->| <--  flex size -->
/// 
pub trait ResolvePopAndSetProc {
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
//                                 trait end
// =========================================================================================

/// func_tableから名前の一致する関数を探し出す
fn find_function_definition_by_name<'a>(
    name: &str,
    func_table: &'a [FuncDef],
) -> Result<&'a FuncDef, CompileErr> {
    if let Some(func_def) = func_table.iter().find(|f| f.name == name) {
        Ok(func_def)
    } else {
        Err(CompileErr::UndefinedFunction(name.to_string()))
    }
}

// ------------------------- resolve instructions -----------------------------
// resolve_*_instructions functions
//
// function set that build executable sed script.

/// resolve SoilIRInstruction::Sed
/// Generates the sed script directly from SedCode.
fn resolve_sed_instruction(rstr: &mut String, sed: &SedCode, stack_size: usize) -> usize {
    rstr.push_str(&format!("{}\n", sed.0));
    stack_size
}

/// resolve SoilIRInstruction::Call
/// Appends the sed script equivalent to a function call to `rstr` and returns the stack size.
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

/// resolve SoilIRInstruction::Val
fn resolve_argval_instruction(rstr: &mut String, a: &ArgVal, stack_size: usize) -> usize {
    rstr.push_str(&resolve_stack_push_proc(stack_size, a.id));
    stack_size + 1
}

/// resolve SoilIRInstruction::Val
fn resolve_localval_instruction(
    rstr: &mut String,
    a: &LocalVal,
    func_def: &FuncDef,
    stack_size: usize,
) -> usize {
    rstr.push_str(&resolve_stack_push_proc(stack_size, func_def.argc + a.id));
    stack_size + 1
}

/// resolve SoilIRInstruction::Const
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
        format!(
            "\\({}\\)~[^\\~]*\\({}\\)\\(~[^\\~]*\\)",
            "~[^\\~]*".repeat(offset),
            "~[^\\~]*".repeat(stack_size - offset - 2)
        ),
        "\\1\\3\\2"
    )
}

/// resolve SoilIRInstruction::Set
fn resolve_set_instruction(
    rstr: &mut String,
    a: &dyn ResolvePopAndSetProc,
    func_def: &FuncDef,
    fixed_offset: usize,
    mut stack_size: usize,
) -> Result<usize, CompileErr> {
    // -- ja --
    // スタックの最上部を消費して、値をsetする
    // stack_sizeが
    // fixed_offsetだったらerror
    // -- en --
    // pop stack, and set value
    // if stack size less than fixed_offset size, this function return error

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

fn resolve_ret_instructions(
    rstr: &mut String,
    func_def: &FuncDef,
    stack_size: usize,
    fixed_offset: usize,
) -> Result<usize, CompileErr> {
    if fixed_offset + func_def.retc > stack_size {
        return Err(CompileErr::PoppingValueFromEmptyStack(format!(
            "@ {}",
            func_def.name
        )));
    }
    let arg_pattern: String = format!(
        "{}\\({}\\)",
        "~[^\\~]*".repeat(stack_size - func_def.retc),
        "~[^\\~]*".repeat(func_def.retc)
    );
    let arg_string = "\\1;";
    rstr.push_str(&format!("s/{}/{}/\n", arg_pattern, arg_string));
    // rstr.push_str("b return_dispatcher\n"); // 最後は必ずreturn
    let return_label = format!("return{}\n", func_def.id);
    rstr.push_str(&format!("b {}", return_label));
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
    stack_size -= 1;
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

/// Converts the intermediate representation into a sed script and appends it to the output buffer `rstr`.
fn resolve_instructions(
    rstr: &mut String,
    func_def: &FuncDef,
    proc_contents: &[SoilIRInstruction], // IR
    fixed_offset: usize,
    mut stack_size: usize,
    func_table: &[FuncDef],
) -> Result<usize, CompileErr> {
    stack_size += fixed_offset;
    for instruction in proc_contents {
        stack_size = match instruction {
            SoilIRInstruction::Sed(sed) => resolve_sed_instruction(rstr, sed, stack_size),
            SoilIRInstruction::Call(func_call) => {
                resolve_call_instruction(rstr, func_call, func_table, stack_size)?
            }
            SoilIRInstruction::Val(a) => match *a {
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
            SoilIRInstruction::ConstVal(a) => resolve_constval_instruction(rstr, a, stack_size),
            SoilIRInstruction::Set(a) => resolve_set_instruction(
                rstr,
                match *a {
                    Value::Local(index) => &func_def.local_list[index],
                    Value::Arg(index) => &func_def.arg_list[index],
                },
                func_def,
                fixed_offset,
                stack_size,
            )?,
            SoilIRInstruction::IfProc(a) => {
                resolve_if_instructions(rstr, a, func_def, stack_size, func_table)?
            }
            SoilIRInstruction::Ret => {
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

fn sedgen_func_def(
    func_def: &FuncDef,
    func_table: &[FuncDef],
    call_tree: &BTreeMap<String, Vec<ReturnAddrResolveCode>>,
) -> Result<String, CompileErr> {
    let is_entry = func_def.name == "entry";
    let fixed_offset = func_def.argc + func_def.localc;

    let mut rstr = if is_entry {
        // 引数も考慮する
        let pattern = format!("\\({}\\)", "~[^\\~]*".repeat(func_def.argc));
        let locals_out = (0..func_def.localc).map(|_| "~init").collect::<String>();
        format!("s/{}/\\1{}/\n", pattern, locals_out)
    } else {
        let pattern = format!("\\({}\\)", "~[^\\~]*".repeat(func_def.argc));
        let args_out = "\\1";
        let locals_out = (0..func_def.localc).map(|_| "~init").collect::<String>();
        format!(
            "
# def {}
:{}\n
s/:retlabel[0-9]\\+{}[^\\|]*|$/{}{}/
s/\\n\\(.*\\)/\\1/
",
            func_def.name,
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
        let return_label = format!("return{}", func_def.id);
        rstr.push_str(&format!(":{}\n", return_label));
        rstr.push_str("b done\n"); // entry return
    } else {
        // return addr resolve logic
        let return_label = format!("return{}", func_def.id);
        rstr.push_str(&format!(
            "
:{}
H
x
h
s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\1/
x
s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\2/
",
            return_label
        ));
        if let Some(codes) = call_tree.get(&func_def.name) {
            for return_addr_resolve_code in codes {
                rstr.push_str(&return_addr_resolve_code.code);
            }
        } else {
            return Err(CompileErr::Fatal);
        }
    }
    Ok(rstr)
}

// ifを表現するラベルに割り当てる名前を解決する関数
/// resolve if label name
fn resolve_if_label(proc_contents: &mut Vec<SoilIRInstruction>, mut min_id: usize) -> usize {
    for j in &mut *proc_contents {
        if let SoilIRInstruction::IfProc(a) = j {
            a.set_id(min_id);
            min_id += 1;
            min_id = resolve_if_label(&mut a.then_proc, min_id);
            min_id = resolve_if_label(&mut a.else_proc, min_id);
        }
    }
    min_id
}

struct ConsumedTable {
    func_label_id: usize,
    if_id: usize,
}

#[cfg(test)]
mod code_gen_test {
    use crate::{
        code_gen::*,
        embedded::{
            em_add, em_ends_with_zero, em_is_empty, em_mul, em_shift_left1, em_shift_right1,
        },
    };
    #[test]
    fn create_return_dispatcher_btree_map_test00() {
        let mut entry = FuncDef::new("entry", 0, 2, 1);
        let func_mul = em_mul();
        let func_add = em_add();
        let func_shift_left1 = em_shift_left1();
        let func_shift_right1 = em_shift_right1();
        let func_is_empty = em_is_empty();
        let func_ends_with_zero = em_ends_with_zero();

        entry.set_proc_contents(vec![
            SoilIRInstruction::Sed(SedCode("s/.*/~init~init/".to_string())), //ローカル変数の初期化
            SoilIRInstruction::ConstVal(ConstVal::new("101101110")),
            SoilIRInstruction::Set(Value::Local(0)),
            SoilIRInstruction::ConstVal(ConstVal::new("11101110111")),
            SoilIRInstruction::Set(Value::Local(1)),
            SoilIRInstruction::Val(Value::Local(0)), // L0
            SoilIRInstruction::Val(Value::Local(1)), // L0
            SoilIRInstruction::Call(CallFunc::new("mul")),
            SoilIRInstruction::Set(Value::Local(0)),
            SoilIRInstruction::Val(Value::Local(0)),
            SoilIRInstruction::Ret,
        ]);

        let func_def_table = vec![
            entry,
            func_mul,
            func_add,
            func_shift_left1,
            func_shift_right1,
            func_is_empty,
            func_ends_with_zero,
        ];
        if let Ok(tree) = create_return_dispatcher_btree_map(&func_def_table) {
            println!("{:#?}", tree);
        } else {
            println!("Something wrong");
        }
    }
}
