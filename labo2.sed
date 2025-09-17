
# func_a関数の呼び出し
s/.*/:retlabel0-arg1|/
H
b func1
:retlabel0

# func_b関数の呼び出し
s/.*/:retlabel1-arg1-arg2|/
H
b func2
:retlabel1
b done
:func1
s/.*/helloworld/

# func_b関数の呼び出し
s/.*/:retlabel2-arg1-arg2|/
H
b func2
:retlabel2
b return_dispatcher
:func2
s/.*/helloworld/
b return_dispatcher

:return_dispatcher
x

/\n:retlabel0[^\|]*|$/ {
        s/\(.*\)\n\(.*\)|$/\1/
        x
        b retlabel0
}

/\n:retlabel1[^\|]*|$/ {
        s/\(.*\)\n\(.*\)|$/\1/
        x
        b retlabel1
}

/\n:retlabel2[^\|]*|$/ {
        s/\(.*\)\n\(.*\)|$/\1/
        x
        b retlabel2
}
:done
