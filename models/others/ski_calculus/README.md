# SKI 計算

項はコンビネータ `S`・`K`・`I`、自由変数、二項の適用で構成する。
適用は左結合で、`S K K x` は `(((S K) K) x)` を表す。
`S`・`K`・`I` だけからなる語は一文字ずつ読み、`SKK x` と `S K K x` は同じ項になる。
括弧は結合を変えるために使うので、`S(KI)` と `SKI` は異なる。
それ以外の識別子は自由変数で、値の検索や束縛は行わない。
例えば `foo` は一つの変数、`Sfoo` も一つの変数。

## 簡約

```text
I x     → x
K x y   → x
S x y z → x z (y z)
```

一段階では左端最外の redex を一つだけ簡約する。
項全体に規則が適用できればそれを優先し、そうでなければ左の部分項、右の部分項の順に探す。
引数が不足するコンビネータや自由変数はそれ自体では簡約しないが、その引数の内部も探索する。
例えば `K (I x)` は `K x` まで進む。

```text
S K K x → K x (K x) → x
```

`K x (S I I (S I I))` は一段階で `x` になる。
不要な第二引数を先に評価しないため、その部分が発散しても結果を得られる。
簡約可能な部分がなくなったら、その項を正規形として返す。
正規形を持たない項は step を続けられる。

## 実行

モデル名は `ski_calculus`。Code に項を入れる。
AInput・RInput・ROutput は空文字列で、FOutput は正規形。
保存する状態は現在の項そのもので、描画にも現在の項を表示する。

<div data-model="ski_calculus">
<template data-default-code>
S K K x
</template>
</div>

```sh
opam exec -- dune exec repl-machine -- model ski_calculus --code 'S K K x' --ainput ''
```

空行ごとに一つ簡約し、上の例は二回の簡約後、次の step で `x` を返す。
