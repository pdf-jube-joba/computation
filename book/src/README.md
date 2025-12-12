# ブラウザで動かしながら色々する

この本は、計算モデルの話を動いているのを見つつ楽しむためのもの。

- 計算モデルについて（数学＋α）を書く。
- あまりがっちりと証明は書かない。
- インタプリタとかコンパイラを動かす。

プログラミング言語関連のメモは全部こっちに移すこと。

## モデル表示のテスト
全部を統一したい。
- create と step で全部なんとかする。

<div data-model="example">
<script type="text/plain" class="default-code">
5
</script>
<script type="text/plain" class="default-rinput">
increment
</script>
</div>

二個目の配置
<div data-model="example"> </div>

## katex で数式
### ヘッダーに数式を書く： \(1\)
`mdbook-katex` を用いた、 `katex` での数式の表記： \(x = 1, y = 2\)

## mdbook の新しい拡張
term
  : 定義をここに書く
  : 複数行
  書くこともできる（空白が入る扱い？）

listing
  : 定義の中で listing をしたい
    - a
    - b

- c

> [!TIP]
> admonition がデフォルトで入った。

`!NOTE`,`!TIP`,`!IMPORTANT`,`!WARNING`,`!CAUTION`
が使える。
