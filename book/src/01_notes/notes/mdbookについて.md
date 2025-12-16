## mdbook と `<script>` タグについて
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

## 見た目が壊れている問題
こんな感じのものを書くと壊れた。
```md
term
  : The quick brown fox jumps over the lazy dog. \\(x\\)
   Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam
  : The quick brown fox jumps over the lazy dog. \\(x\\)
   Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam
```
見た目が、数式を1カラムとして3カラムの縦になってしまう。
- mathjax （ mdbook のデフォルト）でもだめ
- term の定義が2つ以上あって数式が入っているときに起きる

理由：
- definition-list は専用の html として `<dd>`, `<dt>`, `<dl>` があってそこに変換される。
- 数式は mathjax でも `<span>` を使うことになる。
- css で `display: flex` を指定すると、 `<span>` のせいで3つと認識される？

```css general.css
/* When there is more than one definition, increment the counter. The first
selector selects the first definition, and the second one selects definitions
2 and beyond.*/
dd:has(+ dd), dd + dd {
    counter-increment: dd-counter;
    /* Use flex display to help with positioning the numbers when there is a p
    tag inside the definition. */
    display: flex;
    align-items: flex-start;
}
```
`custom.css` を書いて解決した。

term
  : The quick brown fox jumps over the lazy dog. \(x\)
   Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam
  : The quick brown fox jumps over the lazy dog. \(x\)
   Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam
  
普通に長い場合

term
  : short
  : Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

## 定義と定理について
definition-list の中で listing を使うべきではないという指摘を（AIから）受けました。

今の使い方
```
定義というよりはラベル付け？
: 定義の前提条件とか
  - 必要な前提条件の列挙1
  - 必要な前提条件の列挙2
  - 「次の条件を満たす」
    - foobar
```

確かに、用語の定義みたいな使い方ができてない。
