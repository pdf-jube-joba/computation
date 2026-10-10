# CLI

OCaml のモデルを直接リンクしたネイティブ CLI です。Web の bundle や Wasmtime は不要です。
リポジトリルートで実行します。

```sh
opam exec -- dune exec computation -- list
opam exec -- dune exec pipe-machine -- model example_counter create --code 5
```

`pipe-machine` の create は snapshot JSON を標準出力に書きます。
step は標準入力の snapshot を復元して一段階実行し、継続時は次の snapshot を標準出力へ、
実行時出力・停止時出力を標準エラーへ書きます。停止時の標準出力は空です。

```sh
printf '%s\n' '{"count":5}' | opam exec -- dune exec pipe-machine -- model example_counter step --rinput inc
```

描画用 JSON は `model NAME render` で標準入力の snapshot から取得できます。

対話実行は以下です。Code / AInput を省略すると入力を求めます。

```sh
opam exec -- dune exec repl-machine -- model lambda_calculus --code '(\x. x) (\y. y)' --ainput ''
```

RInput を一行ずつ入力します。`:snapshot` は状態の出力、`:quit` は終了、
`:begin` ～ `:end` は複数行の入力です。Code / AInput の入力時にも使えます。
エラーでも状態を保持するので再入力できます。

コンパイラも同じ registry から選べます。

```sh
opam exec -- dune exec pipe-machine -- compiler compile-code recursive_function-goto_lang --code SUCC
opam exec -- dune exec pipe-machine -- compiler encode-ainput recursive_function-goto_lang --ainput '(3)'
```

操作は `compile-code`、`encode-ainput`、`encode-rinput`、`decode-routput`、`decode-foutput`。
以前の `compile-ainput` / `compile-rinput` も別名として受け付けます。
