# goto-lang 言語（ざっくり）
プログラミング言語として goto-lang 言語というものを見てみる。
この言語は、変数を 
- 初期化（０を入れる）する
- 1 だけ加算する
- 1 だけ減算する
- 値をコピーする

ような単純な文に加え、
`ifz` と呼ばれる、指定された変数の内容が \(0\) **でない** なら指定された行に飛ぶ機能がある。
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

# goto-lang 言語（ちゃんとした解説）

## 定義
変数の集合 \(V\) を固定しておく。
goto-lang 言語は次の形で与えられる。

goto-lang の文
  :  言語の文は以下のものとする。
    - 変数 \(s \in V\) に対して、 increment \(s\), decrement \(s\), reset \(s\)
    - \(2\) つの変数 \(s_1, s_2 \in V\) に対して、 copy \(s_1, s_2\)
    - 変数 \(s \in V\) と自然数 \(n \in \Nat\) に対して、 ifnz \(s, n\)

goto-lang のプログラム
  : これは文の集合

> [!Note]
> 具体例の部分でわかるように、copy はいらなそう。

環境
  : \(V \to \Nat\) のこと
代入
  : 環境 \(s\) と変数 \(x\) および自然数 \(n\) に対して、
    環境 \(s[x \rightarrow n]\) を、変数 \(x\) に対しては \(n\) とし、
    それ以外の変数 \(y\) では \(s(y)\) として定める。

goto-lang 言語に意味を与える。
まあ適当にやればいい。
