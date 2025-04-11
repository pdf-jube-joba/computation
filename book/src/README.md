# ブラウザで動かしながら色々する

この本は、計算モデルの話を動いているのを見つつ楽しむためのもの。
目標は、

- 計算モデルについて（数学＋α）を書く。
- あまりがっちりと証明は書かない。
- インタプリタとかコンパイラを動かす。

# テスト

## SVG.js と wasm の連携コード
`src/assets/test.js` を読み込んで、`SVG.js` で書くテスト。
```
<script type="module">
  import { write_rect } from "./assets/test.js";
  await write_rect();
</script>
```

<script type="module">
  import { load } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await load();
  document.dispatchEvent(new Event("wasm-ready"));
</script>

`SVG.js` の呼び出し↓
<div id="svg_test1">
<button id="left1"> left </button>
<button id="right1"> right </button>

<script type="module">
  import { ready, tape_init } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await ready;
  let tape_reload = tape_init("svg_test1", "left1", "right1");
  tape_reload("1,2,3 | 4 | 5,6,7");
</script>
</div>

もう一つ

<div id="svg_test2">
<button id="left2"> left </button>
<button id="right2"> right </button>

<script type="module">
  import { ready, tape_init } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await ready;
  let tape_reload = tape_init("svg_test2", "left2", "right2");

  const res = await fetch("./assets/component/test_global_tape/tape.txt");
  const text = await res.text();
  tape_reload(text);
</script>
</div>

ユーザーが入力するものも。

<div id="svg_test3">
<button id="left3"> left </button>
<button id="right3"> right </button>
<button id="load"> load </button>
<textarea id="user_defined" rows="1"></textarea>

<script type="module">
  import { ready, tape_init } from "./assets/generated/test_global_tape/test_global_tape_glue.js";
  await ready;
  let tape_reload = tape_init("svg_test3", "left3", "right3");
  
  document.getElementById("load").addEventListener("click", () => {
      const tape_str = document.getElementById("user_defined").value.trim();
      tape_reload(tape_str);
    });
</script>
</div>

呼び出しここまで

## `utils.js` を呼び出す
<script type="module">
  import { TextAreaSource, UserControls } from "./assets/utils.js";
  let textinput = new TextAreaSource("test_code");
  let controlinput = new UserControls("test_control");
  controlinput.setOnLoad(() => {console.log("load")});
  controlinput.setOnStep(() => {console.log("step")});
  controlinput.time_interval = 1000;
</script>
<textarea id="test_code"> </textarea>
<div id="test_control"> </div>

`mdbook-katex` を用いた、 `katex` での数式の表記： \(x = 1, y = 2\)
