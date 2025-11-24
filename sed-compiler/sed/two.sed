y/10/01/

s/\([01]*\)/1;\1;/

:two_complement_loop
s/1;\([01]*\)1;\([01]*\)/1;\1;0\2/
ttwo_complement_loop
s/0;\([01]*\)1;\([01]*\)/0;\1;1\2/
ttwo_complement_loop
s/1;\([01]*\)0;\([01]*\)/0;\1;1\2/
ttwo_complement_loop
s/0;\([01]*\)0;\([01]*\)/0;\1;0\2/
ttwo_complement_loop

s/[01];;\([01]*\)/~\1;/
