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
