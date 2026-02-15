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
  : 言語の文は以下のものとする。
    - 変数 \(s \in V\) に対して、
      - \(\text{inc} s\): \(s\) に1を加算する
      - \(\text{dec} s\): \(s\) から1を減算する
      - \(\text{clr} s\): \(s\) を \(0\) にする
    - \(2\) つの変数 \(s_1, s_2 \in V\) に対して、
      - \(\text{copy} s_1, s_2\): \(s_2\) の値を \(s_1\) に代入する。
    - 変数 \(s \in V\) と自然数 \(n \in \Nat\) に対して、
      - ifnz \(s, n\): \(s\) に入っている値が \(0\) のとき、 \(n\) 行目のコードに移る。

> [!Note]
> 具体例の部分でわかるように、copy はいらなそう。

goto-lang のプログラム
  : これは文の集合

環境
  : \(V \to \Nat\) のこと

代入
  : 環境 \(s\) と変数 \(x\) および自然数 \(n\) に対して、
    環境 \(s[x \rightarrow n]\) を、変数 \(x\) に対しては \(n\) とし、
    それ以外の変数 \(y\) では \(s(y)\) として定める。

goto-lang 言語に意味を与える。
まあ適当にやればいい。
