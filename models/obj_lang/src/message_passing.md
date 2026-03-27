[[Model]] [[Actor Model]]

# message passing and actor model
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
引数としてはメッセージについてくる bit 列1つを特殊な変数として持つ。
また、 world というものを導入して、メッセージの流入出を制限しつつ入れ子を作る。
メッセージはタグが重複するものも許し、重複する分呼び出す。
順序は実行順に list にする。（ hashmap ではないイメージ。）

> [!Note]
> 他のオブジェクトに言及することができないので、 identity の話がいらない。

## 構文

\[
\begin{aligned}
\\
\NT{bit} &\defeq 0 | 1 \\
\NT{bitseq} &\defeq \NT{bit}* \\
\NT{message} &\defeq \NT{string} \NT{bitseq} \\
\\
\NT{expr} &\defeq \NT{bit} \\
    &| \T{\LSB} \NT{bitseq} \T{\RSB} &\syntaxname{constant bit sequence} \\
    &| \NT{var} \\
    &| \NT{\# payload} \\ 
    &| \T{tail} \NT{expr} \\
    &| \T{if-empty} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{end} \\
    &| \T{if-head-0} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{end} \\
    &| \T{if-head-1} \NT{expr} \T{then} \NT{expr} \T{else} \NT{expr} \T{end} \\
\\
\NT{stmt} &\defeq \\
    &| \NT{var} \T{=} \NT{expr} &\syntaxname{assign} \\
    &| \T{send} \NT{string} \NT{expr} &\syntaxname{send message} \\
    &| \T{new} \NT{object} &\syntaxname{object creation}\\
    &| \T{finally-drop-this} &\syntaxname{drop} \\
\\
\NT{var-decl} &\defeq \T{var} \NT{var} \\
\NT{recv-decl} &\defeq \T{recv} \NT{string} \T{=>} \T{\LSB} \NT{stmt}* \T{\RSB} \\
\NT{object} &\defeq \T{\LSB} \\
    &\syntaxmacro{semicolon-separated}{\NT{var-decl}} \T{;} \syntaxmacro{semicolon-separated}{\NT{recv-decl}} \T{;} \\
    &\T{\RSB} \\
\\
\NT{world} &\defeq \syntaxmacro{comma-separated}{(\T{income} | \T{outcome}) \NT{string}} \T{\LCB} \\
    &(\NT{world} | \NT{object})* \\
    &\T{\RCB} \\
\\
\end{aligned}
\]

## 意味論
オブジェクト単体の意味論の定義はこんな感じ：
- \(O\) := \(\text{list}(\NT{var}, \NT{bitseq}) * \text{list}(\NT{string}, \text{list}(\NT{stmt}))\)
- \(W\) := \(\text{list}(W | O) * \text{Income}(\text{list}(\NT{string})) * \text{Outcome}(\text{list}(\NT{string})) * \text{State}(\text{list}(\NT{message}))\)
- \(\text{eval-expr}(o: O, e: \NT{expr}, a: \NT{bitseq}): \NT{bitseq}\) :=
    - \(\T{\LSB} c \T{\RSB}\) ならそのまま
    - \(\NT{var}\) なら \(o\) から探す
    - \(\T{\# payload}\) なら \(a\) のこと
    - \(\T{tail} e\) なら \(\text{eval-expr}(e)\) に対して、 \([] \mapsto [], xs::xs \mapsto xs\) とする。
    - \(\T{if-empty} L M N\) なら \(L == []\) で分岐
    - \(\T{if-head-0} L M N\) なら、 \(\text{eval-expr}(e) == 0::xs, \exists xs\) で分岐
    - \(\T{if-head-1} L M N\) なら、 \(\text{eval-expr}(e) == 1::xs, \exists xs\) で分岐
- \(\text{eval-stmt}(o: O, s: \NT{stmt}, a: \NT{bitseq}): \text{option}(\NT{message}) * \text{option}(\NT{object}) * (\text{drop} | \text{non-drop})\) := 疑似コードで書く（めんどくさいので）
    ```rust
    match s {
        Stmt::Assign(v, e) => {
            let bs = o.eval_expr(e, a)?;
            o.set(v, bs)?;
            (None, None, false)
        }
        Stmt::Send(s, e) => {
            let bs = o.eval_expr(e, a)?;
            (Some(Message(s, bs)), None, false)
        }
        Stmt::New(obj_decl) => {
            (None, Some(obj_decl), false)
        }
        Stmt::Drop => {
            (None, None, true)
        }
    }

    ```
    - assign ： \(o: O\) のうち左辺の var を \(\text{eval-expr}(e)\) の結果で置き換える。
    - send ： eval-expr して最終的に得られた \(\NT{message}\) を結果のリストに加える
    - new: 最終結果の object に加える
    - drop: 書かれると \(\text{drop}\) を結果に入れる。この文が実行されなかったら、最後の結果は \(\text{nondrop}\) とする。 **tick 終了時に drop を行うので、この文以降も、他の recv もこの文が書かれた tick では実行すること**
- \(\text{tick-object}(o: O, m: \text{list}(\NT{message})): \text{list}(\NT{message}) * \text{list}(\NT{object}) * (\text{drop} | \text{non-drop})\) := 疑似コードで書く（めんどくさいので）
    ```rust
    let mut drop = false;
    let mut msgs = vec![];
    let mut objs = vec![];
    for (t, a) in ms {
        for (t_2, s) in o.recv {
            if t != t_2 {
                continue;
            }
            let (msgs_2, objs_2, dr) = o.eval_stmt(s, a);
            drop = drop || dr;
            msgs.extend(msgs_2);
            objs.extend(objs_2);
        }
    }
    ```
- \(\text{tick-world}(w: W, m: \text{list}(\NT{message})): \text{list}(\NT{message})\) := 
    1. \(m\) のうち income に含まれているタグのものだけを filter して、自分の State に加える...今回 World 内で響くメッセージの集合 \(m'\) がこれで得られる。
    2. 次の World で響くメッセージの集合 （State 用）の list \(m''\) を空で用意し、追加するオブジェクトの一覧 \(os\) も用意する。
    3. 各オブジェクトと World ごとに \(m'\) で tick を呼び出す。
        - World の場合は出てくるのは message だけなので、これを \(m''\) に加える。
        - Object の場合は、出てきたメッセージは \(m''\) に加えて追加するオブジェクトも \(os\) に加える。
            drop についてはまだ処理をせず、目印をつけておく
    4. 追加するオブジェクト一覧を追加し、削除するオブジェクト一覧を削除する。
    5. \(os\) を State としてセットし、 Outcome として記述されているものだけをそこから filter して返す。

あくまでも疑似コードなので、これでそのまま書くとコンパイルが所有権の関係でできなかったりしそう。
