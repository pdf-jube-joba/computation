[[Model]]

- `new` があって object identity の話ができる。
- `this` 付き

## 構文
なんかラムダ計算みたいなのがある。
method と variable を区別してない。

\[
\begin{aligned}
\NT{x} &\defeq \NT{string} \\
\NT{f} &\defeq \NT{string} \\
\\
\NT{expr} &\defeq \\
    &| \T{this} \\
    &| \NT{object-decl} \\
    &| \N | \T{\#true} | \T{\#false} | \T{null} \\
    &| \NT{x} | \NT{x} \T{.} \NT{f} \\
    &| \NT{expr} \NT{expr} \\
    &| \T{fun} \NT{x} \T{=>} \NT{expr} \\
    &| \NT{expr} \NT{binop} \NT{expr} \\
    &| \NT{expr} \T{=} \NT{expr} & \syntaxname{equality of integer} \\
    &| \NT{expr} \T{===} \NT{expr} & \syntaxname{equality of object identity} \\
    &| \T{begin} \NT{stmt}* \T{end} \\
\\
\NT{stmt} &\defeq \\
    &| \T{Nop} \\
    &| \NT{x} \T{:=} \NT{expr} \\
    &| \NT{x} \T{.} \NT{f} \T{:=} \NT{expr} \\
    &| \T{if} \NT{expr} \T{then} \NT{block} \\
    &| \T{while} \NT{expr} \T{then} \NT{block} \\
    &| \T{return} \NT{expr} \\
\\
\NT{field-decl} &\defeq \NT{f} \T{:} \NT{expr} \T{;} \\
\NT{object-decl} &\defeq \T{new} \T{\LSB} \NT{field-decl}* \T{\RSB} \\
\\
\NT{program} &\defeq \NT{object-decl}+ \\
\\
\end{aligned}
\]

## 意味論
program 中で最後に作られたオブジェクトの `.main` を"実行"する。

> [!Note]
> expr を \(E \to (V, E)\) にして、 stmt を \(E \to E\) みたいに解釈する必要がありそう。

ふんわり考える
- object store というものがあってこれは KV である。
- 代入関連は、右辺が object なら K を入れて、そうじゃないならそのまま primitive を入れる
- それをもとにすると、 V は primitive + K + で書ける。
- 難しいのが closure の評価と this の評価。
  - stmt や expr を実行している"主体"みたいなものがあるはずなので、状態として object key を詰んでおく。
  - 
