s/.*/hello/

# func_a関数の呼び出し
s/\(.*\)/:retlabel0-\1|/
H
b func1
:retlabel0
b done
:func1
g
s/:retlabel[0-9]\+-\([^\-]*\)|$/\1/

# func_b関数の呼び出し
s/\n\(.*\)/:retlabel1-\1-\1|/
H
b func2
:retlabel1
b return_dispatcher
:func2
s/:retlabel[0-9]\+-\([^\-]*\)-\([^\-]*\)|$/\1===\2/
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
:done
