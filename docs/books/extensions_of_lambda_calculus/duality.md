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

## cbv と cbn について
reduction の形をしている \((\lambda x. M) N\) に対して、 \(N\) が値になっているかを見る。
プログラミング言語でよく見るのが cbv で、引数を値になるまで評価してから代入する。
cbn だとそのままいれる。

> [!note]
> ラムダ計算の内部からは、あるラムダ項が値かどうかを判定することはできなかったので、
> メタな取り扱いをするしかなさそう。
> ただし、 weak head normal は簡約されない評価戦略の下では、お互いの簡約されたくない部分をその形に変形することで、いい感じにできる。

### CPS 変換との関係性
CPS は、型 \(A\) の項を継続を受け取って適用するような \(\forall R, (A \to R) \to R \) に変換するかのようなイメージ。
線形空間でよくある双対の双対の空間への埋め込みに似ている。
値側のものと関数側のものを入れ替えるための操作？

CBV の場合の CPS 変換は次のような感じになる。
- \([x] = \lambda k. k x\)
- \([\lambda x. M] = \lambda k. k (\lambda x. [M]\)
- \([M N] = \lambda k. [M] (\lambda m. [N] (\lambda n. (m n) k))\)

最後のだけ非自明。

cps 変換された \([M]\) に渡すのは、関数 \(m\) を受け取って何をするかが書かれた継続になっているはずで、
\(M\) 自身をこの継続の \(m\) 部分に代入しようとしているはず。この意味で、 large M と small m が対応している。だから結果として、最後に \(m n k\) が表れている。

ただしよく考えると、 \(m n\) の型は \(M N\) と同じような \(B\) になっているわけではない。
だからこの変換は型 \(A\) を受け取って \(\forall R, (A \to R) \to R\) にしているわけではない。

このように型を考えると **失敗する** ので注意する。
| 前の項 | 型 | 対応する継続 |
| --- | --- | --- |
| \(M: A \to B\) | \([M]: ((A \to B) \to R _ 1) \to R _ 1\) | \(\lambda m: (A \to B) \to R _ 1\) |
| \(N: A\) | \([N]: (A \to R _ 2) \to R _ 2\) | \(\lambda n: A \to R _ 2\) |
| \(M N: B\) | \([M N]: (B \to R) \to R\) | \(\lambda k: B \to R\) |

ちゃんとした型をつける。
基底となる型に対しては、 \(\alpha^\dagger = \alpha\) として、 \((A \to B)^\dagger = A^\dagger \to (B^\dagger \to R) \to R\) とする。
こうすれば、 CPS 変換後には \((A^\dagger \to R) \to R\) の型がつく。

CBN の場合の CPS 変換は次のような感じになる。
- \([x] = x\)
- \([\lambda x. M] = \lambda k. k (\lambda x. [M]\)
- \([M N] = \lambda k. [M] (\lambda m. (m [N]) k)\)

CBV との違いとしては、関数適用で、 \(N\) が継続を持つ形に変換されていないこと。
もともとこれを書いた Plotkin は CBV と CBN がお互いをシミュレーションするための埋め込みとして定義したらしい。
変換でのシミュレーションの際には、恒等写像を継続として与えるなどの工夫は必要。

ちゃんとした型をつける。
基底となる型に対しては、 \(\alpha^\dagger = (\alpha \to R) \to R\) として、 \((A \to B)^\dagger = A^\dagger \to B^\dagger\) とする。 

### duality について
この時点ではそんなに duality が見えない。

## dual calculus
- Parrigot による lambda mu caluclus
- Herbelin と Curien による lambda mu mu-tilde calculus
- Barbanera と Berardi による symmetric lambda calculus
継続を陽に扱って古典論理に対応する計算体系を定義している。
ここらへんで、 call by name と call by value が dual という話が出てきたらしい。
ただ、 reduction 先が複数あって全部の項が equal とみなせたり、ちょっと問題があったりするらしい。
term と coterm がそれぞれ極性に対応しているので、論理の極性をちゃんとみることでわかりやすくなる。

ここでは Wadler による dual calculus を見る。
これは古典シーケント計算に対応していて、かつ CBV と CBN の対応が見やすいらしい。
また、古典論理での双対（ de Morgan 的なもの）も構文レベルで対応が取れる。

\[\begin{aligend}
\NT{Type} &\defeq \NT{type-var} \\
& | \NT{Type} \T{and} \NT{type} | \NT{Type} \T{or} \NT{type} \\
& | \T{neg} \NT{Type} | \NT{Type} \T{implies} \NT{Type} \\
\\
\NT{Term} &\defeq \NT{var} \\
& | \T{pair} \NT{Term} \NT{Term} | \T{inl} \NT{Term} | \T{inr} \NT{Term} | \T{neg} \NT{Coterm} \\
& | \T{lam} \NT{var} \NT{Term} | \T{abs} \NT{Statement} \NT{co-var} \\
\\
\NT{Coterm} &\defeq \NT{co-var} \\
& | \T{either} \NT{CoTerm} \NT{Coterm} | \T{fst} \NT{Coterm} | \T{snd} \NT{Coterm} | \T{co-neg} \NT{Term} \\
& | \T{apply} \NT{Term} \NT{Coterm} | \T{co-abs} \NT{var} \NT{Statement} \\
\\
\NT{Statement} &\defeq \T{eval} \NT{Term} \NT{Coterm}
\end{aligned}\]
term は value を生み出し、 coterm は value を消費する。
statment は term の cut を行う。

ここでは型変数自体には極性はないらしい。
シーケントとしてこんな感じに定義する。
\[\begin{aligned}
\NT{Ante} &\defeq \text{list of \((\NT{var}, \NT{Type})\)}
\NT{Succ} &\defeq \text{list of \((\NT{co-var}, \NT{Type})\)}
\\
\NT{R-seq} &\defeq \NT{Ante} \T{to} \NT{Succ} \T{|} \NT{Term} \T{:} \NT{Type} \\
\NT{L-seq} &\defeq \NT{Coterm} \T{:} \NT{Type} \T{|} \NT{Ante} \T{to} \NT{Succ} \\
\NT{C-seq} &\defeq \NT{Ante} \T{|} \NT{Statement} \T{to} \NT{Succ}
\end{aligned}\]

computational content としての judgement の解釈：
- ante にある 各 var に対して value を与えておく。
- succ にある covar か coterm に対して、value を与える。
- R-seq \([x: A_i]_i \vdash [\beta_i: B_i]_i | M: B\) の場合、 expression \(M\) を評価すると 「\(B\) の型の値を生み出す」か「\(\beta_i\) になんかの継続を渡す」をやる。
- L-seq \(K: A | [x_i: A_i]_i \vdash [\beta_i: B_i]_i\) の場合、coterm \(K\) に対して \(A\) の型を持つ value を入れると \(\beta_i\) のいずれかに継続を入れる。
- C-seq \([x_i: A_i]_i | S \vdash [\beta_i: B_i]_i\) の場合は、 Statement を評価するとどれかに返ってくる。

ルールを書くのはめんどくさい。
Cut に対応するのは R-seq と L-seq から Statement を得て C-seq にする操作である。

### reduction は？
普通にやるとこんな感じになる。
- \(\beta_L\): \(\T{eval} (M, (\T{abs}(x, S)) \to S[x := M]\)
- \(\beta_R\): \(\T{eval} (\T{co-abs}(S, \alpha), K) \to S[\alpha := K]\) 
これは合流性を持たない。
- CBV にするなら \(\beta_L\) の \(M\) は Value のみにする
- CBN にするなら \(\beta_R\) の \(K\) は Co-Value のみにする

他の部分でも、 \(\T{eval}\) のうち、左側が Value になるようにするのが CBV で、右側が CoValue になるようにするのが CBN になる。

## polarity について
まあここは後で書く。
- 古典論理での dual は、 de Morgan の話で、これは \(A\) と \(\neg A\) の入れ替えになっている。ただし、これは値と継続を入れ替える話なので、対応する計算体系には値だけじゃなくて継続を入れるといい。
- 古典シーケント計算では \(\and\) や \(\or\) の R/L 則に対称性がある。ただし、証明の構築を見るときに、可逆性が消えていたりするので、それを復活させたい。
- CBV と CBN の dual というのは、ちゃんと値と継続を陽に扱う体系に洗練したときに、どちらを先に簡約していくかに対応している。

## call by push value
value と computation をわけるのが call by push value で、ここでいう computation は continuation ではなくて thunk を作るという話になっている。
そもそも CBV と CBN の duality は value と continuation のどっちを先に見るかという軸での対立になっている。
だから、 value と continuation を合わせた構文であることが重要。
一方で、 CBV と CBN の比較として副作用がいつ表れるかを比較するのが、 computational effect の話。
CBPV は（AIによると） continuation の双対を computation として別枠っぽく見ているととらえられるらしい。

\[\begin{aligned}
\NT{p-type} &\defeq \T{U} \NT{n-type} | \NT{sum} \NT{p-type} \NT{p-type} | \T{1} | \NT{pair} \NT{n-type} \NT{n-type} \\
\NT{n-type} &\defeq \T{F} \NT{p-type} | \NT{prod} \NT{n-type} \NT{n-type} | \T{fun} \NT{p-type} \NT{n-type} 
\end{aligned}]

ここで p-type は value で n-type が computation に対応する。
- \(\T{U}\) は computation を value にする操作なので、 UB を作るのは thunk で、 UB を消費するのが thunk を起動する force である。
- \(\T{F}\) は value を computation にする操作で、これは `return  V` に近い。

pair と prod で直積っぽいものが p-type にも n-type にもあるのは、
- p-type の pair は value の pair で、これは pair にある両方を取り出して `match v with (x, y) => M` してよい。
- n-type の prod は comp の pair のため、必要に応じてどれかを fst/snd で projection して取り出すことができる。ここでは、 `fst(V, W)` が value にならないように、そもそも fst がついたら computation 側になるような配慮をしている。

\[\begin{aligned}
\NT{Value} &\defeq \NT{var} | \T{thunk} \NT{Term} \\
& | \T{inl} \NT{Value} | \T{inr} \NT{Value} \\
& | \T{pair} \NT{Value} \NT{Value} \\
\\
\NT{Term} &\defeq \NT{var} \\
& | \T{produce} \NT{Value} | \T{force} \NT{Value} \\
& | \T{let} \T{var} \T{:=} \NT{Value} \T{in} \NT{Term} \\
& | \T{let2} \NT{Term} \T{to} \NT{var} \T{in} \NT{Term} \\
& | \T{pm-sum} \NT{Value} \T{with} \T{inl} \NT{var} \T{=>} \NT{Term} \T{|} \T{inr}  \NT{var} \T{=>} \NT{Term} \T{end} \\
& | \T{pm-pair} \NT{Value} \T{with} \NT{var} \NT{var} \T{=>} \NT{Term}  \\
& | \T{prod-c} \NT{Term} \NT{Term}  | \T{fst} \NT{Term} | \T{snd} \NT{Term} \\
& | \T{fun} \NT{var} \NT{Term} | \NT{apply} \NT{Term} \NT{Value} \\
\end{aligned}\]

let と let2 の比較：
- let の方は、今すぐに value を得てそれを計算に組み込む。これは文の合成というよりも、命令文？計算上は継続を見ずに V を代入する操作になる。
- let2 の方はちょっと違って、ここの bind は continuation をとる `let x := [] in N` と continuation の dual っぽい computation の合成操作になっている。つまり、 `() -> A` と `A -> B` の合成に近い。 `produce V` に対応している。

typing は対応しそうな pair と prod と sum と lam については飛ばして書く。
- \(\Gamma \vdash^c \T{produce} V: \T{F} A\)
  - \(\Gamma \vdash^v V: A\)
- \(\Gamma \vdash^v \T{thunk} M: \T{U} B\)
  - \(\Gamma \vdash^c M: B\)
- \(\Gamma \vdash^c \T{force} V: B\)
  - \(\Gamma \vdash^v V: \T{U} B\)
- \(\Gamma \vdash^c \T{let2} x \T{:=} M \T{in} N: B\)
  - \(\Gamma \vdash^c M: \T{F} A\)
  - \(\Gamma, x: A \vdash^c M: B\)

judgement には \(\Gamma \vdash^v \NT{term}: \NT{n-type}\) と \(\Gamma \vdash^c \NT{value}: \NT{p-type}\) の2種類があって、何が value かをこの judgement で定義する。

CK 機械で動作を見る。CEK じゃないのは、環境を考えずにただちに代入を行うから（ closure が生成されない）。
あと、継続の部分に詰まれることがあまりなくて、 \(M\) だけで話が終わることが多い？気がする。
lambda に対応するものは確かに継続に push を行うが、 let に対応するものは即座に C 上で代入を行う。この意味では、 let2 が lambda の逆っぽい？

\[\begin{aligned}
(\T{let}(x, V, M), K) &\to (M[x := M], K) \\
(\T{let2}(x, M, N), K) &\to (M, (\T{let2}(x, [], N) ::K)) \\
(\T{produce}(V), (\T{let2}(x, [], N)) K) &\to (N[x := V], K) \\
(\T{force}(\T{thunk}(M)), K) &\to (M, K) \\
(\T{pm-pair} (\T{inl} V, (x _ 1, M _ 1), (x _ 2, M _ 2)), K ) &\to (M _ 1[x _ 1 := V] K) \\
(\T{pm-pair} (\T{inr} V, (x _ 1, M _ 1), (x _ 2, M _ 2)), K ) &\to (M _ 2[x _ 2 := V] K) \\
(\T{fst} M, K) &\to (M, (\T{fst} [] ):: K) \\
(\T{snd} M, K) &\to (M, (\T{snd} [] ):: K) \\
(\T{prod-c} (M _ 1, M _ 2), (\T{fst} [])::K) &\to (M _ 1 , K) \\
(\T{prod-c} (M _ 1, M _ 2), (\T{snd} [])::K) &\to (M _ 2 , K) \\
(\T{apply} (M, V)) &\to (M, \T{apply}([], V):: K) \\
(\T{fun}(x, M), \T{apply}([], V):: K) &\to (M[x := V], K) 
\end{aligned}\]


ちなみに、ここでは computation のうち closed なものに対してのみうまくいく。
なので、 \(\Gamma\) に対応するような、いつ push されたかわからない variable を取り出そうと思うとちょっと苦労している？
CK 機械の継続 \(K\) を computation として型付けするような judgement を定義している。

### CBV や CBN との関係
CBV 側の \(A \to B\) は \(T{U} (A \to \T{F} B)\) として考えることができる。
CBN 側の \(A \to B\) は \(\T{U} A \to B\) として考えることができる。
