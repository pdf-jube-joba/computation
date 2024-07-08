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
ここで、 CombEnv は List \((\String, \text{Exp})\) のこととする。

- 組み合わせ回路の意味
  - \(\text{eval-comb}\): CombEnv \(\times\) List \((\text{Variable}, \text{Value})\) \(\times\) \(\text{Exp}\) \(\partfunction\) \(\text{Value}\) := \(\text{eval-comb} E G e |->\)
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

順序回路の記述は Moore 型のステートマシンを用いるが、
ステートマシンの記述は
- 新たに作る
- すでにあるものを合成する
- 繰り返しにより合成する
の \(3\) つを用意する。
ステートマシンは入力と出力は \(\text{Value}\) とし、状態は構成に応じて定義する。
つまりステートマシンは
- 状態の全体 (\(S\)): Set
- 初期状態: \(S\)
- 遷移関数: \(S times \text{Value} \partfunction S\)
- 出力計算: \(S\ partfunction \text{Value}\)

ステートマシンとその名称の組のリスト \((\String, \text{SM})\) を SMEnv と書くことにする。

単純ステートマシン
- 記述
  - \(s\): `state` (`IN`: \(t_I\)`;` `STATE`: \((v_0: t_S)\)`;`, `OUT`: \(t_O\)) `transition=`\(e_1\)`;` `output`=\(e_2\)`;` where \(s: \String, t_I, t_S, t_O: \text{Type}, e_i: \text{Exp}, v_0: \text{Value}\)
  - つまり単純ステートマシン SimpSM は \((\String, \text{Value}, \text{Exp}, \text{Exp})\) のこと
- 意味論 ... \(\text{toSM}\): CombEnv \(\times\) SMEnv \(\times\) SimpSM \(\partfunction\) ステートマシン := \(\text{toSM} E S (s, v, e_1, e_2) |->\)
  - 状態の全体は \(\text{Value}\) 
  - 初期状態 := \(v_0\)
  - 遷移関数 := \((v, i) |-> v^\prime\) where
    - \(\text{eval-comb} E [("IN", i), ("STATE", v)] e_1 = v^\prime\)
  - 出力計算 := \(v |-> o\) where
    - \(\text{eval-comb} E [("STATE", v)] e_2 = o\)
  - \(s\) はこの後の他のマシン宣言時に使う、 \(S\) は参照しない。

とても再帰的な定義になっているちゃんと書くのがめんどくさい。

ステートマシンのグラフによる構成
- 記述
  - \(s\): `graph` (`IN`: \(t_I\)`;` `MACHINE`: \((s_1: N_1, ..., s_n: N_n)\)`;`, `OUT`: \(t_O\)) `transition=`\(e_1\)`;` `output`=\(e_2\)`;` where \(s: \String, t_I, t_S, t_O: \text{Type}, e_i: \text{Exp}, v_0: \text{Value}\)
  - つまりグラフ構成 GraphSM は \((\String, \text{Set of} \, (\String \times \String), \text{Exp}, \text{Exp})\) のこと
- 意味論 ... \(\text{toSM}\): CombEnv \(\times\) SMEnv \(\times\) GraphSM \(\partfuncion\) ステートマシン := \(\text{toSM} E S (s, (s_i: S_i), e_1, e_2) |->\)
  - \((S_i, v_i, \delta_i, r_i)\) := \((N_i, (S_i, v_i, \delta_i, r_i)) \in S\)
  - 状態の全体 := \((S_1, \ldots, S_n)\)
  - 初期状態 := \((v_1, \ldots, v_n)\)
  - 遷移関数 := \(((v_i)_i, i) |-> (v^\prime_i)\)

# コンパイルについて
