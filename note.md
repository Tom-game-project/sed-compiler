# ローカル変数初期化の最適化

```
(none initialized local memory) ~init~init
const A
set 
const B
set
```
 
-> 

```
~A~B
```

上のようにコンパイル時に判明する

tee

https://developer.mozilla.org/en-US/docs/WebAssembly/Reference/Variables/Local_tee
