# mdbook と `<script>` タグについて
mdbook では、 book.toml に次のように書くと自動で生成時に追記してくれる。
```book.toml
[output.html]
additional-js = [
    "assets/vendor/svg.js",
]
```
こう書くと、生成される `html` の全部で、 `<script src="assets/vendor/svg.js"> </script>` とかがついている。
ただ、 `type="module"` はつかないので、これでやる場合は気を付けること。
つまり、 `export` をするような `js` ファイルはこれで読み込んじゃダメ。

# html のパーサーの問題？
markdown の中でタグを配置するときの注意
```markdown
<div id="canvas"></div>

<script type="module">
  import init, { parse } from "./assets/generated/turing_machine_web.js";
  import { load } from "./assets/generated/turing_machine_glue.js";

  async function run() {
    await init();       // wasm の初期化
    load(parse);        // SVG描画を行う
  }

  run();
</script>
```
- `<div>` と `<script>` の間をあけること
- `<script>` の中に改行を入れないこと
どちらかも破ると、html がイメージ通りにできない。
（上で試してみるといい）
