markdown がうまく行っているかテストする。

## モデル表示のテスト
全部を統一したい。

<div data-model="example_counter">
<script type="text/plain" class="default-code">
5
</script>
<script type="text/plain" class="default-rinput">
inc
</script>
</div>

二個目の配置
<div data-model="example_counter">
<script type="text/plain" class="default-code">
8
</script>
<script type="text/plain" class="default-rinput">
inc
</script>
</div>

identity compiler の場合

<div data-model="example_counter-example_counter">
<script type="text/plain" class="default-code">
8
</script>
<script type="text/plain" class="default-rinput">
inc
</script>
</div>

## katex で数式
### ヘッダーに数式を書く： \(1\)
`mdbook-katex` を用いた、 `katex` での数式の表記： \(x = 1, y = 2\)

\(\text{日本語}\)

bnf っぽいものを書く用のやつ

\(\begin{aligned}
\NT{stmt} &\defeq \NT{var} \sp \T{add} \sp \NT{var} \\
\NT{var}  &\defeq \NT{ident}
\end{aligned}\)

## markdown 
### 定義リスト

term
  : 定義をここに書く
  : 複数行
  書くこともできる（空白が入る扱い？）

listing
  : 定義の中で listing をしたい
    - a
    - b

### github 方式の admonition
> [!note]
> admonition がデフォルトで入った。

```
> [!note]
> このように書く
```

`!NOTE`,`!TIP`,`!IMPORTANT`,`!WARNING`,`!CAUTION`
が使える。

## table, inline code, `|` のとき
table を書き、その中で inline code を書き、その inline code に `|` が含まれているとき、 `\|` のように escape を行う。
```
|  a  |  b  |
| --- | --- |
|  c  | `x \|> f` |
```

| a | b |
| --- | --- |
| c | `x \|> f` |

## html のパーサーの問題？
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
