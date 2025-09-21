# recursive banana maker

# 関数の呼び出し
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
s/\(.*\)/:retlabel1-\1|/
H
b sub_routine0
:retlabel1
b return_dispatcher
# --- ここまでが関数の内容 ---

:return_dispatcher
x
# 4. 保存しておいた戻り先アドレスにジャンプする
/\n:retlabel0[^\|]*|$/ {
	# pop
	s/\(.*\)\n\(.*\)|$/\1/
	x
	b retlabel0 
}
/\n:retlabel1[^\|]*|$/ {
	# pop
	s/\(.*\)\n\(.*\)|$/\1/
	x
	b retlabel1 
}
/\n:done[^\|]*|$/ {
	b done
}

:done
