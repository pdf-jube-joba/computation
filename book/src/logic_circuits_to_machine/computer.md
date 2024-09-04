# CPU を作る
いろいろ考えてたらわからなくなってきた。
ちょっと特殊かも。
- レジスタについて
    - レジスタ：R0,R1,R2,R3 とアキュームレータ：Aとフラグ：Fとプログラムカウンタ：P
        - 汎用という言葉はおかしいかも
    - レジスタのサイズは program counter を除き 8 bit で、 program counter は任意有限 bit が入れられる。
    - フラグは carry と zero の判定に使い、 \(F[z], F[c] \in \{0,1\}\) でそれぞれを表す。
- メモリについて
    - 各セルが 8 bit の大きさをしていて、それが可算個並んでいる。
- アドレスの指定
    - 8 bit よりも大きい指定がほしいのでその対策をする。主な問題はどこまでがアドレスの指定なのかがわからないこと。
        - 例えば2bit のCPUがあってアドレスは単に2進数だったとすると、 1010101010...みたいな列があったときに最初の2bitからメモリの呼び出しをするコードとわかったとして、残りのどの部分までがアドレスの指定になっているのか（命令の終わりがどこまでなのか）がわからない。
    - "チャンク"とその長さの組を合わせ、長さが最大の時は次のチャンクもそのアドレス指定にくっついていることとする。
        - チャンクとは、初めの8bitと、それに続く長さ \(2^8 * s\) bit の部分のことである。ただし、 \(s\) は初めの8bitを2進数と考えて自然数に直したときの値（この値をチャンクの長さと呼ぶ。）
        - あるチャンクの長さが \(2^8\) だった場合には、次のbit列もチャンクとして解釈する。

- 命令セット
    - 一覧
        - ADD Rn Rm := A <- Rn + Rm
        - NOT Rn := A <- !Rn
        - AND Rn Rm := A <- Rn & Rm
        - CPY Rn Rm := Rn <- Rm
        - BCK Rn := Rn <- A
        - LDM Rn m := Rn <- [m]
        - SRM m := [m] <- A
        - JMP m := P <- m
        - BRZ m := if F[z] then P <- m else none
        - BRC m := if F[c] then P <- m else none
        - NOP := none
        - HLT := none
    - P は 「JMP,BRZ,BRC, で P への代入が起こった場合」と HLT を除いて、+1される。
    - フラグは、 ADD,NOT,AND の命令が行われた際に更新する。 
- 後でデコーダなどが簡単になるように命令形式の設計を行うが、アドレスの指定以外の部分は1Byteに収まるようにする。

## もうちょっと動かしやすいように書く
- Bit と Byte と toN: Byte -> Nat はあるものとする。
- toN: List Byte -> Nat :=
    - [] => 0
    - [x :: xs] => toN(x) + (2^8) * toN(xs) 
- MemoryFragment := f: \(\Nat\) -> Byte s.t. f(m) = 0 if m large
- fetch-chunk: MemoryFragment -> List Byte := f =>
    - let s := toN(f(0))
    - let m: List Byte := (i in 0..s |-> f(i + 1)) // List の形をしていないけどまあわかるでしょ
    - (s, m)
- fetch-address: MemoryFragment -> List Byte := f =>
    - let (s, m) = fetch-chunk f
    - if s == 11111111 then [m :: fetch-address (i |-> f(i + 2^8))] else [m]

Opcode の詳細はあとで決めるのでこんな感じにしておく。
- Opcode := f: Byte -> {ADD Rn Rm, NOT Rn, AND Rn Rm, CPY Rn (Rn/A), LDM Rn, SRM Rn/A, JMP, BRC, BRZ, NOP, HLT} s.t. f(00000000) = NOP

- State := 

# 実装について