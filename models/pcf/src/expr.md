[[Model]]
[[Compiler]]

これから以下をコンパイル云々ができるようにしていきたいが、
コンパイル前後で `ROutput` 出力の順序が保たれているかどうかは気にするが、その step 数は気にしない。

## 構文
\[
\begin{aligned}
\NT{var} &\defeq \NT{string} \\
\NT{binop} &\defeq \T{+} | \T{-} | \T{\&\&} \\
\NT{unop} &\defeq \T{inc} | \T{dec} | \T{not} \\
\\
\NT{expr} &\defeq \\
  &| \T{\LP} \NT{expr} \T{\RP} & \syntaxname{結合順序の制御用} \\
  &| \N | \T{\#true} | \T{\#false} | \T{\#unit} \\
  &| \T{print} \N &\syntaxname{呼び出し順序確認用} \\
  &| \NT{var} \\
  &| \NT{expr} \NT{binop} \NT{expr} \\
  &| \NT{unop} \NT{expr} \\
  &| \T{fun} \NT{var} \T{=>} \NT{expr} \\
  &| \T{rec} \NT{var} \NT{var} \T{:=} \NT{expr} \\
  &| \NT{expr} \T{\LP} \NT{expr} \T{\RP} \\
  &| \T{if} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{fi} \\
\end{aligned}
\]

## 意味論
reduction の列で意味論が定義できる。
値渡しにして、 \(\T{print} \N\) は `ROutput` に表示しつつ \(\T{\#unit}\) を返す。
- if と case では分岐判定部分を簡約して、値が得られたらそのまま適用する。分岐先は簡約しない。
- これとは逆に、 `&&` は strict である。
- 関数適用だけ違って、関数部分も引数部分も値にすること。
- `AInput` は \(\N\) の tuple として `FOutput` \(\N\) と考える。実行は、プログラム \(M\) と `AInput` \(n_1, \ldots n_k\) を \(((M n_1) \cdots n_k)\) にして実行を始める。

### ちゃんとした定義１？
- 値：\(V \subset M\) := \(\N + \T{\#true} + \T{\#false} + \T{\#unit} + (\T{fun} x M)\)
  - なお、 `FOutput` としては \(\N\) 以外は許されない。
- stuck（ step で `Result<T, E>` の `Err` の方）： \(\bot\)

\[
\begin{aligned}
\T{print} (n \in \N) & \to^{\text{step} n} \T{\#unit} \\
M \NT{binop} N &\to M' \NT{binop} N && M \to M' \\
V \NT{binop} N &\to V \NT{binop} N' && N \to N' \\
V \NT{binop} V' &\to V'' && V, V' \in \text{domain} \NT{binop} \\
V \NT{binop} V' &\to \bot && \text{otherwise} \\
\NT{unop} M &\to \NT{unop} M' && M \to M' \\
\NT{unop} V &\to V' && V \in \text{domain} \NT{unop} \\
\NT{unop} V &\to \bot && \text{otherwise} \\
\T{rec} f x \T{:=} M &\to \T{fun} x \T{=>} M' && M' = M[f := (\T{rec} f x \T{:=} M)] \\
M N &\to M' N && M \to M' \\
V N &\to V N' && N \to N' \\
(\T{fun} x \T{=>} M) V &\to M[x:=V] \\
V M &\to \bot && V \neq (\T{fun} x M) \\
\T{if} L M N &\to \T{if} L' M N && L \to L' \\
\T{if} \T{\#true} M N &\to M \\
\T{if} \T{\#false} M N &\to N \\
\end{aligned}
\]

### ちゃんとしてない定義？
evaluation context を使って定義もできる。
- focus \(R\) : 上の簡約表で、実質的に計算が進んでいる部分。 \(V \NT{binop} V'\), \(\NT{unop} V\), \(\T{rec}\) \((\T{fun} x M) V\), \(\T{if}\) の true, false,
- evaluation context はフレームの部分
- \(M = E[R]\) となる \(E, R\) が \(M\) に対して定まるので、 \(M = E[R] \to E[N] = M'\) のようにして簡約が進む。
