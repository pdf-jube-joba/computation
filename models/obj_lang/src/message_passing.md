[[Model]] [[Actor Model]]

tick ごとに変化していくオブジェクトの集合を考える。
tick ごとにメッセージを受け取って、処理をして、出力されるメッセージの全体を考えて、伝播させる。
"オブジェクトに対して"ではなくて、全体にメッセージを流すことにする。
値として用いるのはビット列にする。
- 各 stmt は停止するようにループを使わない。（分岐はすることに注意。）
- こんな感じで進行していく
    1. invoke されるメソッドを最初に決定する（メッセージとそのままメソッドを対応させる）
    2. 内部の stmt を実行する（オブジェクトの状態が変化する・外部に送られるメッセージが決定する。）
    3. stmt を実行した結果として得られるメッセージを。
- new で他のオブジェクトを作り出し、 drop で自分自身を破棄する。

内部状態としての変数環境は tick ごとにリセットされないものとする。
引数としてはメッセージについてくる bit 列1つを持つ。
また、 world というものを導入して、メッセージの流入出を制限しつつ入れ子を作る。

> [!Note]
> 他のオブジェクトに言及することができないので、 identity の話がいらない。

# message passing and actor model

\[
\begin{aligned}
\\
\NT{bit} &\defeq 0 | 1 \\
\NT{message} &\defeq \NT{string} \NT{bit}* \\
\\
\NT{expr} &\defeq \NT{bit} \\
    &| \T{\LSB} \NT{bit}* \T{\RSB} &\syntaxname{constant bit sequence} \\
    &| \NT{var}
    &| \T{tail} \NT{expr} \\
    &| \T{if-empty} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{end} \\
    &| \T{if-head-0} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{end} \\
    &| \T{if-head-1} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{end} \\
\\
\NT{stmt} &\defeq \\
    &| \NT{var} \T{=} \NT{expr}  &\syntaxname{self modify} \\
    &| \T{send} \NT{string} \NT{bit}*            &\syntaxname{send message} \\
    &| \T{if-head-0} \NT{expr} \T{then} \NT{stmt}* \T{end} \\
    &| \T{if-head-1} \NT{expr} \T{then} \NT{stmt}* \T{end} \\
    &| \T{new} \NT{object} \\
    &| \T{drop-this} \\
\\
\NT{var-decl} &\defeq \T{var} \NT{var} \\
\NT{recv-decl} &\defeq \T{recv} \NT{string} \NT{var} \T{=>} \T{\LSB} \NT{stmt}* \T{\RSB} \\
\NT{object} &\defeq \T{\LSB} \\
    &\syntaxmacro{semicolon-separated}{\NT{var-decl}} \T{;} \syntaxmacro{semicolon-separated}{\NT{recv-decl}} \T{;} \\
    &\T{\RSB} \\
\\
\NT{income} &\defeq  \\
\NT{outcome} &\defeq \\
\NT{world} &\defeq \T{\LSB} \\

    &\T{\RSB} \\
\\
\end{aligned}
\]