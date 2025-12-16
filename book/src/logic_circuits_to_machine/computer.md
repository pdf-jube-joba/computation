# CPU を作る
いっそ論理回路側で頑張った方がいいと思うので、こっちの CPU は書きやすくしたい。
- メモリセルを、ほとんど 0-fill された無限のビット列にする。
- メモリは自然数で添え字づけ。
- レジスタは有限個で、ほとんど 0-fill された無限のビット列を入れる。
- 命令は最初の n bit で（どのレジスタを操作するかも含めて） opcode みたいにして、 operand は残りのビット列（自然数）にする。
    - 「どのレジスタを使うか」が有限で扱えるので、引数は残りのビット列全部にできる。

途中まで

データの転送
- load-immidiate rd imm: rd <- imm
- load-immidiate-offset rd rs off: rd <- M[rs + off]
- load rd addr : rd <- M[ addr ]
- store rs addr : M [ addr ] <- rs
- store-offset rd rs offset : M[ rd + off ] <- rs
- mov rd rs : レジスタ間
算術
- add rd rs
