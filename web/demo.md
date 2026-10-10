# OCaml の計算モデル

既存の Markdown 埋め込みと `assets/renderer.js` を使った OCaml 版です。
それぞれ Create → Step で実行します。下のカウンター二つは独立した状態を持ちます。

## Example counter（Wasm）

<div data-model="example_counter">
<template data-default-code>
3
</template>
<template data-default-rinput>
inc
</template>
</div>

## Example counter（JS）

<div data-model="example_counter_js">
<template data-default-code>
8
</template>
<template data-default-rinput>
dec
</template>
</div>

## Lambda calculus

RInput は 0 始まりの redex 番号です。この例は 0 を選ぶと `\y. y` で停止します。

<div data-model="lambda_calculus">
<template data-default-code>
(\x. x) (\y. y)
</template>
<template data-default-rinput>
0
</template>
</div>
