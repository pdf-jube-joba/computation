[[Model]]

expr に加えて stmt を入れる場合のメモ
## 構文
\[
\begin{aligned}
\NT{var} &\defeq \NT{string} \\
\NT{binop} &\defeq \T{+} | \T{-} | \T{\&\&} \\
\NT{unop} &\defeq \T{inc} | \T{dec} | \T{not} \\
\\
\NT{expr} &\defeq \\
    &| \T{\LP} \NT{expr} \T{\RP} & \syntaxname{結合順序の制御用} \\
    &| \N | \T{\#true} | \T{\#false} | \T{\#unit} \\
    &| \NT{var} \\
    &| \NT{expr} \NT{binop} \NT{expr} \\
    &| \NT{unop} \NT{expr} \\
    &| \T{fun} \NT{var} \T{=>} \NT{expr} \\
    &| \T{rec} \NT{var} \NT{var} \T{=>} \NT{expr} \\
    &| \NT{expr} \T{\LP} \NT{expr} \T{\RP} \\
    &| \T{if} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{fi} \\
    &| \T{\LCB} \NT{stmt} \T{;} \NT{expr} \T{\RCB} \\
\\
\NT{stmt} &\defeq \\
    &| \T{skip} \\
    &| \T{print} \N &\syntaxname{呼び出し順序確認用} \\
    &| \NT{stmt} \T{;} \NT{stmt} \\
    &| \T{let} \NT{var} \T{:=} \NT{expr} \\
    &| \T{if} \NT{expr} \T{then} \NT{stmt} \T{end} \\
    &| \T{while} \NT{expr} \T{do} \NT{stmt} \T{end} \\
\end{aligned}
\]

## 意味論
単体で reduction により意味論を与えることを考えるのは難しいので、 CEK をそのまま意味論に採用する。
`let` のスコープは \(\T{\{} s \T{;} e \T{\}}\) の block の中だけとする。

## CEK
[CEK](./CEK.md) と同じく、 control, environment, kontinuation による抽象機械をそのまま意味論とみなす。
`expr` と `stmt` で相を分けて、
`expr` は `eval-expr` から `return` に進み、
`stmt` は `eval-stmt` から `done` に進むようにする。

- \(\text{Cls}\) := \(\text{Fun}(x, M, E) + \text{Rec}(f, x, M, E)\)
- \(V\) := \(\N + \T{\#true} + \T{\#false} + \T{\#unit} + \text{Cls}\)
- \(E\) := \(\text{list}(\NT{var}, V)\)
    - lookup は先頭から順に探す
    - `let x := v` は先頭に \((x, v)\) を追加する
- \(K\) := \(\text{list}K_f\) where \(K_f\) :=
    - \(([] \NT{binop} M, E)\)
    - \((V \NT{binop} [], E)\)
    - \((\NT{unop} [], E)\)
    - \(([] (N), E)\)
    - \((f \in \text{Cls})[]\)
    - \((\T{if} [] M N, E)\)
    - \((\T{let} x \T{:=} [], E)\)
    - \(([] ; s, E)\)
    - \((\T{if} [] \T{then} s \T{end}, E)\)
    - \((\T{while} M \T{do} s \T{end}, E)\)
    - \(\text{scopeout}(M)\) ... `stmt` が終わったら、そのときの環境で \(M\) を評価する
    - \(\text{init}\)
- \(S\) := \(\text{eval-expr}(\NT{expr}, E, K) + \text{return}(V, K) + \text{eval-stmt}(\NT{stmt}, E, K) + \text{done}(E, K)\)

ここで、 \(\text{return}(v, K)\) は `expr` が値 \(v\) に評価された状態で、
\(\text{done}(E, K)\) は `stmt` の実行が終わって環境 \(E\) だけが更新された状態である。

遷移は次の通り：

\[
\begin{aligned}
\text{eval-expr}(v, E, K) &\to \text{return}(v, K) && v = n \in \N | \T{\#true} | \T{\#false} | \T{\#unit} \\
\text{eval-expr}(\T{fun} x \T{=>} M, E, K) &\to \text{return}(\text{Fun}(x, M, E), K) \\
\text{eval-expr}(\T{rec} f x \T{=>} M, E, K) &\to \text{return}(\text{Rec}(f, x, M, E), K) \\
\\
\text{eval-expr}(x, E, K) &\to \text{return}(\text{lookup}(E, x), K) \\
\text{eval-expr}(M \NT{binop} N, E, K) &\to \text{eval-expr}(M, E, ([] \NT{binop} N, E)::K) \\
\text{eval-expr}(\NT{unop} M, E, K) &\to \text{eval-expr}(M, E, (\NT{unop} [], E)::K) \\
\text{eval-expr}(M(N), E, K) &\to \text{eval-expr}(M, E, ([] (N), E)::K) \\
\text{eval-expr}(\T{if} L M N, E, K) &\to \text{eval-expr}(L, E, (\T{if} [] M N, E)::K) \\
\text{eval-expr}(\T{\{} s \T{;} e \T{\}}, E, K) &\to \text{eval-stmt}(s, E, \text{scopeout}(e)::K) \\
\\
\text{return}(v, ([] \NT{binop} M, E_0)::K) &\to \text{eval-expr}(M, E_0, (v \NT{binop} [], E_0)::K) \\
\text{return}(v, (v' \NT{binop} [], E_0)::K) &\to \text{return}(v'', K) && v'' = v' \NT{binop} v \\
\text{return}(v, (\NT{unop} [], E_0)::K) &\to \text{return}(v', K) && v' = \NT{unop} v \\
\\
\text{return}(f \in \text{Cls}, ([] (N), E_a)::K) &\to \text{eval-expr}(N, E_a, (f [])::K) \\
\text{return}(v, (\text{Fun}(x, M, E_f) [])::K) &\to \text{eval-expr}(M, (x, v)::E_f, K) \\
\text{return}(v, (\text{Rec}(f, x, M, E_f) [])::K) &\to \text{eval-expr}(M, (x, v)::(f, \text{Rec}(f, x, M, E_f))::E_f, K) \\
\\
\text{return}(\T{\#true}, (\T{if} [] M N, E_0)::K) &\to \text{eval-expr}(M, E_0, K) \\
\text{return}(\T{\#false}, (\T{if} [] M N, E_0)::K) &\to \text{eval-expr}(N, E_0, K) \\
\\
\text{eval-stmt}(\T{skip}, E, K) &\to \text{done}(E, K) \\
\text{eval-stmt}(\T{print} n, E, K) &\to^{\text{print} n} \text{done}(E, K) \\
\text{eval-stmt}(s_1 \T{;} s_2, E, K) &\to \text{eval-stmt}(s_1, E, ([] ; s_2, E)::K) \\
\text{eval-stmt}(\T{let} x \T{:=} M, E, K) &\to \text{eval-expr}(M, E, (\T{let} x \T{:=} [], E)::K) \\
\text{eval-stmt}(\T{if} M \T{then} s \T{end}, E, K) &\to \text{eval-expr}(M, E, (\T{if} [] \T{then} s \T{end}, E)::K) \\
\text{eval-stmt}(\T{while} M \T{do} s \T{end}, E, K) &\to \text{eval-expr}(M, E, (\T{while} M \T{do} s \T{end}, E)::K) \\
\\
\text{return}(v, (\T{let} x \T{:=} [], E_0)::K) &\to \text{done}((x, v)::E_0, K) \\
\text{return}(\T{\#true}, (\T{if} [] \T{then} s \T{end}, E_0)::K) &\to \text{eval-stmt}(s, E_0, K) \\
\text{return}(\T{\#false}, (\T{if} [] \T{then} s \T{end}, E_0)::K) &\to \text{done}(E_0, K) \\
\text{return}(\T{\#true}, (\T{while} M \T{do} s \T{end}, E_0)::K) &\to \text{eval-stmt}(s, E_0, ([] ; \T{while} M \T{do} s \T{end}, E_0)::K) \\
\text{return}(\T{\#false}, (\T{while} M \T{do} s \T{end}, E_0)::K) &\to \text{done}(E_0, K) \\
\\
\text{done}(E, ([] ; s, E_0)::K) &\to \text{eval-stmt}(s, E, K) \\
\text{done}(E', \text{scopeout}(M)::K) &\to \text{eval-expr}(M, E', K) \\
\end{aligned}
\]

これで、 block の中で入った `let` 束縛は block の評価中だけ使われ、
block を抜けたあとの環境復元は外側の continuation frame がもともと持っている環境で行われる。
最終的に \(\text{return}(v, \text{init})\) になったとき、それを結果と思えばよい。

> [!note]
> `expr` の側では `return` は値だけを持ち、環境の復元は continuation frame が担う。
> したがって、 `scopeout` 自身は外側環境を持たなくてもよく、
> `done` で返ってきた環境をそのまま使って block 末尾の `expr` を評価すればよい。
