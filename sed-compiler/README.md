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
+---------:done------------+
:done
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
