enum Instruction <'a>{
    ArgVal(&'a ArgVal),
    LocalVal(&'a LocalVal),
    ConstVal(ConstVal),
    FuncCall(FuncCallVal),
}

struct ArgVal {
    id: usize
}
impl ArgVal {
    fn new(id: usize) -> Self{
        Self { id }
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


struct LocalVal {
    id: usize,
}
impl LocalVal {
    fn new(id: usize) -> Self {
        Self {
            id
        }
    }
}

/// function as value
struct FuncCallVal {
    func_name: String,
    argc: usize,
    retc: usize
}
impl FuncCallVal{
    fn new(func_name: &str, argc: usize, retc: usize) -> Self {
        Self { func_name: func_name.to_string(),  argc, retc}
    }
}


//enum ValResolveErr 
//{
//    A
//}
//
//
//fn valgen_assignment_code(a:&[LocalVal], b:&[LocalVal]) 
//    -> Result<Vec<(usize, usize)>, ValResolveErr>
//{
//    Err(ValResolveErr::A)
//}


// l1**2 + l2**2
//
// (+ (pow l1 2) (pow l2 2))
//
// 上のプログラムは以下のように記述できる
//
// ```
// local.get l1
// const 2
//
// call pow
//
// local.get l2
// const 2
//  
// call pow
//
// call add
// ```
pub fn val_test00() {
    let arg_vals: Vec<ArgVal> = vec![];
    let local_vals = vec![
        LocalVal::new(0), // L0
        LocalVal::new(1), // L1
    ];

    let instructions = vec![
        Instruction::LocalVal(&local_vals[0]), // L0
        Instruction::ConstVal(ConstVal::new("2")),
        Instruction::FuncCall(FuncCallVal::new("pow", 2, 1)),
        Instruction::LocalVal(&local_vals[1]), // L1
        Instruction::ConstVal(ConstVal::new("2")),
        Instruction::FuncCall(FuncCallVal::new("pow", 2, 1)),
        Instruction::FuncCall(FuncCallVal::new("add", 2, 1)),
    ];
}

/// TODO マシな関数名を考える
fn code_gen_from_instructions_list(
    arg_vals:Vec<ArgVal>,      // スコープ内の引数の個数
    local_vals: Vec<LocalVal>, // スコープ内のローカル変数の個数
    instructions: Vec<Instruction> // 命令列
) -> ()
{
    let arg_vals_len = arg_vals.len();
    let local_vals_len = local_vals.len();
    let fixed_offset = arg_vals_len + local_vals_len;

    let mut stack_size = fixed_offset;
    for i in instructions {
        match i {
            Instruction::ArgVal(a) => {
                let mut next_pattern = 
                    (1..stack_size + 1)
                        .map(|d| format!("~\\{}", d))
                        .collect::<Vec<String>>();
                next_pattern.push(format!("~\\{}", 
                        a.id 
                        + 1 // index が1から始まる
                )); // スタックに引数を積む
                let rstr = format!("s/{}/{}/",
                    "~\\([^\\~]*\\)".repeat(stack_size),
                    next_pattern.join("")
                );
                println!("{}", rstr);
                stack_size += 1;
            }
            Instruction::LocalVal(a) => {
                let mut next_pattern = 
                    (1..stack_size + 1)
                        .map(|d| format!("~\\{}", d))
                        .collect::<Vec<String>>();
                next_pattern.push(format!("~\\{}",
                        a.id 
                        + 1 // index が1から始まる
                        + arg_vals_len // 引数分のoffset
                )); // スタックにローカル変数を:world積む
                let rstr = format!("s/{}/{}/",
                    "~\\([^\\~]*\\)".repeat(stack_size),
                    next_pattern.join("")
                );
                println!("{}", rstr);
                stack_size += 1;
            }
            Instruction::ConstVal(a) => {
                let mut next_pattern = 
                (1..stack_size + 1)
                    .map(|d| format!("~\\{}", d))
                    .collect::<Vec<String>>();
                next_pattern.push(format!("~\\{}",
                        a.data
                )); // 定数をスタックに積む
                let rstr = format!("s/{}/{}/",
                    "~\\([^\\~]*\\)".repeat(stack_size),
                    next_pattern.join("")
                );
                println!("{}", rstr);
                stack_size += 1;
            }
            Instruction::FuncCall(funccall) => {
                
                stack_size -= funccall.argc;
                stack_size += funccall.argc;
            }
        }
    }

}

// l1**2 + l2**2
//
// (+ (pow l1 2) (pow l2 2))
//
// 上のプログラムは以下のように記述できる
//
// ```
// local.get l1
// const 2
//
// call pow
//
// local.get l2
// const 2
//  
// call pow
//
// call add
// ```
/*
pub fn val_test01() {
    let l1 = LocalVal::new(0);
    let l2 = LocalVal::new(1);

    let add_func00 = FuncCallVal::new(
        String::from("add_func"), 
        vec![Val::FuncCallVal(FuncCallVal::new("pow", args, rets))],
        vec![&Val::localval(&l1)]
    );
}
*/
