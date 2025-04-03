# ブラウザで動かしながら色々する

この本は、計算モデルの話を動いているのを見つつ楽しむためのもの。
目標は、

- 計算モデルについて（数学＋α）を書く。
- あまりがっちりと証明は書かない。
- インタプリタとかコンパイラを動かす。

# テスト

`src/assets/test.js` を読み込んで、`SVG.js` で書くテスト。
```
<script type="module">
  import { write_rect } from "./assets/test.js";
  await write_rect();
</script>
```

<script type="module">
  import { write_rect } from "./assets/test.js";
  await write_rect();
</script>

`SVG.js` の呼び出し↓
<div id="svg_test"></div>
呼び出しここまで

`mdbook-katex` を用いた、 `katex` での数式の表記： \(x = 1, y = 2\)
