# s/.*/\n:retlabel0~hello~world~Tom~hello~Tom|\n:retlabel1~world~Tom~hello~world~Tom~hello|\n~worldTom;/
# s/.*\n:retlabel[0-9]\+~\([^|]*\)|\n~\([^;]*\);$/\1:::\2/

s/.*/\n:retlabel0~hello~world~Tom~hello~Tom|\n:retlabel1~world~Tom~hello~world~Tom~hello|\n~worldTom;/

ba
:a
h
s/^\(.*\)\(\n:retlabel[0-9]\+[^|]*|.*\)$/\1/
x
s/^\(.*\)\(\n:retlabel[0-9]\+[^|]*|.*\)$/\2/


s/^\n:retlabel0~[^\~]*~[^\~]*~[^\~]*~\([^\~]*\)~\([^\~]*\)~\([^\|]*\)|\n~\([^\~;]*\);$/~\1~\2~\3~\4/
t retlabel0
s/^\n:retlabel1~[^\~]*~[^\~]*~[^\~]*~\([^\~]*\)~\([^\~]*\)~\([^\|]*\)|\n~\([^\~;]*\);$/~\1~\2~\3~\4/
t retlabel1
s/^\n:retlabel2~[^\~]*~[^\~]*~[^\~]*~\([^\~]*\)~\([^\~]*\)~\([^\|]*\)|\n~\([^\~;]*\);$/~\1~\2~\3~\4/
t retlabel2

:retlabel0
bdone
:retlabel1
bdone
:retlabel2
bdone

:done
