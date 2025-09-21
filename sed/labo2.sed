# l1 = hello
# l2 = world
s/.*/~hello~world/

# func_a関数の呼び出し
s/~\([^~]*\)~\([^~]*\)/:retlabel0~\1~\1~\2|/
H
b func1
:retlabel0
b done
:func1
g
s/:retlabel[0-9]\+~\([^\~]*\)[^\|]*|$/~\1/

# TODO 返り値の設定形式は非常に重要
s/\n~\(.*\)/~\1string~\1newstring;/
# ";"をいれてreturn
b return_dispatcher

:return_dispatcher
H
x

/\n:retlabel0[^\|]*|[^\;]*;$/ {
	h
	s/\(.*\)\n:retlabel0\([^\|]*\)|\n[^\;]*;$/\1/
	x
	s/.*\n:retlabel0~[^\-~|]*~\([^\~|]*\)~\([^\~|]*\)|\n~\([^\~;]*\)~\([^\~;]*\);$/~\1~\2~\3~\4/
	b retlabel0
}
:done
