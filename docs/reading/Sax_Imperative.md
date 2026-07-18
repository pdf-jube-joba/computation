# [Lecture Notes on Sax: Imperative Intermidiate Language](https://www.cs.cmu.edu/~fp/courses/15417-s25/lectures/)
Sax というコンパイラ用の中間言語についての講義

## 1
1回目では、 positive (関数が first-order) で linear (memory cell が読み込まれると消えるので、GC がいらない) 言語を導入する。

jugement としては large value \(V\) に対する type \(A\) の judgement を考える。
- value の構成 variable は含まれないので context がいらない。
- large というのは、単一の値に必要な情報が全部入っているため。

### judgement

1. pair \((V, W): A \tensor B\) if \(V: A\), \(W: B\)
2. unit \((): 1\)
3. sum \(k(V): \sum_{l \in L} (l: A_l)\) if \(k \in L, V: A_k\)
  - これは、 \(L\): set ... ラベル用の空じゃない有限集合、と \(A_{l \in L}\) に対して。
4. equirecursive: 再帰型については \(\mu \alpha. A\) みたいに、型変数を許して再帰する型変数を宣言するスタイルじゃなくて、トップレベルの宣言にする。例えば、 \(\text{nat} = \oplus \{\text{zero}: 1, \text{succ}: nat\}\) のように、 \(\tensor_i A_i \to \alpha\) の \(A_i\) の部分だけ許される。
  - equi と書いたのは、型の同型があるというよりも equation があるという感じの定義だかららしい。
  - こうして作られるものは帰納的な型になる。
  - なお、 constructor に対応するものは直和と同じようにラベルの扱い方をしている。

> [!Note]
> 再帰型の実装だと、 equi-recursive と iso-resurcive があったはず。

### memory と small values
large value を unit だけ入れれるメモリ上の表現として与えたい。
- cell として、 _address_ \(a\) と small value \(v\) の組を考える： \(\text{cell} c v\)
- small value は 1. unit 2. address の pair, 3. label と address の pair

ちゃんとメモリ全体を見ないと large value が見れない。

### typing memory
ここでは、メモリに対するコマンドとしての write を考える。
以下では、 \(c\) は address とする。
- tensor \(a: A, b: B \vdash \text{write} c (a, b):: (c: A \tensor B\), for \(a, b\): address
- unit \( \vdash \text{write} c ():: c: 1\)
- sum \(a: A_{k \in L} \vdash \text{write} c k(a):: c: \sum_{l \in L} A_l\)

この judgement において、 \(\vdash\) の前は read from, 後は write to と読める。

Process \(P\) という何かコマンドが与えられたときに、 \(\text{proc}(P)\) で実行を表す。
\(\text{proc}(\text{write} c v \to \text{cell} c v\) が small value に対しては成り立つ。

### reading from memory
メモリの読み込みがちょっと難しい。
Process \(P\) が変数付きのときに、 \(\text{read} c (x, y) \to P(x, y)\) でメモリを読み込んで P を実行したものを表す。
\(c: A \tensor B\) で \((a, b)\) という small value のときに、 \(\text{read} c (x, y)\) では \(x, y\) に \(a, b\) が束縛されてほしい。

- tensor \(\Gamma, c: A \tensor B \vdash \text{read} c (x, y) \to P(x, y):: (d: D)\) if \(\Gamma, x: A, y: B \vdash P(x, y):: (d: D)\)
- unit \(\Gamma c: A \vdash \text{read} c () \to P :: (d: D)\) if \(\Gamma \vdash P:: (d: D)\)
- sum \(\Gamma c: \sum_{l \in L} A_l \vdash \text{read} c {l(x_l) \to P_l (x_l)}:: (d: D)\) if \(\Gamma x_l: A_l \vdash P_l(x_l):: (d: D), \forall l \in L\)
  - これはメモリの内容に応じて分岐することになる。全ての分岐先を用意できている必要があるので、 \(\forall\)

> [!Note]
> これはちゃんと資料に書かれていることだが、まとめてここに書く。
> シーケント計算でみるとこんな感じ
> - write
>   - \(A, B \vdash A \tensor B\)
>   - \(\vdash 1\)
>   - \(A \vdash A \sum B\)
>   - \(B \vdash A \sum B\)
> - read
>   - \(\Gamma A \tensor B \vdash C\) if \(\Gamma A, B, \vdash C\)
>   - \(\Gamma, 1 \vdash C\) if \(\Gamma \vdash C\)
>   - \(\Gamma \vdash A \oplus B \vdash C\) if \(\Gamma, A \vdash C\) && \(\Gamma, B \vdash C\)
> read は L 則に対応している。
> R 則は axiom で代用できるので、 semi-axiomatic sequent calculus と読んでいる所以らしい。

### cut and identity
cut はこんな感じ
- \(\Gamma, \Delta \vdash \text{cut} (x: A) P(x) Q(x):: (d: D)\) if \(\Gamma \vdash P(x):: (x: A)\) && \(\Delta, x: A \vdash Q(x):: (d: D)\)

これはプログラムとしては、
メモリセル \(a\) として fresh なものをとり、 \(P(a)\) を走らせて \(a\) への書き込みがあった後に、 \(Q(a)\) が \(a\) から読みだす。
\(a\) は future らしい。
イメージとしては、 \(\text{proc}(\text{cut} (x: A) P(x) Q(x)) \to \text{proc} P(a), \text{proc} Q(a)\) 並列な実行になる。

この方向での動的な実行は、 substructural operational semantis と呼ばれて、 multiset rewriting に基づいている。

identity はメモリセルの移し替え。
\(b: A \vdash \text{id} a b:: a: A\)
### procedure
手続きをどう定義するか？（メモリに対する作用があるので、関数とは言わない。）

名前が \(p\) の procedure の定義の仕方は、
\(\text{PROC} p (x: A) (y _ 1: B _ 1) \ldots (y_n: B_n) = P(x, y_1, \ldots, y_n\) のように書く。

ここで、全ての procedure は return する先の destination を一番初めに書き、 \(y_i\) は argument になる。
全ての変数が address に bound されたときに、
\(\text{CALL} p a b_1, \ldots b_n\) のようにして呼び出す。
呼び出しは再帰的にできる。

変数とは何かみたいなことを考えると、この手続きは [[contextual modal type theory]] と呼ばれるものになる。

### 一旦文法のまとめをしたい。
- ラベルは必要に合わせて好きなものをとっていい。
- address の集合 \(A\) は固定するがなんでもいい。
- 変数の集合 \(V\) をなんでもいいのでとる。
