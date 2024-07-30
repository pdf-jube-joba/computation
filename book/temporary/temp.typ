#import "@preview/ctheorems:1.1.2": *
#import "@preview/curryst:0.3.0": rule, proof-tree
// #set math.equation(numbering: "(1)")
#show: thmrules

#let problem = thmbox(
  "problem",
  "Problem",
  stroke: rgb("#000000") + 1pt,
).with(numbering: none)

#let result = thmbox(
  "result",
  "Result",
  stroke: rgb("#000000") + 1pt,
).with(numbering: none)

#let ind = h(10pt)
#let h = h(4pt)
#let kw(word) = text(green)[#word]

= 継続によるラムダ計算の拡張の実装

== ラムダ計算の振り返り
=== 単純な評価関数
ラムダ計算の項は次のような文法で定義される。
（ただし $x$ は変数である。）
$ "Exp"
  &::= "Var" (x: "variable") \
  &| "Lam" (x: "varable", e: "Exp") \
  &| "App" (e_1: "Exp", e_2: "Exp")
$
ラムダ計算の項は真ん中の形をしているとき value という。

次の簡約規則により、 call-by-value で left-to-right な評価戦略を定める。
ここで、 $"subst"(e, x, t)$ は（変数の束縛とかをちゃんと考えた）代入である。
- base-case: $"App" ("Lam" x #h e) #h v -> "subst"(e, x, v)$ ただし $v$ は value 
- eval-r-case: $"App" v_1 #h e_2 -> "App" v_1 #h e'_2$ ただし $v_1$ は value で $e_2 -> e'_2$
- eval-l-case: $"App" e_1 #h e_2 -> "App" e'_1 #h e_2$ ただし $e_1 -> e'_1$
base-case に現れるような形をしたラムダ項を redex という。
以降では、 $r$ を redex 、 $v$ を value を表すメタ変数とする。
簡約規則を見ると全ての $e -> e'$ なる関係は base-case から eval-l か eval-r を用いて得られ、$e -> e'$ を与える理由付けは一意である。
つまり、 $e -> e'$ を与える証明を証明木に直せばそれは一意になっている。
特に、 $e -> e'$ は（部分）関数を定めている。

（疑似言語ではあるが、）ここから単純に再帰関数を使えば簡約が定まる。
// ml ではないが
```ml
step: Exp -> Option Exp :=
step (Var _) = None,
step (Lam _ _) = None,
step (App (Lam x e) e2) =
  if is_value(e2) then subst(e, x, e2)
  else App (Lam x e) step(e2)
step (App e1 e2) = App step(e1) e2
```
=== 継続概念について
Exp を redex になっている部分と残りの部分に分けることを考える。
残りの部分はどのように表現されるか。
具体例として $t = "App" v_1 #h ("App" r #h e)$ の簡約を考えると、 $r -> r'$ とおけば $"App" v_1 #h ("App" r' #h e)$ へと簡約される。
これを $f = M |-> "App" v_1 #h ("App" M #h e)$ なる $r$ の"外側"を用いて $t = f(r) -> f(r')$ と簡約されたととらえることができる。
これを踏まえると、 項を redex と外側に分けるつぎの関数が得られる。
// TODO 参考資料として大堀先生の youtube を張る。

```ml
decomp: Exp -> Option (Exp, Exp -> Exp) :=
decomp (Var _) = None,
decomp (Lam _ _) = None,
decomp (App e1 e2) =
  if is_value(e1) && is_value(e2) then (App e1 e2, fun x -> App e1 f(x)) 
  else if is-value(e1) then
    let (r, f) = decomp e2
    (r, fun -> App e1 f(x))
  else
    let (r, f) = decomp e1
    (r, fun -> App f(x) e2)
```

decomp の右側にある $"Exp" -> "Exp"$ について、どんな関数でも現れるわけではないことに注意し、より適切なデータ型を与えると、それが評価文脈である。
$ "Cxt" ::= "Hole" | "EvalL" (e: "Exp", E: "Cxt") | "EvalR" (v: {e: "Exp" | "is-value" e}, E: "Cxt") $
直感的には、 `fun x -> App e1 f(x)` なる関数を覚えておくために必要なデータ型が `EvalR e1 f` であり、 `fun x -> App f(x) e2` なる関数を覚えておくために必要なデータ型が `EvalL e2 f` である。
このデータ型と評価文脈の対応は次のようになる。
```ml
plug: Cxt -> Exp -> Exp :=
plug Hole t = t
plug (EvalL e c) t = plug c (App t e)
plug (EvalR v c) t = plug c (App v t)
```
この対応のもとで、評価文脈 `f` を受け取って評価文脈 `fun x -> App e1 f(x)` を返す関数は次のように書ける。
```ml
extend_l: Exp -> Cxt -> Cxt :=
extend_l e1 Hole = EvalL e1 Hole
extend_l e1 (EvalL e c) = EvalL e (extend_l e1 c)
extend_l e1 (EvalR v c) = EvalR e (extend_l e1 c)
```
`fun x -> App f(x) e2` を返す関数は次のように書ける。
```ml
extend_r: Exp -> Cxt -> Cxt :=
extend_r e2 Hole = EvalR e2 Hole
extend_r e2 (EvalL e c) = EvalL e (extend_r e2 c)
extend_r e2 (EvalR v c) = EvalR e (extend_l e2 c)
```

また redex を表すデータ型を次のように与える。
$ "Red" ::= "Red" (x: "variable", e: "Exp", v: {e: "Exp" | "is-value"(e)}) $
redex に関して次のような写像がある。
- redex をラムダ項に戻す関数 $"as-lam": "Red" -> "Exp"$ 
- ラムダ項が redex かを判定する関数 $"is-rdx": "Exp" -> "Option" "Red"$
- redex の簡約を進める $"step": "Rdx" -> "Exp"$

この状態で再度 decomp を定義すると次のように書ける。
```ml
decomp: Exp -> Option (Red, Cxt) := fun e -> 
match is_rdx(e)
Some r -> Some (r, Hole)
None ->
  match e
  Var _ = None
  Lam _ _ = None
  App e1 e2 =
    if is_value(e1) then
      let (r, c) = decomp(e2)
      Some (r, extend_r e1 c)
    else
      let (r, c) = decomp(e1)
      Some (r, extend_l e2 c)
```
また、`step`は次のように書ける。
```ml
step: Exp -> Option Exp := fun e ->
let Some (r, c) = decomp e
plug c step(r)
```

注意点として、この状態で次のように簡約関係を定義してみると、これは先ほど定義した簡約関係とは性質が異なる。
- base-case: $"App" ("Lam" x #h e) #h v -> "subst"(e, x, v)$ ただし $v$ は value
- cxt-case: $"plug" c #h e -> "plug" c #h e'$ ただし $e -> e'$
性質としては、 $M -> M'$ なる関係自体は部分関数的であるが、その理由付けとしての証明木自体には一意性がなりたっていない。
（つまり、 $E_1 != E_2$ と $e_1 != e_2$ を用いて $M = E_1[e_1] = E_2[e_2]$ のようにかけ $e_1 -> e'_1$ かつ $e_2 -> e'_2$ となるような $M$ が簡単に作り出せる。
ただし、 $E_1[e'_1] = E_2[e'_2]$ が成り立つ。）

=== stack に直す。
今、 Cxt の型定義に注目すると、これは次の Frame なるデータ型を用いた List として表せる。
$ "Frame" ::= "EvalL" (e: "Exp") | "EvalR" (v: {e: "Exp" | "is-value"(e)}) $
このように見ると、 decomp や plug 及び step について、次のように理解することができる。
ラムダ項の実行状態を表す型を次のように定める。
$ "State" ::= "State" (e: "Exp", c: "List" "Frame")  = angle.l e | c angle.r $
これらの間に次のような関係が定まっている。
- $angle.l v | ("EvalL" e') :: c angle.r ->_("comp") angle.l "App" v #h e' | c angle.r$
- $angle.l v | ("EvalR" v') :: c angle.r ->_("comp") angle.l "App" v' #h v | c angle.r$
- $angle.l r | c angle.r ->_("step") angle.l r' | c angle.r$
- $angle.l "App" v_1 #h e_2 | c angle.r ->_("decomp") angle.l e_2 | ("EvalR" v_1) :: c angle.r$ ただし $v_1$ は value
- $angle.l "App" e_1 #h e_2 | c angle.r ->_("decomp") angle.l e_1 | ("EvalL" e_2) :: c angle.r$ ただし $e_1$ は value ではない。

このとき、 $angle.l m | c angle.r ->_("*") angle.l m' | c' angle.r$ なる関係は部分関数になっている。
これは実際、次の関数のコールスタックと対応しているはずである。
```ml
step: Exp -> Option Exp :=
step (Var _) = None,
step (Lam _ _) = None,
step (App (Lam x e) e2) =
  if is_value(e2) then subst(e, x, e2)
  else App (Lam x e) step(e2)
step (App e1 e2) = App step(e1) e2
```

== grab/limit によるラムダ計算の拡張
=== grab/delimit の定義と 1 ステップ簡約
grab/delimit による拡張は次のようになる。
$ "Exp"
  &::= "Var" (x: "variable") \
  &| "Lam" (x: "varable", e: "Exp") \
  &| "App" (e_1: "Exp", e_2: "Exp") \
  &| "Delim" (e: "Exp") \
  &| "Grab" (k: "variable", e: "Exp") \
$

value の定義は変わらないが、 redex の定義は異なり、次に述べる PureCxt 及び plug 関数を用いた再帰的な定義になる。
対応する Cxt を次の $2$ 種類与える。
$ "PureCxt" &::= "Hole" \
  &| "EvalL" (e: "Exp", E: "PureCxt") \
  &| "EvalR" (v: {e: "Exp" | "is_value"(e)}, E: "PureCxt") \
  "Cxt" &::= "Hole" \
  &| "EvalL" (e: "Exp", E: "Cxt") \
  &| "EvalR" (v: {e: "Exp" | "is_value"(e)}, E: "Cxt") \
  &| "Delim" (e: "Cxt")
$
`plug` 関数の実装は省略する。

簡約を定める関係式としては次のようになる。
- base-case: $"App" ("Lam" x #h e) #h v -> "subst"(e, x, v)$
- delim-case: $"Delim" v -> v$
- grab-case: $"Delim" "plug"(c, "Grab" k #h e) -> "subst"(k, e, "Lam" y #h "plug"(c, y))$ ただし $c$ は PureCxt
- cxt-case: $"plug"(c, e) -> "plug"(c, e')$ ただし $e -> e'$ かつ $c$ は Cxt

この関係については、部分関数性は成り立つが、証明木の一意性は成り立たない。
また、 "grab" が現れることで単純な再帰関数を用いた step の実装が難しくなっている。
例えば次のように実装を書いてみる。
```ml
step: Exp -> Option Exp :=
step (Var _) = None,
step (Lam _ _) = None,
step (App (Lam x e) e2) =
  if is_value(e2) then subst(e, x, e2)
  else App (Lam x e) step(e2)
step (App e1 e2) = App step(e1) e2
step Delim e =
  if is_value(e) then e else Delim (step)
step Grab k e = ?
```
ここでどのように Grab を書こうとしても、再帰的に step が呼ばれる中で対応する Delim の情報が（このコード上には）残っていないために、実装を行うことができない。
（当然、ホスト言語のコールスタックには残っているが、それを陽に扱わなければいけない。）

これを解決するため、以降では順に必要なデータ型を（通常のラムダ計算のやり方にのっとり）定義していく。
redex は次のようなデータ型である。
（右側はラムダ式への変換を表す。）
$ "Rdx" &::= "Red" (x: "variable", e: "Exp", v: {e: "Exp" | "is-value"(e)}) = "App" ("Lam" x #h e) #h v \
  &| "Delim" (v: {e: "Exp" | "is-value"(e)}) = "Delim" v \
  &| "DelGrab" (c: "PureCxt", k: "variable", e: "Exp") = "Delim" ("plug"(c, #h ("Grab" k #h e)))
$
次の関数が定義できる。
- $"as_lam": "Rdx" -> "Exp"$
- $"is_rdx": "Exp" -> "Option" "Rdx"$
ここで、 $"as_lam"$ が単射であることが重要である。
また、そのいわば逆写像である $"is_lam"$ もプログラムとして書ける。
そのさいに必要な extend_l や extend_r は変わらない。

次の step 関数が簡約における cxt を除いた関係を担っている。
```ml
step: Rdx -> Exp :=
Rdx x e v = subst(e, x, v)
Delim v = v
DelGrab c k e =
  let cont = Lam y (Delim plug(c, y))
  subst(e, k, cont)
```

これらをもとに decomp が定義できる。
```ml
decomp: Exp -> Option (Rdx, Cxt) := fun t ->
if is_value(t) then None
else if is_redex(t) then
  let r = is_rdx(t)
  Some (r, Hole)
else match t
  Var _ | Lam _ _ | Grab _ _ = None
  App e1 e2 =
    if is_value(e1) then
      let (r, c) = decomp(e2)
      Some (r, extend_l c e1)
    else
      let (r, c) = decomp(e1)
      Some (r, extend_r c e2)
  Delim e =
    let (r, c) = decomp(e)
    Some (r, extend_d c e)
```

これにより step が同様に定義できる。

== stack を用いる。
$ "Frame" ::= "EvalL" (e: "Exp") | "EvalR" (v: {e: "Exp" | "is-value"(e)}) | "Delim" $
$ "State" ::= "State" (e: "Exp", c: "List" "Frame") = angle.l e | c angle.r $

State の関係は次のようになる。
- $angle.l v | ("EvalL" e') :: c angle.r ->_("comp") angle.l "App" v #h e' | c angle.r$
- $angle.l v | ("EvalR" v') :: c angle.r ->_("comp") angle.l "App" v' #h v | c angle.r$
- $angle.l v | ("Delim") :: c angle.r ->_("comp") angle.l "Delim" v | c angle.r$
- $angle.l r | c angle.r ->_("step") angle.l r' | c angle.r$
- $angle.l "App" v_1 #h e_2 | c angle.r ->_("decomp") angle.l e_2 | ("EvalR" v_1) :: c angle.r$ ただし $v_1$ は value
- $angle.l "App" e_1 #h e_2 | c angle.r ->_("decomp") angle.l e_1 | ("EvalL" e_2) :: c angle.r$ ただし $e_1$ は value ではない。
- $angle.l "Delim" e | c angle.r ->_("decomp") angle.l e | "Del" :: c angle.r$
- $angle.l "grab" k #h e | c angle.r ->_("grab") angle.l "Delim" F["Grab" k #h e] | c_2 angle.r$ ただし、次の条件が満たされている。
  - $c = (c_1: "List" "Frame") :: "Del" :: c_2$ で $c_1$ には "EvalL" か "EvalR" の Frame のみ含まれる。
  - $F$ は $c_1$ から得られる PureCxt とする。
このとき、 $angle.l m | c angle.r ->_(*) angle.l m' | c angle.r$ は部分関数になっている。