## チューリングマシンの構成
チューリングマシンの構成のために考察を行う。

### 単体のチューリングマシンに対する操作
議論を楽にするため（だけ）にいろいろ導入する。
チューリングマシン \(M = (\Sigma, Q, q_{\text{init}}, Q_{\text{fin}}, \delta)\) を固定する。

> **theorem**
> - \(\Sigma \subset \Sigma^\prime\) となる \(\Sigma^\prime\) があるとする。
> - \((\Sigma^\prime, Q, q_{\text{init}}, Q_{\text{fin}}, \delta)\) はチューリングマシンである。

\(\delta\) は \(\Sigma^\prime - \Sigma\) の元が入力されると（正常でない）停止をすることに注意する。
これを用いて、 \(M\) とその拡張を同一視することがある。

> **theorem**
> - \(f\): bijection of \(Q \to Q^\prime\) があるとする。
> - \(\delta^\prime\): \(Q^\prime - f(Q_{\text{fin}}) \times \Sigma \to Q^\prime \times \Sigma \times \{L,R,C\}\) を \((q, s) \mapsto (f \times 1 \times 1) \delta(f^{-1}q, s)\) とする。
> - \((\Sigma, Q^\prime, f(q_{\text{init}}), f(Q_{\text{fin}}), \delta^\prime)\) はチューリングマシンである。

> [!NOTE]
> ここでは、 \(\Sigma\) や \(Q\) を変形するような操作をチューリングマシンにしたが、
> チューリングマシンの行う計算の"本質"は変わっていないことに注意したい。
> （計算の"本質"が何であるかは全く議論をしていないが）

これでいつでも必要に応じてチューリングマシンを（その計算内容を変えずに）取り替えることができる。

### チューリングマシンの連結

次に、チューリングマシンをつなげる構成について述べる。
2 つのチューリングマシン \(M_1, M_2\) があったとき、 \(M_1\) を行った後 \(M_2\) を行うチューリングマシンを作りたい。
単に、状態や遷移関数を（集合として）合併した場合、望むような動作ができない。
次のように仮定がある場合には適当な合併によりできる。

> **theorem**
> - \(M_i = (\Sigma_i, Q_i, {q_{\text{init}}}^i, {Q_{\text{fin}}}^i, \delta_i)\) をチューリングマシンとする。
> - \(\Sigma_1 = \Sigma_2 = \Sigma\), \(Q_1 \cap Q_2 = \emptyset\) を仮定する。
> - \(q \in Q_{\text{fin}}^1\) を選ぶ。
> - \(Q = Q_1 \sqcup Q_2\) とする。
> - \(Q_{\text{fin}} = Q_{\text{fin}}^1 \sqcup Q_{\text{fin}}^2 - \{q\}\) とする。
> - \(\delta = \delta_1 \cup \delta_2 \cup (\delta_{\text{glue}} = \{(q, s, {q_{\text{init}}}^2, s, C) \mid s \in \Sigma\})\) とする。
> - \((\Sigma, Q, {q_{\text{init}}}^1, Q_{\text{fin}}, \delta)\) はチューリングマシンである。

証明を行う。
- \(\delta^\prime\) の定義域と値域が \((Q - Q_{\text{fin}}) \times \Sigma \to Q \times \Sigma \times \{L,R,C\}\) におさまっているかどうかを確かめる。
    - 値域については \(\text{Im} \delta = \text{Im} \delta_1 \cup \text{Im} \delta^2 \cup \{q_{\text{init}}^2\} \times \Sigma \times \{L,R,C\}\) から正しい。
    - 定義については、 \(Q - Q_{\text{fin}} = (\sqcup (Q_1 - Q_{\text{fin}}^1)) \cup \{q\}\) に注意すると、 \(\text{dom} \delta = \text{dom}\delta_1 \cup \text{dom} \delta_2 \cup \{q\} \times \Sigma \subset Q_1 - Q_{\text{fin}}^1 \times \Sigma \cup Q_2 - Q_{\text{fin}}^2 \times \Sigma \cup \{q\} \times \Sigma = Q - Q_{\text{fin}} \times \Sigma\) よいよい。
- \(\delta\) が部分関数になっているかを確かめる。
    - 確認するのは \((q, s, \_, \_, \_) \in \delta\) が \((q,s)\) に対して高々一つしかないことである。
    - \(\delta_1, \delta_2 \delta_{\text{glue}}\) の値域がそれぞれ交わらないことに注意すればわかる。

いまこうして作られるチューリングマシンを \(M_1 \rightarrow_{s} M_2\) のように書く。
もし仮定が満たされていない場合でも、 \(\Sigma_i\) をそれぞれ適当に拡張して、 \(Q_i\) がそれぞれ交わらないように取り替えることで、全てのチューリングマシンに対してこの構成を行うことができる。
と、より一般に、グラフへの拡張を行うことができることが想像できる。
グラフの上のチューリングマシンがあり、各辺に状態がのっているものを考え、そこからチューリングマシンを作りたい。
グラフ \((V,E)\) の辺 \(e \in E\) に対して、その始点を \(\text{sr} e\) 終点を \(\text{tr} e\) と書く。

チューリングマシンの連結
:   - 次のものが与えられているとする。
        - \((V, E)\) :有限な有向グラフ
        - \(v_{\text{init}}\) : \(V\)
        - \(M_v = (\Sigma_v, Q_v, q_{\text{init}}^v, Q_{\text{fin}}^v, \delta_v)\): 各 \(v \in V\) に対してチューリングマシンを与える関数
        - \(S_e\): 各 \(e \in E\) に対してその辺の始点に対応するチューリングマシンの終了状態の一つをあたえる関数
        - \(\Sigma_v = \Sigma_{v^\prime}\) forall \(v, v^\prime \in V\) (\(\Sigma\) と書くことにする。)
        - \(Q_v \cap Q_{v^\prime} = \emptyset\) if \(v \not = v^\prime\)
    - チューリングマシンの連結は次のように定義する。
        - \(Q = (\bigcup_{v \in V} Q_v)\)
        - \(q_{\text{init}} = q_{\text{init}}^{v_{\text{init}}}\)
        - \(Q_{\text{fin}} = {\bigcup_{v \in V} Q_{\text{fin}}^{v}} - \bigcup_{e \in E} Q_e\)
        - \(\delta = (\bigcup_{v \in V} \delta_v) \cup \{(S_e, s, q_{\text{init}}^{\text{tr} e}, s, C) \mid e \in E, s \in \Sigma\}\)
        - \((\Sigma, Q, q_{\text{init}}, Q_{\text{fin}}, \delta)\) はチューリングマシンである。

証明は似たような感じでできると思う。

いくつか観察を行いたい。

辺の始点と終点が一致している場合も許容されていることに注目する。
例えば頂点が１つ、辺が１つのグラフ \((\{v\}, \{e\})\) にチューリングマシン \(M\) と状態 \(q \in Q_{\text{fin}}\) がのっているとする。
このとき、得られるチューリングマシンの遷移関数は \((q, s, q_{\text{init}}, s, C)\) だけ増えているから、もともとのチューリングマシンで \(q\) で停止するような計算があったとき、新しく得られたチューリングマシンはもう一度テープに計算を施すことがわかる。

さらに、施した結果が \(q\) で停止した場合にはもう一度テープに計算を施すから、結果として、 \(q\) で停止しなくなるまで永遠に計算し続けるものが得られる。
ここで対比として \(M \rightarrow_{q} M\) を考える。
（ただし、「もし仮定が満たされていない場合でも」に対応していることに注意する。）
\(\rightarrow\) を使った場合は \(q\) で一度停止した場合は確かにもう一度 \(M\) による計算を行うが、再度 \(q\) が出た場合はそこで停止する。


このグラフの構成についても、仮定を満たしていないようなチューリングマシンがくっついている場合に、適切に記号 \(\Sigma\) を拡張したり状態 \(Q\) を重ならないように動かしたりすることで、"計算"行為を保ったまま構成ができる。

> [!Note]
> 多分、グラフに対して一気にチューリングマシンを作るのと、辺ごとにチューリングマシンをくっつけていったものとでは同値になるのではないか？
> \(M_1 \sim_{\text{eq}} M_2\) のような関係に対してはグラフ上のチューリングマシンを単に取り換えることができないことに注意する。
> \(\sim_{\text{eq}}\) を状態の移りあいを考慮して定義することが必要である。
