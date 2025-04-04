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
  import { load, add_tape } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await load();
  document.dispatchEvent(new Event("wasm-ready"));
</script>

`SVG.js` の呼び出し↓
<div id="svg_test1">
<button id="left1"> left </button>
<button id="right1"> right </button>
<script type="module">
  import { ready, add_tape } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await ready;
  add_tape("svg_test1", "1,2,3", "4", "5,6,7", "left1", "right1");
</script>
</div>

もう一つ

<div id="svg_test2">
<button id="left2"> left </button>
<button id="right2"> right </button>
<script type="module">
  import { ready, add_tape } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await ready;
  add_tape("svg_test2", "a,b,c", "d", "e,f,g", "left2", "right2");
</script>
</div>

呼び出しここまで

`mdbook-katex` を用いた、 `katex` での数式の表記： \(x = 1, y = 2\)
