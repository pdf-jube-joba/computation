# ステートマシンを中心にしたハードウェア記述言語
生で論理回路を書くのは厳しいので、ここからは論理回路にコンパイルされるような計算モデルを考えて、それで論理回路の記述としたい。
ほしい機能を入れたらとても長くなった。

- 回路を流れる値
  - \(\text{Value}\) :=
    - \(\true\) | \(\false\) ... Boolean
    - | \([v_1, ..., v_n]\) where \(v_i: \text{Value}\) ... Array of Value
    - | \(\{s_1: v_1, ..., s_n: v_n\}\) where \(n \in \Nat, s_i: \String, v_i: \text{Value}\) ... struct of Value
    - | \(\langle s \rangle\) where \(s \in S\) and \(S\) is finite set ... type of finite state
- 値につく型
  - \(\text{Type}\) :=
    - \(\Bool\)
    - | \(t[n ]\) where \(t: \text{Type}, n in \Nat\)
    - | \(\{s_1: t_1, ..., t_n: t_n\}\) where \(n \in \Nat, s_i: \String, t_i: \text{Type}\)
    - | \(\langle S \rangle\) where \(S\) is fintie set
- 型付け規則
    - 見ての通り、ただし、 \(s \in S_1, S_2\) なら \(s : S_1\) かつ \(s: S_2\) であるので、型は一意ではない。
    - 判定はアルゴリズム的にできそう。

型は記述には使うが意味論には使わない。

組み合わせ回路の記述にはループのない手続き型言語みたいなものを用いる。

- 組み合わせ回路の式
  - \(\text{Exp}\) :=
    - \(v\) where \(v: \text{Value}\)
    - | \(x\) where \(x: \text{Variable}\)
    - | \(\text{Exp}\) `||` \(\text{Exp}\) | \(\text{Exp}\) `&&` \(\text{Exp}\) | `!`\(\text{Exp}\) ... bit の表現
    - | `if` \(\text{Exp}\) `then` \(\text{Exp}\) `else` \(\text{Exp}\) ... if 文
    - | \(e[n ]\) where \(e: \text{Exp}, n \in \Nat\)
    - | \([e_1, \ldots, e_n]\) where \(n \in \Nat, e_i: \text{Exp}\)
    - | \(e.s\) where \(e: \text{Exp}, s: \String\)
    - | \(\{s_1: e_1, ..., s_n: e_n\}\) where \(n \in \Nat, s_i \in \String, e_i: \text{Exp}\)
    - | `switch` \(e\) `case` \(s_1: e_1\), ..., `case` \(s_n: e_n\) where \(s_i: \String, e_i: \text{Exp}\)
    - | `seq` \(x_1 := e_1\) `;` ... \(x_n := e_n\) `;` e where \(x_i: \text{Variable}, e_i, e: \text{Exp}\)
    - | `comb` \(s\)(\(e\)) where \(s: \String\), \(e: \text{Exp}\)

組み合わせ回路の内部で別の組み合わせ回路を呼び出すことができるので、その意味論には現在宣言されている組み合わせ回路の一覧が必要になる。
また、変数を扱うためその宣言についての環境も必要である。
それを踏まえると意味は次のように与えることができる。
- 組み合わせ回路の意味
  - \(\text{eval-comb}\): List \((\String, \text{Exp})\) \(\times\) List \((\text{Variable}, \text{Value})\) \(\times\) \(\text{Exp}\) \(\partfunction\) \(\text{Value}\) := \(\text{eval-comb} E G e |->\)
    - \(v\) if
      - \(v\) = \(e\) where \(v: \text{Value}\)
    - \(v\) if
      - \(x\) = \(e\) where \(x: \text{Vabiable}\)
      - \((x, v) in G\)
    - \(v_1 \vee v_2\) if
      - \(e_1\) `||` \(e_2\) = \(e\)
      - \(\text{eval-comb} E G e_i = v_i\)
      - \(v_i \in \Bool\)
    - \(v_1 \wedge v_2\) if
      - \(e_1\) `&&` \(e_2\) = \(e\)
      - \(\text{eval-comb} E G e_i = v_i\)
      - \(v_i \in \Bool\)
    - \(\neg v\) if
      - `!` \(e^\prime\) = \(e\)
      - \(\text{eval-comb} E G e^\prime = v\)
    - \(v\)
      - `if` \(e_1\) `then` \(e_2\) `else` \(e_3\) = \(e\)
      - \(\text{eval-comb} E G e_1 = \true\)
      - \(\text{eval-comb} E G e_2 = v\)
    - \(v\)
      - `if` \(e_1\) `then` \(e_2\) `else` \(e_3\) = \(e\)
      - \(\text{eval-comb} E G e_1 = \false\)
      - \(\text{eval-comb} E G e_3 = v)
    - \(v_i\)
      - \(e^\prime[i ] = e\)
      - \(\text{eval-comb} E G e^\prime = [v_1, \ldots, v_n]\)
      - \(v_i\) if \(i \leq n\)
    - \([v_1, \ldots, v_n]\)
      - \([e_1, \ldots, e_n] = e\)
      - \(\text{eval-comb} E G e_i = v_i\)
    - \(v_i\)
      - \(e^\prime. s\) = \(e\)
      - \(\text{eval-comb} E G e^\prime = \{s_1: v_1, \ldots , s_n: v_n\}\)
      - \(s = s_i\)
    - \(\{s_1: v_1, \ldots, s_n: v_n\}\)
      - \(\{s_1: e_1, \ldots, s_n: e_n\} = e\)
      - \(\text{eval-comb} E G e_i = v_i\)
    - \(v\)
      - `switch` \(e^\prime\) `case` \(s_1\): \(e_1\), ..., `case` \(s_n\): \(e_n\) = \(e\)
      - \(\text{eval-comb} E G e^\prime = s_i\)
      - \(\text{eval-comb} E G e_i = v\)
    - \(v\)
      - `seq` \(x_1 := e_1\) `;` ... \(x_n := e_n\) `;` e = \(e\)
      - \(G_1 = G\)
      - \(\text{eval-comb} E G_i e_i = v_i\), \((X_i, \_) \not \in G_i\), let \(G_{i+1} = [G_i, (x_i, v_i)]\)
      - \(\text{eval-comb} E G_{n+1} e = v\)
    - \(v_1\)
      - `comb` \(s\) (\(e^\prime\)) = \(e\)
      - \(\text{eval-comb} E G e^\prime = v\)
      - \((s, e_1) \in E\)
      - \(\text{eval-comb} E [("IN", v)] e_1 = v_1\)

そしたら、組み合わせ回路の宣言自体は次のようになる。
- \(s\): `comb` (`IN`: \(t_I\)`;` `OUT`: \(t_O\)) \(e\) where \(s: \String, t_I, t_O: \text{Type}, e: \text{Exp}\)

順序回路の記述はステートマシンを用いるが、
ステートマシンの記述は
- 新たに作る
- すでにあるものを合成する
- 繰り返しにより合成する
の \(3\) つを用意する。
ステートマシンはここでは次の組とする。
- 初期状態: \(\text{Value}
- 遷移関数: \(\text{Value} \times \text{Value} \partfunction \text{Value} \times \text{Value})\)

単純ステートマシンの記述
- \(s\): `state` (`IN`: \(t_I\)`;` `STATE`: \((v: t_S)\)`;`, `OUT`: \(t_O\)) \(e\) where \(s: \String, t_I, t_S, t_O: \text{Type}, e: \text{Exp}, v: \text{Value}\)
- 意味論 ... \(\text{toSM}\): List \((\String, \text{Exp})\) \(\times\) \((\text{Value}, \text{Exp})\) \(\partfunction\) ステートマシン := \(\text{toSM} E (v, e) |->\)
  - 初期状態 := \(v\)
  - 遷移状態 := \((s, i) |-> (v_s, v_O)\)
    - \(\text{eval-comb} E [("IN", i), ("STATE", s)] e = {"OUT": v_O, "STATE": v_s}\)

# コンパイルについて
