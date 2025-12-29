# abstract non-sense な議論
結局この話は、統一的にモデルを書こうとする話（このリポジトリ）では必要になったので書いておく。
（もともと Coq で書こうとして抽象的すぎる抽象化がいやでやめたのに...）

- 部分関数に対しても適当に合成 * をやっておく
- 定義域を含めた（両者の定義域が一致していて）等式 = を考えておく。
- 定義域を含めない（両者が定義されている範囲では等しい）等式 ~ を考えておく。

## 計算モデルは何をしている？
計算能力に関する議論は次のようにまとめることができる。
\(\Sigma = \{\mathbb{B}, 0, -\}\) を固定する。
\(\Nat\) で自然数全体を表す。

- チューリングマシンについて
  - \(\Sigma\) 上の Tape 全体を \(T_{\Sigma}\) とする
  - \(\Sigma\) 上のチューリングマシン全体を \(C\) とする。
  - \(\text{ev}: C \to (T_{\Sigma} \partfunction T_{\Sigma})\)
- 再帰関数について
  - 再帰関数全体の集合 := \(\text{Rec}\)
  - \(\text{Rec}\) は \(\Nat \partfunction \Nat\) の部分集合であり、包含写像を \(i\) とする。
- 符号化について
  - 符号化 \(s_1: \Nat \to T_{\Sigma}\) と 符号化 \(s_2: T_{\Sigma} \partfunction \Nat\) がある。
  - 関係式として \(s1 * s2 \sim \text{id}\), \(s2 * s1 \sim \text{id}\) が成り立つ。
  - 写像間の encode の定義： \(\text{encode}^{\Nat}_{T_{\Sigma}}: (T_{\Sigma} \partfunction T_{\Sigma}) \partfunction (\Nat \partfunction \Nat)\) と逆も
- 翻訳について
  - 再帰関数のチューリングマシンへの翻訳 \(\text{cp}: \text{Rec} \to C\) がある。
  - ただし次を満たす： \(s_2 * \text{ev}(\text{cp}(f)) * s_1 = f\)
  - もうちょっと式を整理： \(\text{Rec} \overset{\text{cp}}{\to} C \overset{\text{ev}}{\to} (T_{\Sigma} \partfunction T_{\Sigma}) \overset{\text{encode}}{\to} (\Nat \partfunction \Nat)\) と \(\text{Rec} \overset{i}{\to} (\Nat \partfunction \Nat)\) が一致する。

これが、再帰関数がチューリングマシンによってあらわせたという状況の整理である。

> [!Note]
> 一般に、チューリングマシンを帰納関数から再現することを行う場合、符号化を変える必要がある。
> ただ、チューリングマシンと帰納関数の場合、符号化が部分関数になっているのはこっちの都合というか、
> 頑張れば符号化の部分は部分関数じゃない全域で定義された関数を使うことができそう。
> 理由は、符号化の部分は本質的には"計算"ではないから。

また、 universal Turing machine に対しても同様の感じで議論を進めようとすると次のようになる。
- チューリングマシンとテープの符号化 \(\text{emb}: C \times T_{\Sigma} \to T_{\Sigma}\)
- universal Turing machine \(U \in C\) がある。
- これらは次を満たす... \(\text{ev} U (\text{emb}(M, t)) = \text{ev} M t\) が任意の \(M \in T\) と \(t \in T_{\Sigma}\) で成り立つ。
- 計算可能な関数の全体とは \(\Im (\text{encode} * \text{ev})\) のことである。
- "universal Turing machine" に対応する再帰関数 \(u \in \text{Rec}\) がある
- すなわち次が成り立つ... \(u = \text{encode} (\text{ev}(U))\)



# 参考になる
## 一般の system を調べる場合
### Mealy machine / Moore machine
次のものは同じ。
- 有限集合: 状態 \(S\), 入力 \(I\), 出力 \(O\)
- 状態遷移 \(S \times I \to S\)
出力をどう計算するかが違う。

Mealy machine
: \(S \times I \to O\) が出力

Moore machine
: \(S \to O\) が出力

回路とか "system" っぽいものの一般論に使われる。
ただし、入力・出力がリアルタイムに入ってくる想定に近い。

集合の有限性の仮定を外してしまえば一般のシステムが記述できる。

> [!Note]
> 有限性の仮定を外すことはいいが、"次の状態"が確実に計算できないといけない。
> 状態遷移で partial を許すことはできないし、
> \(S\) が無限の場合に"全体を見渡して"計算することも許されない。
> ただし、そういう制限自体がうまく定義できれば、
> 各計算モデルに対して対応する Machine が存在するように作れてもよさそう。
> つまり、「 \(I \partfunction O\) が部分関数にならざるを得ないのは、"無限ステップ"かかるから」を定式化できる？

### universal coalgebra
[universal coalgebra] を呼んだが、ちょっと関係なかったかも。
確かに Mealy machine を含めたいろんな系が議論できるのはわかるが、"計算"の話とはちょっとずれてる？
主に、系の発展と bisimulation の話を一般論でやるための設定っぽい。

## T diagram
Wiki [Tombstone diagram] を見ると、必要最低限ぐらいの物が書かれている。
Tombstone diagram と呼ばれる、言語処理系の系譜を説明するために使われるような図式のこと。

I diagram
  : interpreter は、メタ言語 Lm で書かれたプログラムが言語 L の処理系になる。
  これを I の形にして、下側に Lm、 上側に L を書く。

T diagram
  : compiler は、 メタ言語 Lm で書かれたプログラムが言語 L1 を入力として受け取って言語 L2 を出力する。
  これを T の形にして、下側に Lm、左に L1、右にL2を書く。

これの別の表し方として、 J diagram というのが
[digram for composing compilers] とかで紹介されている。

プログラムが言語を受け取る（？）の部分は、符号化を考えないといけないが、
全部自然数と思えるからいいだろうと思っている？

## encode のない世界の場合
syntax と semantics の中で考える場合には syntax 側のものを semantics 側に移す必要がある。
semantics 側だけで考えていいとしても、いろんなことが示せる。

### Turing category
[Intro Turing cat] これはかなり考えていることに近い気がする。
- partial である, paring がある
- eval と encoding がある（ application ができる）
- universal なものが作れる

### partial combinatorial algebra
partial な application \(A \times A \partfunction A\) を考えるらしい。
これは暗黙的に program と data を変換している。

## Futamura projection の話を読んで
[is there a fourth]
話のまとめ
- プログラミング言語に寄らずに、データは固定する： \(D\)
- \(D\) には、pairing がある： \(D \times D \to D\)
- 各プログラミング言語は \((P, \text{sem}_P: P \to (D \partfunction D))\) の組と思える。
- 各プログラミング言語自体もデータ自体へのエンコード \(\text{enc}: P \to D\) を持つ。

これがあれば、 Tombstone diagram とかも書ける。

ふと思ったのが、この \(D\) は PCA にできる？
構文側の話を抜きにすれば、 \(D \to (D \partfunction D)\) がデータをコードとして扱う写像で、
curry 化の \(k \dot a \dot b = k \dot (a, b) = a\) もすでに存在する。

## ブラムの公理
計算模型に寄らない形で計算複雑性を定義できるらしい。
後で調べる。

## その他メモ
可算であるか判定できることはどの議論にも必要に思える。
なので、初めから自然数の上で議論していると考えたほうがいい？

## reference
mdbook のビルドでは見えないが、以下がちゃんとしたリンク

[universal coalgebra]: https://ir.cwi.nl/pub/48/0048D.pdf
[Intro Turing cat]: https://www.sciencedirect.com/science/article/pii/S0168007208000948
[digram for composing compilers]: https://johnwickerson.github.io/papers/jdiagrams.pdf
[Tombstone diagram]: https://en.wikipedia.org/wiki/Tombstone_diagram
[is there a fourth]: https://gwern.net/doc/cs/algorithm/2009-gluck.pdf
[fixpoint theorem in computability]: https://www.math.ru.nl/~terwijn/publications/surveyrecthm.pdf
- https://arxiv.org/pdf/2204.03553