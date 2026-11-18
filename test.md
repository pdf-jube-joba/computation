markdown がうまく行っているかテストする。

## katex で数式
### ヘッダーに数式を書く： \(1\)
`katex` での数式の表記： \(x = 1, y = 2\), 数式内のテキストは\(\text{日本語}\)みたいになる。

ネストする例：`\(\text{ \(1 + \text{if \(n\) is zero then \(2\) else \(k\) }\) }\)`
こう書くと、\(\text{ \(1 + \text{if \(n\) is zero then \(2\) else \(k\) }\) }\) になる。

```
\(\text{ \(1 + \text{if \(n\) is zero then \(2\) else \(k\) }\) }\)
```

\(a\_b,  c\_d\)

bnf っぽいものを書く用のやつ

\[
\begin{aligned}
\NT{stmt} &\defeq \NT{var} \sp \T{add} \sp \NT{var} \\
\NT{var}  &\defeq \NT{ident}
\end{aligned}
\]

## link index の自作
[[テスト]]

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

term
: The quick brown fox jumps over the lazy dog. \(x\)
   Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam
: The quick brown fox jumps over the lazy dog. \(x\)
   Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam
: `mdbook` だと崩れる。

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
