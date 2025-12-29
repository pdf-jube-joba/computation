# ラムダ計算による自然数関数の計算可能性
ラムダ計算もまた帰納関数の計算を"埋め込む"ことができる。
帰納関数の埋め込み方は複数あるが、ここでは代表的な Church encoding を用いる。
自然数の埋め込みと、帰納関数の埋め込みを定義する。

### Church encoding

自然数とラムダ項の対応
: \(n \in \Nat \mapsto \lambda f. \lambda z. f^{n} x\) により定義する。

その他必要な項の定義
- `true = \x y. x`
- `false = \x y. y`
- `succ = \ x y z. y(x y z)`
- `pred = \x y z. x(\ p q. q(p y))(\ v. z)(\ v.v)`
- `iszero = \x. x (\y. false) true)`
- `if = \x y z. x y z`
- \(\text{proj}^n_i = \lambda x_1, \ldots , x_n. x_i\)
- \(\text{comp}^n_m = \lambda g f_1, \ldots f_n. \lambda x_1, \ldots x_m. g (f_1 x_1 \cdots x_m) \cdots (f_n x_1 \cdots x_m\)\)

これでゼロ関数、後者関数、射影関数、合成関数については定義できたと思ってよい。

これ以上の部分については次の不動点に関する議論が必要になる。

### 不動点コンビネータと再帰関数
再帰関数とは、その関数の定義に自分自身が用いられているもののことを言う。
例えば、階乗 \(\text{fact}\) の定義は次のように書ける
- \(\text{fact}(0) = 1\)
- \(\text{fact}(n + 1) = (n + 1) * \text{fact}(n)\)

\(2\) 番目のケースにおいて自分自身を用いて定義していることがわかる。
このような関数を定義するうえで重要なこととして、「実際にすべての値で必ず関数値が定まっているか」を確かめる必要がある。
例えば、次の関数は再帰的に定義されているように思えるが、本当に関数になっているかは証明の余地がある。
- \(\text{f} (1) = 1\)
- \(\text{f} (2 * n) = n\)
- \(\text{f} (n) = \text{f} (3 * n + 1)\)

本当に関数になっているかはさておき、こういう再帰的定義をラムダ計算に表すには工夫が必要である。

\(\mathbf{Y}\) コンビネータ
: \(\lambda f. (\lambda x. f(x x)) (\lambda x. f(x x))\)

> **theorem**
> - 任意のラムダ式 \(M\) に対して \(\mathbf{Y} M \rightarrow^* M (\mathbf{Y} M)\) が成り立つ。
> - 任意のラムダ式 \(M\) に対して \(H = \mathbf{Y} (\lambda h x_1 , \ldots x_n. M)\) とおくと \(H \sim \lambda x_1, \ldots x_n. M[h := H]\) が成り立つ。

これを用いると再帰関数が簡単に書ける。
\(2\) つめを用いて以降は単に \(H = \lambda x_1, \ldots x_n. M[h := H]\) のような式で定義を行う。

このコンビネータは不動点コンビネータと呼ばれる。
その理由は、"関数" \(M\) に対して "値" \(v = \mathbf{Y} M\) は不動点になっている（ \(M v = v\) をみたす）からである。
すなわち、 \(\mathbf{Y}\) は関数の不動点を計算し返している関数である。

再帰関数の残りの部分を定義してしまう。


原始再帰関数のラムダ式による表現
: ラムダ式 \(f, g\) に対して
  \[H = \lambda x_0, x_1, \ldots x_n. \text{if} (\text{iszero} x_0) (f x_1 \cdots x_n) (g (H (\text{pred} x_0) x_1 \cdots x_n) (\text{pred} x_0) x_1 \cdots x_n) \]

\(\mu\) 再帰関数のラムダ式による表現
: ラムダ式 \(f\) に対して
  \[H = \lambda x_0, x_1, \ldots x_n. \text{if} (\text{iszero} x_0) x_0 (H (\text{succ} x_0) x_1 \cdots x_n)\]

あとは再帰関数との対応をしっかり与えればよいが、ここまでやれば自明なのでいいか。
こうして再帰関数がラムダ計算で計算可能であることが分かった。

<div data-model="recursive_function-lambda_calculus">
<script type="text/plain" class="default-code">
PRIM[z: PROJ[1, 0] s: COMP[SUCC: PROJ[3, 0]]]
</script>
<script type="text/plain" class="default-ainput">
(2, 3)
</script>
</div>
