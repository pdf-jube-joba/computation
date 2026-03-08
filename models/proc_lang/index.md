call/return で手続呼び出しを行う言語を考える。
- 引数と返り値あり。
  - いずれも複数でいい。
- 大域変数と局所変数あり。
  - global か自分の local で宣言された変数のみ使用できる。
  - 呼び出し元手続きの変数環境は見えないものとする。
- 値呼びにする。

\(\begin{aligned}

\NT{var}    &\defeq \NT{string} \\
\NT{name}   &\defeq \NT{string} \\
\NT{atom}   &\defeq \NT{var} | \NT{imm} \\

\NT{aexp}   &\defeq \NT{atom} \NT{binop} \NT{atom} \\
\NT{bexp}   &\defeq \NT{atom} \NT{binrel} \NT{atom} \\

\NT{var-cmm} &\defeq \syntaxmacro{comma-separated}{\NT{var}} \\

\NT{stmt} &\defeq \\
  &| \T{Nop}
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{string} \T{:=} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{\LSB} \NT{stmt} \T{\RSB} \\
  &| \NT{while} \NT{bexp} \T{\LSB} \NT{stmt} \T{\RSB} \\
  &| \T{call} \NT{name} \T{\LP} \NT{var-cmm} \T{\RP} \T{->} \NT{var-cmm} \\
  &| \T{return} \NT{var-cmm} \\

\NT{proc} &\defeq \NT{string} \T{LP} \NT{var-cmm} \T{\RP} \T{\LSB} \\
  &\T{local} \NT{var-cmm} \\
  &\NT{stmt} \\
  &\T{\RSB} \\

\NT{program} &\defeq \\
  &\NT{proc}+ \\
\end{aligned}\)

\(\T{;}\) は右結合で。

## 意味論（ small-step ）
call-stack が必要になるぐらいで、あとは `while_lang` と同じ。
引数位置に var が来て値渡しなので評価順序も引数評価用の環境もいらない。
`AInput` も `FOutput` も global env と考える。

- early return ができる。
- 名前の解決は終わっていることとする。（ `x = 10;` みたいなものの羅列。）
- 最後に Nop が来たら継続がなければそのまま return と考える。
- main と名前のついている proc から実行する。これの引数はないものとする？
- push するときに、 stmt の最後には return を入れておく。
- \(s_1; s_2\) がすでに継続の形をしていることに気が付いた。

以下です。

- 定義
  - \(V \defeq \N\) ... これは値、そんな大げさなことをしないので \(\N\) でいい。
  - \(E_{\text{global}}, E_{\text{local}} \defeq \NT{var} \to V\) ... 変数の環境
  - \(\lbracket x \in \NT{var} \rbracket_{E_g \in E_{\text{global}}, E_l \in E_{\text{local}}}: \N \defeq \cdots\) ...これは look up を local, global でやってその値の取り出し。
  - \(K \defeq \emptyset | (\NT{stmt}, E_{\text{local}}, x_{i \in 0 \ldots n} \in \NT{var} )::K \) ... 継続
    - 戻ってきたら実行する stmt と、環境と、 return 時に代入する variable の組
  - \(S \defeq (\NT{stmt}, E_{\text{global}}, E_{\text{local}}, K)\) ...これは実行機械の状態
- 文の実行の仕方
  - \((s_1; s_2, E_g, E_s, K) \to (s_1'; s_2, E_g', E_s', K')\) if \((s_1, E_g, E_s, K) \to (s_1', E_g', E_s', K')\) ... seq rule 意味の与え方が木構造になった。
  - \((\T{Nop}; s, E, K) \to (s, E, K)\)
  - \((\T{return} r_i; s, E_g, E, K) \to (\T \return r_i, E_g, E, K)\) ... return を見たら後ろを捨てる。
  - \((\T{return}, E_g, E, \emptyset) \to E_g\) ... これがプログラム全体を通した `AInput` から `FOutput` への写像。
  - \((\T{return} r_{i \in 0 \ldots m'}, E_g, E, (s, E', w_{i \in 0 \ldots m}):: K) \to (s, E_g, E'', K)\)
    - \(m \neq m'\) なら `Err` として、そうじゃないなら、 \(E'' := E'[w_i := \lbracket r_i \rbracket_{E_g, E} ]\)  とする。
  - \((\T{call} \mathit{name} v_{i \in 0 \ldots n} \T{->} w_{i \in 0 \ldots m}; s, E_g, E_l, K) \to (s', E_g, E', (s, E_l, w_{i \in 0 \ldots m} ):: K)\)
    - \(\mathit{name}\) で宣言されている手続きが \(\mathit{name} p_{i \in 0 \ldots n'} s'\) とする。
    - \(n \neq n'\) なら `Err` でいい、そうじゃないなら、 \(p_i := \lbracket v_i \rbracket_{E_g, E_s}\) として**値渡し**
    - local variable は \(\NT{proc}\) のときに宣言されているものを \(0\) で fill して渡して、上と合わせて \(E'\) とする。
  - 他 3 つは straight forward でしょう。

> [!Note]
> AI 談
> - structural operational semantics (Plotkin style?) ... \(s_1; s_2 \to s_1'; s_2\), \(\T{Nop}; s \to s\) にするやりかた
> - abstract machine (SECD, CEK) に近いのは、 \(s_1; s_2\) をみたら \(s_2\) は継続に積むらしい
> あとで本当か調べよう。
> 他に気が付いたこと：
> call/return がきれいに呼び出しの継続を扱っている。
> 例えば、 `(call, E, K) -> (s, E', (Nop, E)::K)` みたいにすると seq rule から `(call; s', E, K) -> (s; s', E', (Nop, E)::K)` となる。
> これは `s` での return がうまくハンドルできていない。（本当は `s` に戻ってほしいのに、 `abort` みたいに、全部の継続が捨てられることになる。）
> なので、 `call; s` という形自体に対する変形ルールが必要になる。**`call` は常に継続をとる！**
> `call` 側がちゃんとしているので、 `return` は文レベルでの継続を全部捨てる形にしていい。

## 意味論 ( program counter 方式 )
どの手続きのどの行を実行しているかを追いかければいい。
あまり述べることがない...

## flow_ir へのコンパイル方法
