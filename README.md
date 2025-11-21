# Sed Compiler

You can transpilate the custom languages "Soil" to sed.

example source

[soil src](soil/example.soil) -> [out sed](sed/c_example.sed)

# Soil

# Run Sed Program

```sed
touch in
sed "$(git show master:sed/c_example.sed)" in
```
