# sed-compiler

sed-compiler: A compiler that targets sed as a runtime environment.

It compiles Soil code into raw sed scripts, managing stack frames and recursion using sed's pattern and hold spaces.

## Structure of the Generated Sed Script

### overview

```
+--------------------------+
| entry function section   |
| junp to :done            |
+--------------------------+
| def func0                |
| -(return addr resolver)- |
+--------------------------+
| def func1                |
| -(return addr resolver)- |
+--------------------------+
| def func2                |
| -(return addr resolver)- |
+--------------------------+
            :
|           :              |
+-------- :done -----------+
```

### function define in Sed

```
|           :              |
+--------------------------+
| def funcn                |
| -(return addr resolver)- |
+--------------------------+
|           :              |

            |
       (expands to)
            |
            v

|                           :                           |
+-------------------------------------------------------+
| :func{n}                                              |
|                                                       |
|                 --(function contents)--               |
|                                                       |
| :retlabel0{return_label}                              |
|                                                       |
| H                                                     |
| x                                                     |
| h                                                     |
| s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\1/   |
| x                                                     |
| s/^\\(.*\\)\\(\\n:retlabel[0-9]\\+[^|]*|.*\\)$/\\2/   |
|                                                       |
|               --(return addr resolver)--              | <- (Restores stack frame & jumps back to the call sites)
|                                                       |
| # For `return addr resolver` implementation details,  |
| # see src/code_gen.rs:[trait SedgenReturnDispatcher]  |
|                                                       |
+-------------------------------------------------------+
|                           :                           |
```

## Internal Intermediate Representation(SoilIR)

SoilIR is a stack-oriented intermediate representation, similar to WebAssembly or Forth.

### SoilIR example

function definition example

```rust
let mut func = FuncDef::new("func_name",
    0, // argc
    2, // localc
    1, // retc
);

func.set_proc_contents(vec![
    SoilIRInstruction::ConstVal(ConstVal::new("101101110")),
    SoilIRInstruction::Set(Value::Local(0)),
    SoilIRInstruction::ConstVal(ConstVal::new("11101110111")),
    SoilIRInstruction::Set(Value::Local(1)),
    SoilIRInstruction::Val(Value::Local(0)),
    SoilIRInstruction::Val(Value::Local(1)),
    SoilIRInstruction::Call(CallFunc::new("mul")),
    SoilIRInstruction::Set(Value::Local(0)),
    SoilIRInstruction::Val(Value::Local(0)),
    SoilIRInstruction::Ret,
]);
```

### SoilIRInstruction

Enum: src/code_gen.rs:`enum SoilIRInstruction`

```rust
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
```

#### ConstVal

The ConstVal instructions are used to declare numbers.

```rust
SoilIRInstruction::ConstVal(ConstVal::new("101101110")), // load the data "101101110" onto the stack
SoilIRInstruction::Call(CallFunc::new("shift_right1")),
```

[Wasm const](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Numeric/const)

#### Val

The Val instruction loads the value of a local variable (or argument) onto the stack.

```rust
SoilIRInstruction::Val(Value::Arg(0)), // load the value of arg $0 variable onto the stack
SoilIRInstruction::Call(CallFunc::new("shift_right1")),
```

```rust
SoilIRInstruction::Val(Value::Local(0)), // load the value of local $0 variable onto the stack
SoilIRInstruction::Call(CallFunc::new("shift_right1")),
```

[Wasm local.get](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Variables/local.get)

#### Set

The Set instruction loads the 

```rust
SoilIRInstruction::ConstVal(ConstVal::new("101101110")), // load the data "101101110" onto the stack
SoilIRInstruction::Set(Value::Local(0)),                 // store the data "101101110" in the variable local $0
```

[Wasm set](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Variables/local.set)

#### Call

Consumes the top $N$ items from the stack (where $N$ is the number of arguments) and pushes the result.

```rust
SoilIRInstruction::Val(Value::Local(0)), // load the value of local $0 variable onto the stack
SoilIRInstruction::Val(Value::Local(1)), // load the value of local $1 variable onto the stack
SoilIRInstruction::Call(CallFunc::new("mul")), // consume the top of the stack as arguments and return result.
```

[Wasm call](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/call)

#### IfProc

```rust
SoilIRInstruction::IfProc(IfProc::new(
    vec![
        // then
        ...
    ],
    vec![
        // else
        ...
    ],
)),
```

[Wasm if..else](https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Control_flow/if...else)

