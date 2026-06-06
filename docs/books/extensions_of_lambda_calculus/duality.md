線形論理、型の polarity と CBPV と、値と継続の双対についてのメモ

> [!Note]
> こっちが、最終的にやりたいことの本質な気がしてきました。
> https://www.cs.cmu.edu/~fp/courses/15417-s25/lectures/

## ラムダ計算と論理体系
とりあえず命題論理と単純型付きラムダ計算で考えたい。
命題論理の証明体系のうち、ラムダ計算に対応するのは自然演繹であって、シーケント計算ではないらしい。
自然演繹のイメージ：最初に習うやつだと仮定を消費するときは証明木の葉にバツをつける。
ただ、シーケント風にすることもできるので、これで比較しながらシーケント計算との違いを考えたい。 
命題を implies と atom だけで定義して比較する。

なにがあるとシーケント計算なのか自然演繹なのかを判定する感覚が自分にはないので、
ここの比較は間違っていそう
- 多重集合/順序付き ... コンテキストの定義は、自然演繹は通常は集合で、シーケント計算は順序付きになる。
  - 自然演繹が証明木の葉の部分を消す流儀なので、シーケント風にするときにも集合になる？
  - これは微妙な考え方。
- 右則と左則でわけるのと、 intro/elim でわける違い ... 与えられたものが右則左則的なのか intro/elim 的なのかをどう判定するのか。
  - これが本命っぽいが、感覚の話であって、与えられた論理体系に対してそれが intro/elim を判定するのが自分には難しい。

\[
\NT{Prop} = \NT{atom} | \NT{Prop} \T{\(\to\)} \NT{Prop} \\
\NT{Context} = \emptyset | \NT{Context} \T{,} \NT{Prop}
\]

自然演繹にせよシーケント計算にせよ、コンテキストを集合じゃなくて ordered list にするなら、以下のものは必要：
| rule | conclusion | assumption |
| --- | --- | --- |
| assum | \(A \vdash A\) | |
| weak | \(\Gamma, B \vdash A\) | \(\Gamma \vdash A\) |
| contraction | \(\Gamma, B \vdash A\) | \(\Gamma, B, B \vdash A\) |
| exchange | \(\Gamma, A, B, \Delta \vdash C\) | \(\Gamma, B, A, \Delta \vdash C\) |

weak と contraction を使わないようにするのが線形論理っぽい。
以下は自然演繹もシーケント計算も ordered list で考えた場合。

自然演繹の場合：
| rule | conclusion | assumption |
| --- | --- | --- |
| intro |  \(\Gamma \vdash A \to B\) | \(\Gamma, A \vdash B\) |
| elim | \(\Gamma, \Delta \vdash B\) | \(\Gamma \vdash A \to B\) と \(\Delta \vdash A\) |

シーケント計算の場合：右と左で分ける
| rule | conclusion | assumption |
| --- | --- | --- |
| right |  \(\Gamma \vdash A \to B\) | \(\Gamma, A \vdash B\) |
| left | \(\Gamma, \Delta, A \to B \vdash C\) | \(\Gamma \vdash A\) と \(\Delta, B \vdash C\) |
| cut | \(\Gamma, \Delta \vdash C\) | \(\Gamma \vdash A\) と \(\Delta, A \vdash C\) |

こうしてみると、確かに自然演繹の方がラムダ計算に対応している。
自然演繹の場合は、 \(\Gamma \vdash B\) という式を見て、それの証明に elim を使うかはわからない。
一方でシーケント計算の場合は、「うまいこと分割をとる」方針で、
context に入っている仮定を一つ一つ分解してバックトラックするとなんとかなりそう。

証明木の方では、ラムダ項の normalization に対応するのが、 intro と elim のある証明木を normalization するもので、
これはシーケント計算の方では cut free 化していく過程になるはず。

### 他の命題論理の要素について
\(\bot, \top, \vee, \wedge\)

## 古典論理の場合
### 自然演繹
普通の古典論理だと排中律（ \(P \vee \neg P\) ）とか二重否定除去だが、これは \(\neg\) が必要なのでちょっと置いておく。 implies だけなら、 Pierce law を使うとよくて、公理として次のものを入れる：\[\vdash ((A \to B) \to A) \to A\]

プログラム側で対応するのは callcc らしい。
ちゃんと考えたい。callcc は継続をとるような記述をうけとって着地させる。
実際に使うなら、 \(\text{let \(x\) := \(\text{callcc} (\lambda k. M) \) in N} \) みたいなシチュエーション。
これは \((\lambda x. N) (\text{callcc} (\lambda k. M) \))\) をまさに簡約しようとしている。
ただし、まだ右側が value の形をしていないのでそれを簡約しようとする状態：（ `let x = 1 + 1 in x * 2` をするために、まず `1 + 1` をしようとしている。）

右側の簡約部分が難しくて、 \(M\) を評価する中で \(k\) は単純に \(\lambda x. N\) になる。
ただし、  \(\lambda x. N\) の評価中であることは変わらなくて、単に、 \(k\) で外側の継続を捕まえることができているだけ。 \(M\) を \(k\) に継続を入れたうえで評価して、その得られた値を改めて \(x\) に束縛する。
これが入ると簡約が局所的な判断では行えなくなる。つまり、「 \(V M\) を見て、 \(M\) だけに注目して \(M'\) にして、 \(V M'\) とする」 ことができない。

> [!Note]
> ちゃんとした定義：評価文脈 \(E[]\) を使って \(E[\text{callcc} (\lambda k. M)] \to E[M[k := (\lambda v. E[v])]]\)

まあその話は置いといて、型について、具体的に \((\lambda x: A. N)(\text{callcc}(\lambda k: ?_0, M))\) の簡約をもとに考える。
- 簡約のコンテキストは \(E[] = (\lambda x: A. N) []\) なので、\(\lambda v. E[v] = \lambda v. \left\{(\lambda x: A. N) v\right\} = \lambda v: A. N[x := v] = \lambda x: A .N\) である。
- \(x: A\) に \(\text{callcc} (\lambda k: ?_0. M)\) を代入しようとしているので、後者の型は \(A\) である。
  - これをもっと進めると、 \(M[k := \lambda v. E[v]]\) の部分が型 \(A\) を持っているべきなので、 \(k: ?_0 \vdash M: A\) になっているはず。
- \(k\) に \(\lambda x: A. N\) を代入しようとしているので、 何かの型 \(B\) があって、 \((\lambda x: A. N): A \to B\) として \(?_0 = A \to B\) になる。
- 以上を合わせると、 \(k: A \to B, M: A\) のはずで、 \(\text{callcc} (\lambda k. M): A\) のはず。

### シーケント計算
右側を複数の式に拡張するだけで、古典論理になるらしい。
直観的には、 \(\Gamma \vdash P_1, \ldots P_n\) は \(\Gamma \vdash P_1 \vee \cdots \vee P_n\) のこと。
Pierce law は公理ではなくなったので、一応導出を見ておくことにする。

- \(\vdash ((A \to B) \to A) \to A \)
  - \((A \to B) \to A \vdash A\)
    - \((A \to B) \to A \vdash A_0, A_1\) ... \(A_0 = A_1 = A\)
      - \(\vdash A \to B, A_0\)
        - \(A \vdash B,(A_0 = A)\)
          - \(A \vdash A\)
      - \(A \vdash A_1\)

これを書いていて途中で \(\to R\) がきもいことに気が付いた。
\(A \vdash B, C\) から \(\vdash (A \to B), C\) とか \(\vdash B, (A \to C)\) にしていいらしい。直感的には \(\vdash (A \to B), (A \to C)\) なのに？
古典論理での説明としては \((A \to B) \vee C = (\neg A \vee B) \vee C = \neg A \vee (B \vee C)\) とか \(\neg A \vee B \vee \neg A \vee C\) なのでいい。
ただ直観論理では成り立たないらしい。
古典論理の場合のモデルは付値だが、直観論理の場合は Kripke frame を使えば成り立たないことが示せるらしい。
