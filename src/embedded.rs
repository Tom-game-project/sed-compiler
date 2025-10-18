use crate::code_gen::*;

pub fn em_shift_left1() -> FuncDef {
    let mut func_shift_left1 = FuncDef::new("shift_left1", 1, 0, 1);
    func_shift_left1.set_proc_contents(vec![SedInstruction::Sed(SedCode(
        "s/\\(~[01]*\\)/\\10;/".to_string(),
    ))]);
    func_shift_left1
}

pub fn em_shift_right1() -> FuncDef {
    let mut func_shift_right1 = FuncDef::new("shift_right1", 1, 0, 1);
    func_shift_right1.set_proc_contents(vec![SedInstruction::Sed(SedCode(
        "s/\\(~[01]*\\)[01]/\\1;/".to_string(),
    ))]);
    func_shift_right1
}

pub fn em_is_empty() -> FuncDef {
    let mut func_is_empty = FuncDef::new("is_empty", 1, 0, 1);
    func_is_empty.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("s/~$/T/   ".to_string())),
        SedInstruction::Sed(SedCode("s/~.*$/F/ ".to_string())),
        SedInstruction::Sed(SedCode("s/T/~1;/  ".to_string())),
        SedInstruction::Sed(SedCode("s/F/~0;/  ".to_string())),
    ]);
    func_is_empty
}

pub fn em_ends_with_zero() -> FuncDef {
    let mut func_ends_with_zero = FuncDef::new("ends_with_zero", 1, 0, 1);
    func_ends_with_zero.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("s/.*0$/~1;/ ".to_string())),
        SedInstruction::Sed(SedCode("s/.*1$/~0;/ ".to_string())),
    ]);
    func_ends_with_zero
}

/// you need to define
/// - shift_left1
/// - is_empty
/// - ends_with_zero
/// - shift_left1
/// - shift_right1
pub fn em_mul() -> FuncDef {
    let mut func_mul = FuncDef::new("mul", 2, 1, 1);
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
    func_mul
}

pub fn em_add() -> FuncDef {
    let mut func_add = FuncDef::new("add", 2, 0, 1);
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
    func_add
}


/// you need to define
/// - twos_complement
/// - zero_padding32
/// - add
pub fn em_sub32() -> FuncDef {
    let mut func_sub32 = FuncDef::new("sub32", 2, 0, 1);
    func_sub32.set_proc_contents(vec![
        SedInstruction::Val(Value::Arg(0)),
        SedInstruction::Call(CallFunc::new("zero_padding32")),
        SedInstruction::Val(Value::Arg(1)),
        SedInstruction::Call(CallFunc::new("zero_padding32")),
        SedInstruction::Call(CallFunc::new("twos_complement")),
        SedInstruction::Call(CallFunc::new("add")),
        SedInstruction::Call(CallFunc::new("zero_padding32")),
        SedInstruction::Ret,
    ]);
    func_sub32
}

pub fn em_twos_complement() ->FuncDef {
    let mut twos_complement = FuncDef::new("twos_complement", 1, 0, 1);

    twos_complement.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("
s/~\\([^\\~]*\\)/\\1/
y/01/10/
s/$/+/
:add_one_loop
s/0+$/1/
t add_one_done
s/1+$/+0/
b add_one_loop
:add_one_done
s/^\\+/1/

s/\\(.*\\)/~\\1;/
".to_string())),
    ]);
    twos_complement
}

pub fn em_zero_padding32() -> FuncDef {
    let mut func_zero_padding32 = FuncDef::new("zero_padding32", 1, 0, 1);

    func_zero_padding32.set_proc_contents(vec![
        SedInstruction::Sed(SedCode("
s/~\\([^\\~]*\\)/\\1/
s/^/00000000000000000000000000000000/
s/.*\\(................................\\)$/~\\1;/
".to_string())),
    ]);
    func_zero_padding32
}
