# hello world

# 関数の呼び出し
s/.*/:done|/
H
s/.*/:retlabel0-ba|/
H
b sub_routine0
:retlabel0
b done

# --- 関数の内容始まり ---
:sub_routine0
# 引数を取り出す
g
s/\(.*\)-\(.*\)|$/\2/

/^banana$/ {
	b return_dispatcher
}
/^bana$/ {
	s/^bana$/banana/
}
/^ba$/ {
	s/^ba$/bana/
}

# 関数の呼び出し
s/\(.*\)/:retlabel0-\1|/
H
b sub_routine0
:retlabel0

b return_dispatcher
# --- ここまでが関数の内容 ---

:return_dispatcher
# pop
# x                        
# h
# s/\(.*\)\n\(.*\)|$/\2/  
# x
x
# 4. 保存しておいた戻り先アドレスにジャンプする
/\n:retlabel0.*|$/ {
	s/\(.*\)\n\(.*\)|$/\1/
	x
	b retlabel0 
}
#/^:retlabel1$/ b 
/\n:done.*|$/ {
	b done
}

:done
