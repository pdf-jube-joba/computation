## プロセッサとメモリ
自然数を扱う単純なプロセッサを考える。

- 扱う対象を自然数 ~= ほとんど 0-fill された無限のビット列にする。
    - メモリセルに入れるのは自然数
    - メモリは自然数で添え字づけ。
    - レジスタは有限個で自然数を入れる。
- 命令は最初の n bit で（どのレジスタを操作するかも含めて） opcode みたいにして、 operand は残りのビット列（自然数）にする。
    - 「どのレジスタを使うか」が有限で扱えるので、引数は残りのビット列全部にできる。

フラグがあった方がいい。
レジスタは4つとして、 \(2\) bit で表す。
メモリに入っているのは無限ビット列だが、命令の判断に使うのは主に最初の \(8\) ビットで、どのレジスタを使うかまでここで書くことにする。
残りの bit を全部自然数にして引数と考える。

機械語の場合は encode/decode が必要になるのが面白い。
（アセンブラ単体ではあまり考えなくてもいいように思える。）

| 種類 | 命令 | encode | 意味 |
| ---- | ---- | ------ | ---- |
| その他 | `nop`       | `0b0000_0000 ??..` | 何もせずに次に行く |
| その他 | `halt`      | `0b0000_1111 ??..` | 停止する |
| その他 | `rst`       | `0b0001_0000 ??..` | flag を reset する |
| メモリ | `ld rd rb`  | `0b0010_ddss ??..` | \(rd \leftarrow M[ rb ]\) |
| メモリ | `st rs rb`  | `0b0011_ddss ??..` | \(M [ rb ] \leftarrow rs\) |
| 即値   | `ldi rd ii` | `0b0100_00dd ii..` | \(rd \leftarrow imm\), imm は `ii..` からとる。 |
| 転送   | `rdpc rd`   | `0b0101_00dd ??..` | \(rd \leftarrow pc\) |
| 転送   | `mov rd rs` | `0b0110_ddss ??..` | \(rd \leftarrow rs\) |
| 算術   | `add rd rs` | `0b1000_ddss ??..` | \(rd \leftarrow rd + rs\) |
| 算術   | `sub rd rs` | `0b1001_ddss ??..` | \(rd \leftarrow rd - rs\) ... \(0\) で飽和する |
| 比較   | `eq rd rs`  | `0b1010_ddss ??..` | \(rd = rs \Rightarrow \mathtt{eq flag}\) |
| 比較   | `lt rd rs`  | `0b1011_ddss ??..` | \(rd < rs \Rightarrow \mathtt{lt flag}\) |
| 分岐   | `jmp rb`    | `0b1100_ddss ??..` | \(pc \leftarrow rb\) |
| 分岐   | `jeq rb`    | `0b1101_ddss ??..` | \(pc \leftarrow rb\) ... if \(\mathtt{eq flag}\) |
| 分岐   | `jlt rb`    | `0b1110_ddss ??..` | \(pc \leftarrow rb\) ... if \(\mathtt{lt flag}\) |

hdl に落とすのは、自然数の入ったメモリの転送さえクリアできれば大丈夫そう。
自己書き換えができるとうれしいから、この"機械語の" `Machine` 実装では
コード領域とデータ領域の論理アドレス空間は同じとする。
これはつまり、 Neumann 型になっているということ。

- 入力（ `AInput` ）： 4 つのレジスタの初期値とメモリの初期値を受け取る。
    - 受け取ったコードを encode したのちに、入力で受け取ったメモリとくっつけて単一のメモリ空間にする。
- `RInput` はなし、しいて言うなら、外部からの割り込みとかがあればいいのかもしれないが、ここでは扱わない。
- 出力（ `Output` ）： 4 つのレジスタとメモリの値を出力する。

## アセンブリ言語
普通のアセンブリ言語。
上記の機械語の mnemonic の記述に加えて、以下の機能を付ける。
- `.text` と `.data` セクションを入れて、ラベルを入れる。
    - `.data` については、登場した順に番号を振り、サイズとそこに代入される値を順に書く。
- コード用のラベルとデータ用のラベルは分けて、レジスタは `%r0, %r1, %r2, %r3` として書く。
    - `ldi` だけ即値が必要になるので、これは、自然数かラベルでの入力とする。
- 特別な値の計算：
    - `@DATA_LEN`, `@CODE_LEN`... それぞれのセクションの長さを記述する。
    - `@THIS` これが書かれている行のアドレスを指す
- アセンブラへの指令や命令（ Directive ）：
    - 特殊な定数値の定義 ... これはラベルと同じ扱いでよい。
    - マクロによる疑似命令

アセンブリ言語単体での意味論としては、コード領域とデータ領域はアドレス空間の異なる領域にあるとしていい。
つまり、 **機械語の単体とは異なり**ハーバード型である。

> [!Note] アドレス空間のとらえ方が機械語側と異なるのはやっていいのか？
> 分けると、アセンブリの `Machine` の方では
> 「コード領域を `Vec<Number>` でもって encode/decode を追加する」
> みたいなことをわざわざやらずに、素直に `Vec<Stmt>` で持てる。
> ただしその場合、 immidiate value で計算したときにどこを指しているかが異なる。

## 仮想レジスタを入れて、 load/store をなくす。
用語の整理：
- 値：計算が生成・消費する対象、単なるデータ。
- アドレス値：値のうち、ある場所を指し示す参照と解釈されるもの。
    - これの deref が場所
- 場所・メモリセル：書き込み先として指定できる対象。
    - これの ref がアドレス値
- 仮想レジスタ：変数と思い、値を束縛するための対象。値の記述なので、場所ではない。

アドレスによる場所へのアクセスを一般化したら、ラベルを生でメモリセルみたいに使えない文法になった。
それと、 control flow graph に寄せるためには、 terminator がブロックの最後に続くようにした方がいいらしい。
ここでは、分岐命令を全部書いて最後に goto にした。

\(\begin{aligned}
\NT{label}  &\defeq \T{@} \sp \NT{string} \\
\NT{var}    &\defeq \T{\%} \sp \NT{string} \\
\NT{addr}   &\defeq \NT{var} \sp | \sp \NT{label} \sp | \sp \T{\&} \NT{place} \\
\NT{place}  &\defeq \T{[} \NT{addr} \T{]} \\
\NT{value}  &\defeq \NT{var} \sp | \sp \NT{imm} \sp | \sp \T{*} \sp \NT{place} \\

\NT{cond}   &\defeq (\NT{value} \sp \T{<} \NT{value}) \sp | \sp (\NT{value} \sp \T{=} \NT{value}) \\

\NT{stmt}   &\defeq ( \\
    &| \sp \NT{var}     \sp \T{:=} \sp \NT{value} \sp \NT{op} \sp \NT{value} \\
    &| \sp \NT{place}   \sp \T{:=} \sp (\NT{var} \sp | \sp \NT{value}) \\
    ) \T{;} \\
\NT{cont}   &\defeq ( \\
    &| \sp \T{goto} \sp \NT{addr} \sp \T{;} \\
    &| \sp \T{if}   \sp \NT{cond} \sp \T{then} \sp \NT{addr} \sp \T{;} \sp \NT{cont} \\
    )
\NT{block}  &\defeq \NT{label} \T{\{} \NT{stmt}* \sp \NT{cont} \T{\}} \\
\NT{static} &\defeq \NT{label} \sp \NT{imm} \\
\end{aligned}\)

## メモリの操作を入れる


