# hello world

# 関数の呼び出し
s/.*/:retlabel0-helloworld|/
H
b sub_routine0
:retlabel0
b done


# 関数の内容
:sub_routine0
# 引数を取り出す
g
s/\(.*\)-\(.*\)|$/\2/
p
b return_dispatcher


:return_dispatcher
# pop
x                        
h
s/\(.*\)\n\(.*\)|$/\2/  
x
s/\(.*\)\n\(.*\)|$/\1/
x

# 4. 保存しておいた戻り先アドレスにジャンプする
/:retlabel0.*/ {
	b retlabel0 
}
#/^:retlabel1$/ b 
/:done/ b done

:done
