# Soil Language

```
pub fn name_hello a:num, b:num, c:string -> bool {
    let a = 42 + 1 + hello;
    let b = a * 2;
    a = a + b;
    let b = a * 2;
    b = c + 1;
}

fn name_world a:num, b:num, c:string -> bool {
    let a = 42 + 1 + hello;
    let b = (a + 1) * 2;
    a = a + b;
    let b = a * 2;
    if a == 1 {
        let b = a * 2;
        b
    } else {
        let b = a * 2;
        b + 1
    }
    b = c + 1
}

pub fn mul a:bit32, b:bit32 -> bit32 {
    if is_empty(b) {
        bit32(0)
    } else {
        if ends_with_zero(b) {
            mul(shift_left1(a), shift_right1(b))
        } else {
            add(mul(shift_left1(a), shift_right1(b)))
        }
    }
}
```

```
pub fn is_empty a:bit32 -> bool {
    sed$(
        "s/~$/T/  ",
        "s/~.*$/F/",
        "s/T/~1;/ ",
        "s/F/~0;/ ",
    )$
}

fn ends_with_zero a:bit32 -> bool {
    sed${
        "s/.*0$/~1;/ ",
        "s/.*1$/~0;/ ",
    }$
}

pub fn mul a:bit32, b:bit32 -> bit32 {
    if is_empty(b) {
        bit32(0)
    } else {
        if ends_with_zero(b) {
            mul(shift_left1(a), shift_right1(b))
        } else {
            add(mul(shift_left1(a), shift_right1(b)))
        }
    }
}
```

