[[Model]] [[構造化言語]]

call/return で手続呼び出しを行う言語を考える。
- 引数と返り値あり。
  - いずれも複数でいい。
- 大域変数と局所変数あり。
  - global か自分の local で宣言された変数のみ使用できる。
  - 呼び出し元手続きの変数環境は見えないものとする。
- 値呼びにする。
- どの関数を呼び出すかを動的に決める仕組みがないです。

\[
\begin{aligned}
\\
\NT{var}    &\defeq \NT{string} \\
\NT{name}   &\defeq \NT{string} \\
\NT{atom}   &\defeq \NT{var} | \NT{imm} \\
\\
\NT{aexp}   &\defeq \NT{atom} \NT{binop} \NT{atom} \\
\NT{bexp}   &\defeq \NT{atom} \NT{binrel} \NT{atom} \\
\\
\NT{var-cmm} &\defeq \syntaxmacro{comma-separated}{\NT{var}} \\
\\
\NT{stmt} &\defeq \\
  &| \T{Nop} \\
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{var} \T{:=} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB} \\
  &| \T{while} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB} \\
  &| \T{call} \NT{name} \T{\LP} \NT{var-cmm} \T{\RP} \T{->} \NT{var-cmm} \\
  &| \T{return} \NT{var-cmm} \\
\\
\NT{proc} &\defeq \NT{string} \T{\LP} \NT{var-cmm} \T{\RP} \T{\LCB} \\
  &\T{local} \NT{var-cmm} \\
  &\NT{stmt} \\
  &\T{\RCB} \\
\\
\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{var}} \T{;} \\
  \NT{proc}+ \\
\end{aligned}
\]

- \(\T{;}\) は右結合
- stmt は return で終わらなかったら \(\T{return}\) が入っているものとして扱う。
- static は code の方で宣言だけしておいて、 `AInput` と名前があっているか確認する。

## 意味論（ small-step ）
call-stack が必要になるぐらいで、あとは `while_lang` と同じ。
引数位置に var が来て値渡しなので評価順序も引数評価用の環境もいらない。
`AInput` も `FOutput` も global env と考える。
early return ができる。\(s_1; s_2\) がすでに継続の形をしていることに気が付いた。

- 名前の解決は終わっていることとする。（ `x = 10;` みたいなものの羅列。）
  - global と local と parameter の名前が被ったら `Err` とする。
- main と名前のついている proc から実行する。これの引数はないものとする？
  - 初期状態は全ての variable が \(0\) に行っているものとする。
- proc から push するときに、 stmt の最後には return を入れておく。

以下です。

- 定義
  - \(V \defeq \N\) ... これは値、そんな大げさなことをしないので \(\N\) でいい。
  - \(E_{\text{global}}, E_{\text{local}} \defeq \NT{var} \pfun V\) ... 変数の環境
  - \(\llbracket x \in \NT{var} \rrbracket_{E_g \in E_{\text{global}}, E_l \in E_{\text{local}}}: \N \defeq \cdots\) ...これは look up を local, global でやってその値の取り出し。
  - \(K \defeq \emptyset | (\NT{stmt}, E_{\text{local}}, x_{i \in 0 \ldots n} \in \NT{var} )::K \) ... 継続
    - 戻ってきたら実行する stmt と、環境と、 return 時に代入する variable の組
  - \(S \defeq (\NT{stmt}, E_{\text{global}}, E_{\text{local}}, K)\) ...これは実行機械の状態
- 文の実行の仕方
  - \((s_1; s_2, E_g, E_s, K) \to (s_1'; s_2, E_g', E_s', K')\) if \((s_1, E_g, E_s, K) \to (s_1', E_g', E_s', K')\) ... seq rule 意味の与え方が木構造になった。
  - \((\T{Nop}; s, E, K) \to (s, E, K)\)
  - \((\T{return} r_i; s, E_g, E, K) \to (\T{return} r_i, E_g, E, K)\) ... return を見たら後ろを捨てる。
  - \((\T{return}, E_g, E, \emptyset) \to E_g\) ... これがプログラム全体を通した `AInput` から `FOutput` への写像。
  - \((\T{return} r_{i \in 0 \ldots m'}, E_g, E, (s, E', w_{i \in 0 \ldots m}):: K) \to (s, E_g', E'', K)\)
    - \(m \neq m'\) なら `Err` として、そうじゃないなら、
      - ローカルへの代入： \(w_i \in E'\) に対しては \(E'' := E'[w_i := \llbracket r_i \rrbracket_{E_g, E} ]\) で更新する。
      - グローバルへの代入： \(w_i \in E_g\) に対しては \(E_g' := E'[w_i := \llbracket r_i \rrbracket_{E_g, E}]\) で更新する。
  - \((\T{call} \mathit{name} v_{i \in 0 \ldots n} \T{->} w_{i \in 0 \ldots m}; s, E_g, E_l, K) \to (s'; \T{return}, E_g, E', (s, E_l, w_{i \in 0 \ldots m} ):: K)\)
    - \(\mathit{name}\) で宣言されている手続きが \(\mathit{name} p_{i \in 0 \ldots n'} s'\) とする。
    - \(n \neq n'\) なら `Err` でいい、そうじゃないなら、 \(p_i := \llbracket v_i \rrbracket_{E_g, E_s}\) として**値渡し**
    - local variable は \(\NT{proc}\) のときに宣言されているものを \(0\) で fill して渡して、上と合わせて \(E'\) とする。
  - 他 3 つは straight forward でしょう。

> [!Note]
> AI 談
> - structural operational semantics (Plotkin style?) ... \(s_1; s_2 \to s_1'; s_2\), \(\T{Nop}; s \to s\) にするやりかた
> - abstract machine (SECD, CEK) に近いのは、 \(s_1; s_2\) をみたら \(s_2\) は継続に積むらしい
> 
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
これは実装しなくていい。（実質 `flow_ir` へのコンパイルだから。）

## flow_ir へのコンパイル方法
`Err` になるようなプログラムは考えなくていいので、引数の個数や返り値の個数があっているかは気にせずにやっていい。
**間違えたプログラムを書いた人が悪いので、コンパイル時にはそれを考えない。（未定義動作みたいなもの？）**
\(E_{\text{global}}\) はそのまま static にする。
残りは call-stack に置けばいい。
proc_lang 側でスタックに触れないので、 push/pop は call/local variable/return のときにしか触らないことが確定しているから、
手続きの内部で引数や局所変数にアクセスするときは、固定 index と考えていい。

- 変数・引数については、名前の解決の段階で global/parameter/local がわかるので次のようにする。
  - global: 最初に static に割り振る。
  - parameter \(p_{i \in 0 \ldots n}\): 最初にスタックの長さを \(\T{\%v\_fp}\) として手に入れておく。（これは実質 frame pointer になっている。）
    \(\T{\%} p_i := \T{\%v\_fp} - (i + 1)\) とする。 \(T{sacc} \T{\%} p_i\) で Key が手に入る。\(i + 1\) はコンパイル時に計算できる。
    局所変数の初期化前は、一番上は return address が入っている状態なので注意。
  - local \(x_{i \in 0 \ldots n}\): 局所変数初期化後にスタックの長さを \(\T{\%v\_sp}\) として手に入れておく。
    \(\T{\%} x_i := \T{\%v\_sp} - i\) とする。 \(\T{sacc} \T{\%} x_i\) で Key が手に入る。
- \(\NT{stmt} _{\text{proc lang}} \to \NT{block} _{\text{flow ir}}+\) は次のようにやる。
  - \(\T{Nop}\), \(s_1; s_2\), \(x := a\), if/while は while language のときと同じようにやる。
  - call: ブロックを2つにする。
    - 呼び出しブロック：
      - 引数を**右から** ld して push ... 逆順にすると \(- i\) と合う。（これは好み。）
      - 戻ってきたときブロックのラベルを push ... return 先
    - 戻ってきたときブロック：
      - pop して st を各 \(w_i\) ごとにやる。 \(i\) の小さい方から。（caller は \(w_1\) に入れるべき値が top に来ていることを期待。）
  - return: ブロック1つでいい。
    - return に使う variable から値を引き出して `\%v_ri` に入れておく（局所変数を使っているとこの後消えるので注意。）
    - 自分が使った local 分を pop （局所変数は最初に宣言...コンパイル時に確定している。）
    - return add を pop して `\%v_ret` に入れる
    - 返り値 `\%v_ri` を全部 push する。 \(i\) の大きい方から。（ callee は caller に合わせて、 \(w_m\) に入れるべき値から順に push する。）
    - `\%v_ret` に飛ぶ。
- 各手続きの最初：
  - \(\T{lget} \T{\%v\_fp}\) で長さを取得。 
  - local variable を push 0 で確保する。
  - \(\T{lget} \T{\%v\_sp}\) で長さを取得。
  - これで call stack は次のような状況になっている。スタックは上から入れるような書き方。
    | 入っている値 | 位置 |
    | --- | --- |
    | 局所変数 \(x_0\) | \(v\_sp\) |
    | 局所変数 \(x_1\) | \(v\_sp - 1\) | 
    | ... | ... |
    | 局所変数 \(x_n\) | \(v\_sp - n = v \_fp + 1\) |
    | return addr | \(v\_fp\) |
    | 引数 \(p_0\) | \(v\_fp - 1\) |
    | 引数 \(p_1\) | \(v\_fp - 2\) |
  - このスタックの長さは最終的に関数を抜ける return 以外では変化しない。
    他の手続きを call しても、ちゃんと処理が終わった後はこの長さになっている。
    なので、相対位置を手続きの初めに確定したらあとは計算する必要がない。
- main だけ戻る先がないので特別扱いする必要がある。コンパイル先の flow_ir で `:halt {halt; }` だけ書かれたものを用意して、
  return addr を一番最初に `:halt` に向けて push しておく。
  これで、 main も implicit/explicit return で扱える。
