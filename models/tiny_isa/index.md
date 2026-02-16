- 扱う対象を自然数 ~= ほとんど 0-fill された無限のビット列にする。
- メモリセルに入れるのは自然数
- メモリは自然数で添え字づけ。
- レジスタは有限個で自然数を入れる
- 命令は最初の n bit で（どのレジスタを操作するかも含めて） opcode みたいにして、 operand は残りのビット列（自然数）にする。
    - 「どのレジスタを使うか」が有限で扱えるので、引数は残りのビット列全部にできる。

フラグがあった方がいい。

データの転送
- ldi rd ..imm : rd <- imm
- mov rd rs : rd <- rs
- ld rd rb : rd <- M[ rb ]
- st rs rb : M [ rb ] <- rs

算術
- add rd rs : rd <- rd + rs
- sub rd rs : rd <- rd - rs ... 0 で飽和する

比較
- eq rd rs: rd == rs => eq-flag
- lt rd rs: rd < rs => lt-flag

条件分岐
- jmp rb : pc <- rb
- jeq rb : pc <- rb ... if eq-flag 
- jlt rb : pc <- rb ... if lt-flag

その他
- nop: 何もせずに次に行く
- halt: 停止する
- reset: flag を reset する
- readpc rd: rd <- pc

レジスタは4つとして、
最初の 8 bit でレジスタ指定を含めた部分を記述する。
残りの bit を全部自然数にして引数と考える。

hdl に落とすのは、自然数の入ったメモリの転送さえクリアできれば大丈夫そう。
これは、n bit のチャンクで扱う？
