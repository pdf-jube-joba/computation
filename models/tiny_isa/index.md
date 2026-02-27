## プロセッサとメモリ
自然数を扱う単純なプロセッサを考える。

- 扱う対象を自然数 ~= ほとんど 0-fill された無限のビット列にする。
    - メモリセルに入れるのは自然数 ... `Number` であり、これをビット列にする。
    - メモリは自然数で添え字づけ。
    - レジスタは有限個で自然数を入れる。
- 命令は最初の n bit で（どのレジスタを操作するかも含めて） opcode みたいにして、 operand は残りのビット列（自然数）にする。
    - 「どのレジスタを使うか」が有限で扱えるので、引数は残りのビット列全部にできる。

機械語の場合は encode/decode が必要になるのが面白い。
（アセンブラ単体ではあまり考えなくてもいいように思える。）

以下はもうちょっとちゃんとした仕様
- 16 bit 固定長が基本、即値があるときだけ後ろからとる。
- レジスタは \(8\) つとして、 \(3\) bit で表す。
- \(0\) 番目のレジスタは program counter とする... `mov` や `add` はジャンプ命令になる。
- 比較命令時には flag が格納される。
- flag を表す `f` が入っている命令なら、 `f` が立っているときには flag が真のときだけ命令を実行する
- 減算は \(0\) で飽和する。

| 種類 | 命令 | encode | 意味 |
| ---- | ---- | ------ | ---- |
| その他 | `nop`       | `0b0000_0000_00_000_000 ??..` | 何もせずに次に行く |
| その他 | `halt`      | `0b0000_0000_01_000_000 ??..` | 停止する |
| その他 | `rst`       | `0b0000_0000_00_111_111 ??..` | flag を reset する |
| メモリ | `ld rd rb`  | `0b0000_0001_00_ddd_bbb ??..` | \(rd \leftarrow M[ rb ]\) |
| メモリ | `st rs rb`  | `0b0000_0010_00_sss_bbb ??..` | \(M [ rb ] \leftarrow rs\) |
| 即値   | `ldi rd`    | `0b0000_1000_1f_ddd_000 ii..` | \(rd \leftarrow im\), im は `ii..` からとる。 |
| 転送   | `mov rd rs` | `0b0000_1000_0f_ddd_sss ??..` | \(rd \leftarrow rs\)      |
| 算術   | `add rd rs` | `0b0000_0100_0f_ddd_sss ??..` | \(rd \leftarrow rd + rs\) |
| 算術   | `sub rd rs` | `0b0000_0101_0f_ddd_sss ??..` | \(rd \leftarrow rd - rs\) |
| 算術   | `addi rd`   | `0b0000_0100_1f_ddd_000 ii..` | \(rd \leftarrow rd + im\) |
| 算術   | `subi rd`   | `0b0000_0101_1f_ddd_000 ii..` | \(rd \leftarrow rd - im\) |
| 比較   | `eq rd rs`  | `0b0001_0001_00_ddd_sss ??..` | \(rd = rs \Rightarrow \mathtt{flag}\) |
| 比較   | `lt rd rs`  | `0b0001_0010_00_ddd_sss ??..` | \(rd < rs \Rightarrow \mathtt{flag}\) |
| 比較   | `gt rd rs`  | `0b0001_0100_00_ddd_sss ??..` | \(rd > rs \Rightarrow \mathtt{flag}\) |

なんかいろいろ考えてビットを配置したが、HDLを触ったことがないので机上の空論になっている。

hdl に落とすのは、自然数の入ったメモリの転送さえクリアできれば大丈夫そう。
メモリセルは無限に続くが有限個を除いて \(0\) のセルになっているから、「これ以降は \(0\) という目印」を付属させるようにすればいい。

自己書き換えができるとうれしいから、この"機械語の" `Machine` 実装では
コード領域とデータ領域の論理アドレス空間は同じとする。
これはつまり、 Neumann 型になっているということ。

- コード（ `Code = Vec<Number>`）：自然数列になる。
- 入力（ `AInput = ([Number; 8], Vec<Number>)` ）： 8 つのレジスタの初期値とメモリの初期値を受け取る。
    - 受け取ったコードを encode したのちに、入力で受け取ったメモリとくっつけて単一のメモリ空間にする。
- `RInput` はなし、しいて言うなら、外部からの割り込みとかがあればいいのかもしれないが、ここでは扱わない。
- 出力（ `FOutput = ([Number; 8], Vec<Number>)` ）： 8 つのレジスタとメモリの値を出力する。
- `r0` （プログラムカウンタ―）については、これを更新する命令以外は実行後に `+1` する。
- decode できないものは `step` 時に `Err` を返す。

> [!Note] \(8\) ビットに戻したい...
> レジスタが \(4\) で \(0\) 番目が pc ならこんな感じ？
> | 種類 | 命令 | encode | 意味 |
> | ---- | ---- | ------ | ---- |
> | その他 | `nop`       | `0b000_0_00_00 ??..` | 何もせずに次に行く |
> | その他 | `halt`      | `0b000_0_00_01 ??..` | 停止する |
> | 即値   | `ldi rd`    | `0b001_0_dd_00 ii..` | \(rd \leftarrow im\), im は `ii..` からとる。 |
> | 比較   | `lt rd rs`  | `0b000_1_dd_ss ??..` | \(rd < rs \Rightarrow \mathtt{flag}\) |
> | メモリ | `ld rd rb`  | `0b010_0_dd_bb ??..` | \(rd \leftarrow M[ rb ]\) |
> | メモリ | `st rs rb`  | `0b011_0_ss_bb ??..` | \(M [ rb ] \leftarrow rs\) |
> | 転送   | `mov rd rs` | `0b100_f_dd_ss ??..` | \(rd \leftarrow rs\)      |
> | 算術   | `add rd rs` | `0b101_f_dd_ss ??..` | \(rd \leftarrow rd + rs\) |
> | 算術   | `sub rd rs` | `0b110_f_dd_ss ??..` | \(rd \leftarrow rd - rs\) |
> | 算術   | `subi rd`   | 廃止 | imm load して add から |
> | 算術   | `addi rd`   | 廃止 | imm load して sub から |
> | 比較   | `eq rd rs`  | 廃止 | lt 2 回やって分岐してないなら |
> | その他 | `rst`       | 廃止 | （0 < 0） をやる |
> これは小さすぎる。特に、まともに使えるレジスタが \(3\) ぐらいしかない。
> ただし、アセンブリの側でメモリを確保しておけば、廃止した命令がマクロで簡単に書ける。
> それと、メモリの最初の方をレジスタファイルとして扱うことを最初から述べておけばいい。

## アセンブリ言語
普通のアセンブリ言語。
アセンブリ言語単体での意味論としては、コード領域とデータ領域はアドレス空間の異なる領域にあるとしていい。
（アセンブリにより、 `.text` のあとに `.data` が来るが、それは保証しなくてよい。）

つまり、 **機械語の単体とは異なり**ハーバード型である。

上記の機械語の mnemonic の記述に加えて、以下の機能を付ける。
- `.text` と `.data` セクションのみ
    - 交互に書いてもいい。
- ラベルは `@` で始まる。
    - コード用のラベルとデータ用のラベルは分けて、
- レジスタは `%` で始まる。
- `.f` をつけたら、条件分岐を入れる。
    - `@DATA_LEN`, `@CODE_LEN`... それぞれのセクションの長さを記述する。
    - `@THIS` これが書かれている行のアドレスを指す
- `mov %r0 %r2` のようなジャンプ命令に対応するものは、 `jump %r2` とも書ける。
- アセンブラへの指令や命令（ Directive ）：
    - 特殊な定数値の定義
    - マクロによる疑似命令

\(\begin{aligned}
\NT{label}  &\defeq \T{@} \sp \NT{string} \\
\NT{reg}    &\defeq \T{\%} \sp {0 ..= 7} \\
\NT{imm}    &\defeq \T{\#} \sp \NT{number} \\
\NT{const}  &\defeq \NT{string} \\
\NT{value}  &\defeq \NT{imm} \sp | \sp \NT{label} \sp | \sp \NT{const} \\
\NT{inst}   &\defeq ( \\
    &| \sp \NT{mnemonic} \sp (\T{.f})? \sp \NT{reg}? \sp \NT{reg}? \sp \NT{value} \\
    ) \T{;} \\
\NT{directive} &\defeq ( \\
    &| \sp \T{.text} \\
    &| \sp \T{.data} \\
    &| \sp \T{.equ} \sp \NT{const} \sp \NT{imm}
    ) \\
\NT{program} &\defeq ( \\
    &| \sp \NT{directive} \\
    &| \sp \NT{label} \sp \T{:} \\
    &| \sp \NT{value} \\
\end{aligned}\)

以上のものはアセンブラ言語の"前処理前"の記述になっている。
意味論を定めるものは、 `Inst` の列とメモリとレジスタの組に対してとする。
（逆に言うと、ディレクリブによる命令自体はこれの意味論とは無関係とする。）

`trait Machine` の実装としては、以下の点を除いて `tiny_isa` と変わらない。
- アドレス空間がコード領域とデータ領域で別れている。
- ラベルや定数が即値にとれる。
- コード側は素直に `(code: Vec<Inst>, static: Vec<Memory>)` で持っておけばよい。

> [!Note] アドレス空間のとらえ方が機械語側と異なるのはやっていいのか？
> 分けると、アセンブリの `Machine` の方では
> 「コード領域を `Vec<Number>` でもって encode/decode を追加する」
> みたいなことをわざわざやらずに、素直に `Vec<Stmt>` で持てる。
> ただしその場合、 immidiate value で計算したときにどこを指しているかが異なる。

## 仮想レジスタと cfg
用語の整理：
- 値：計算が生成・消費する対象、単なるデータ。
- アドレス値：値のうち、ある場所を指し示す参照と解釈されるもの。
    - これの deref が場所
- 場所・メモリセル：書き込み先として指定できる対象。
    - これの ref がアドレス値
- 仮想レジスタ：変数と思い、値を束縛するための対象。値の記述なので、場所ではない。

アドレスによる場所へのアクセスを一般化したら、ラベルを生でメモリセルみたいに使えない文法になった。
それと、 control flow graph に寄せるためには、 terminator がブロックの最後に続くようにした方がいいらしい。
ここでは、最後の部分は分岐命令を連続で書いた後に最後に goto にした。

\(\begin{aligned}
\NT{label}  &\defeq \T{@} \sp \NT{string} \\
\NT{vreg}   &\defeq \T{\%v} \sp \NT{number} \\
\NT{addr}   &\defeq \NT{vreg} \sp | \sp \NT{label} \sp | \sp \NT{imm} \\
\NT{place}  &\defeq \T{[} \NT{addr} \T{]} \\
\NT{value}  &\defeq \NT{vreg} \sp | \sp \NT{imm} \sp | \sp \T{*} \sp \NT{place} \sp | \sp \NT{label} \\

\NT{cond}   &\defeq (\NT{value} \sp \NT{rel} \sp \NT{value})\\

\NT{stmt}   &\defeq ( \\
    &| \sp \NT{vreg}    \sp \T{:=} \sp \NT{value} \\
    &| \sp \NT{vreg}    \sp \T{:=} \sp \NT{value} \sp \NT{binop} \sp \NT{value} \\
    &| \sp \NT{place}   \sp \T{:=} \sp \NT{value} \\
    ) \T{;} \\

\NT{jump-if}    &\defeq \T{if}   \sp \NT{cond} \sp \T{then} \sp \NT{addr} \sp \T{;} \\
\NT{jump}       &\defeq \T{goto} \sp \NT{addr} \sp \T{;} \\
\NT{cont}   &\defeq \NT{jump-if}* \NT{jump} \\
\NT{block}  &\defeq \NT{label} \T{\{} \NT{stmt}* \sp \NT{cont} \T{\}} \\

\NT{static} &\defeq \NT{label} \sp \NT{imm} \\
\NT{program}    &\defeq \NT{static}* \NT{block}+
\end{aligned}\)

これの `AInput` と `FOutput` はどうするかが難しい。
とりあえず、isa と asm とは違って、メモリだけ受け取るのがよさそう
`AInput = FOutput = Vec<Number>` にする。
これを `static` で定義された領域のうしろに連結をする。

- \(\text{static}   := \NT{label} \to \N\) ... ラベルとメモリ番号の numbering
- \(\text{env}      := \NT{vreg} \to \N\) ... 仮想レジスタの値を表す環境
- \(\text{memEnv}   := \N \to \N\) ... メモリの状況を表す環境（アドレスからそこに入っている値への関数）。
- \(\text{eval-addr}: (\text{static}, \text{env}) \to \NT{addr} \to \N := \ldots\)
- \(\text{eval-value}: (\text{static}, \text{env}) \to \NT{value} \to \N := \ldots\)

まあやればできるでしょう。
ところで、型をいれて validation をするなら、 \(\NT{addr}\) を code 用とデータ用に分けて、レジスタも分ければいい？
AI が生成したものにはちょっと不満もあるが、まあこんなものでしょう。
