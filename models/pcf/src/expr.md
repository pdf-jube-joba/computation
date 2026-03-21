[[Model]]
[[Compiler]]

これから以下をコンパイル云々ができるようにしていきたいが、
コンパイル前後で `ROutput` 出力の順序が保たれているかどうかは気にするが、その step 数は気にしない。

## 構文
\[
\begin{aligend}
\NT{var} &\defeq \NT{string} \\
\NT{binop} &\defeq \T{+} | \T{-} | \T{&&} \\
\NT{unop} &\defeq \T{inc} | \T{dec} | \T{not} \\
\\
\NT{expr} &\defeq \\
  &| \T{\LP} \NT{expr} \T{\RP}
  &| \N | \T{\#true} | \T{\#false} | \T{\#unit} \\
  &| \T{print} \N &\syntaxname{呼び出し順序確認用} \\
  &| \NT{var} \\
  &| \NT{expr} \NT{binop} \NT{expr} \\
  &| \NT{unop} \NT{expr} \\
  &| \T{fun} \NT{var} \T{=>} \NT{expr} \\
  &| \T{rec} \NT{var} \NT{var} \T{:=} \NT{expr} \\
  &| \NT{expr} \T{\LP} \NT{expr} \T{\RP} \\
  &| \T{if} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{fi} \\
  &| \T{case} \NT{expr} \T{then} \NT{expr} \T{else} \NT{var} \T{=>} \NT{expr} \T{esac} &\syntaxname{primitive recursion} \\
\end{aligned}
\]

## 意味論
reduction の列で意味論が定義できる。
値渡しにして、 \(\T{\print} \N\) は `ROutput` に表示しつつ \(\T{\#unit}\) を返す。
- if と case では分岐判定部分を簡約して、値が得られたらそのまま適用する。分岐先は簡約しない。
- 関数適用だけ違って、関数部分も引数部分も値にすること。
- `AInput` と `FOutput` は素直に \(\N\) と考えていい。最終的な reduction の結果が \(\N\) 以外だったら stuck として、 step の Err になる。

### ちゃんとした定義？
値：\(V \subset M\) := \(\N + \mathbb{B} + 1 + \NT{var} + (\T{fun} x M)\)
