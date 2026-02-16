# goto-lang 言語（ざっくり）
プログラミング言語として goto-lang 言語というものを見てみる。
この言語は、変数を 
- 初期化（０を入れる）する
- 1 だけ加算する
- 1 だけ減算する
- 値をコピーする

ような単純な文に加え、
`ifz` と呼ばれる、指定された変数の内容が \(0\) **でない** なら指定された行に飛ぶ機能がある。
ちなみに、「\(0\) でなら」の言語を作っても同じぐらいの計算能力になるが、
書きやすかったので、こっちにした。
Turing machine よりはこっちの方が理解しやすそう。

## 具体例
\(x * y\) を計算するプログラム：
<div data-model="goto_lang">
<script type="text/plain" class="default-code">
cpy y2 <- y
inc z
dec y2
ifnz y2 : 1
dec x
ifnz x : 0
</script>
<script type="text/plain" class="default-ainput">
x = 3
y = 4
</script>
</div>
