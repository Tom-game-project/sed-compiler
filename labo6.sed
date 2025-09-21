s/.*/world/

/hello/ {
	s/hello/Hello/
	t a
}

/world/ {
	s/world/World/
	t b
}


# branch a 
:a
bdone
# branch b
:b
bdone
:done
