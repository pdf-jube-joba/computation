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

= 継続によるラムダ計算の拡張
ここでは疑似言語を使う。

== ラムダ計算の振り返り
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
簡約規則を見ると全ての $e -> e'$ なる関係は base-case から eval-l か eval-r を用いて得られ、$e -> e'$ を与える理由付けは一意である。
つまり、 $e -> e'$ を与える証明を証明木に直せばそれは一意になっている。
そのため、 $e -> e'$ は（部分）関数を定めている。

（疑似言語ではあるが、）ここから単純に再帰関数を使えば簡約が定まる。
- $"step": "Exp" -> "Option" "Exp"$
- $"step" ("Var" \_ ) = "None"$
- $"step" ("Lam" \_ #h \_) = "None"$
- $"step" ("App" ("Lam" x #h e) #h e_2) = #kw("if") "is-value"(e_2) kw("then") "subst"(e, x, e_2) kw("else") "App" (("Lam" x #h e) #h "step"(e_2)) $
- $"step" (e_1 e_2) = ("step"(e_1) e_2)$

ところで、 Exp を redex になっている部分と残りの部分に分けることを考える。
残りの部分はどのように表現されるかを説明する。
具体例として $t = "App" v_1 #h ("App" r #h e)$ は $r -> r'$ とすると $"App" v_1 #h ("App" r' #h)$ へと簡約される。
これを $f = M |-> "App" v_1 #h ("App" M #h e)$ なる $r$ の"外側"を用いて $t = f(r) -> f(r')$ と簡約されたととらえることができる。
これを踏まえると、 項を redex と外側に分けるつぎの関数が得られる。
// TODO 参考資料として大堀先生の youtube を張る。
- $"decomp": "Exp" -> "Option" ("Exp", "Exp" -> "Exp")$
- $"decomp" ("Var" \_) = "None"$
- $"decomp" ("Lam" \_ #h \_) = "None"$
- $ "decomp" ("App" e_1 #h e_2) = &kw("if") "is-value"(e_1) \& "is-value"(e_2) #kw("then") ("App" e_1 #h e_2, kw("fun") x => x) \
&kw("else-if") "is-value"(e_1) kw("then") \
  &#ind kw("let") (r, f) = "decomp"(e_2) \
  &#ind (r, kw("fun") x => "App" e_1 #h f(x)) \
&kw("else") \
  &#ind kw("let") (r, f) = "decomp"(e_1) \
  &#ind (r, kw("fun") x => "App" f(x) #h e_2) \
$

decomp の右側にある $"Exp" -> "Exp"$ について、どんな関数でも現れるわけではないことに注意し、より適切なデータ型を与えると、それが評価文脈である。
$ "Cxt" ::= "Hole" | "EvalL" (e: "Exp", E: "Cxt") | "EvalR" (v: {e: "Exp" | "is-value" e}, E: "Cxt") $
また redex を表すデータ型を次のように与える。
$ "Red" ::= "Red" (x: "variable", e: "Exp", v: {e: "Exp" | "is-value"(e)}) $
redex をラムダ項に戻す関数 $"as-lam": "Red" -> "Exp"$ とラムダ項が redex かを判定する関数 $"is-lam": "Exp" -> "Option" "Red"$ がつくれる。
また、上記で定義した decomp は次のような写像になる。
- $"decomp": "Exp" -> "Option" ("Red", "Cxt")$
- $"decomp" ("Var" \_) = "None"$
- $"decomp" ("Lam" \_ #h \_) = "None"$
- $"decomp" ("App" e_1 e_2) = $
