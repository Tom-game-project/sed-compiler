# Sed Compiler

sed as a runtime.

You can transpilate the custom languages "Soil" to sed.

example source

[soil src](sed-compiler/soil/basic_operations.soil) -> [out sed](sed-compiler/sed/basic_operations.sed)

# Soil

# Run Sed Program

[sed-compiler/soil/basic_operations.soil](sed-compiler/soil/basic_operations.soil)

```
...

pub fn entry a:bit32, b:bit32 -> bit32, bit32, bit32 {
    let r1 = 0;
    let r2 = 0;

    r1 = gcd(b, a);
    r2 = gcd(a, b);

    return 0, r1, r2;
}
```

```sh
# Passing arguments to the entry function
# in sed-compiler/soil/basic_operations.soil case
# To pass a 32-bit integer, the number must be converted to binary.

# args must be delimited by "~"
#     |<--         32bit         -- >| |<--         32bit         -- >|
echo ~00000111010110111100110100010101~00111010110111100110100010110001 | sed -f sed-compiler/sed/basic_operations.sed
# There is a slight lag before the result is displayed.

```


```sh
touch in
sed "$(git show master:sed/c_example.sed)" in
```
