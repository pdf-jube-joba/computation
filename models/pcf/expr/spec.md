[[Model]]
[[Compiler]]

これから以下をコンパイル云々ができるようにしていきたいが、
コンパイル前後で `ROutput` 出力の順序が保たれているかどうかは気にするが、その step 数は気にしない。

## 構文
\[
\begin{aligned}
\NT{var} &\defeq \NT{string} \\
\NT{binop} &\defeq \T{+} | \T{-} | \T{\&\&} \\
\NT{unop} &\defeq \T{\textbackslash inc} | \T{\textbackslash dec} | \T{\textbackslash not} \\
\\
\NT{expr} &\defeq \\
  &| \T{\LP} \NT{expr} \T{\RP} & \syntaxname{結合順序の制御用} \\
  &| \N | \T{\textbackslash true} | \T{\textbackslash false} | \T{\textbackslash unit} \\
  &| \T{\textbackslash print} \N &\syntaxname{呼び出し順序確認用} \\
  &| \NT{var} \\
  &| \NT{expr} \NT{binop} \NT{expr} \\
  &| \NT{unop} \NT{expr} \\
  &| \T{\textbackslash fun} \NT{var} \T{=>} \NT{expr} \\
  &| \T{\textbackslash rec} \NT{var} \NT{var} \T{=>} \NT{expr} \\
  &| \NT{expr} \T{\LP} \NT{expr} \T{\RP} \\
  &| \T{\textbackslash if} \NT{expr} \T{\textbackslash then} \NT{expr} \T{\textbackslash else} \NT{expr} \T{\textbackslash fi} \\
\end{aligned}
\]

## 意味論
reduction の列で意味論が定義できる。
値渡しにして、 \(\T{\textbackslash print} \N\) は `ROutput` に表示しつつ \(\T{\textbackslash unit}\) を返す。
- if では分岐判定部分を簡約して、値が得られたらそのまま適用する。分岐先は簡約しない。
- これとは逆に、 `&&` は strict である。
- 関数適用だけ違って、関数部分も引数部分も値にすること。
- `AInput` は \(\N\) の tuple として `FOutput` \(\N\) と考える。実行は、プログラム \(M\) と `AInput` \(n_1, \ldots n_k\) を \(((M n_1) \cdots n_k)\) にして実行を始める。

### ちゃんとした定義１？
- 値：\(V \subset M\) := \(\N + \T{\textbackslash true} + \T{\textbackslash false} + \T{\textbackslash unit} + \text{Fun}(x, M) + \text{Rec}(f, x, M)\)
  - なお、 `FOutput` としては \(\N\) 以外は許されない。
- stuck（ step で `Result<T, E>` の `Err` の方）： \(\bot\)

\[
\begin{aligned}
\T{\textbackslash print} (n \in \N) & \to^{\text{print} n} \T{\textbackslash unit} \\
M \NT{binop} N &\to M' \NT{binop} N && M \to M' \\
V \NT{binop} N &\to V \NT{binop} N' && N \to N' \\
V \NT{binop} V' &\to V'' && V, V' \in \text{domain} \NT{binop} \\
V \NT{binop} V' &\to \bot && \text{otherwise} \\
\NT{unop} M &\to \NT{unop} M' && M \to M' \\
\NT{unop} V &\to V' && V \in \text{domain} \NT{unop} \\
\NT{unop} V &\to \bot && \text{otherwise} \\
M N &\to M' N && M \to M' \\
V N &\to V N' && N \to N' \\
(\T{\textbackslash fun} x \T{=>} M) V &\to M[x:=V] \\
(\T{\textbackslash rec} f x \T{=>} M) V &\to M[f := \T{\textbackslash rec} f x \T{=>} M][x:=V] \\
V M &\to \bot && V \neq (\T{\textbackslash fun} x M) \\
\T{\textbackslash if} L M N &\to \T{\textbackslash if} L' M N && L \to L' \\
\T{\textbackslash if} \T{\textbackslash true} M N &\to M \\
\T{\textbackslash if} \T{\textbackslash false} M N &\to N \\
\end{aligned}
\]

> [!Note]
> rec は値に入れずに \(\T{\textbackslash fun} x => (M[f := \T{\textbackslash rec} f x \T{=>} M])\) に reduction されるとしてもいい。

### ちゃんとしてない定義？
evaluation context を使って定義もできる。
- focus \(R\) : 上の簡約表で、実質的に計算が進んでいる部分。 \(V \NT{binop} V'\), \(\NT{unop} V\), \(\T{\textbackslash rec}\) \((\T{\textbackslash fun} x M) V\), \(\T{\textbackslash if}\) の true, false,
- evaluation context はフレームの部分
- \(M = E[R]\) となる \(E, R\) が \(M\) に対して定まるので、 \(M = E[R] \to E[N] = M'\) のようにして簡約が進む。
