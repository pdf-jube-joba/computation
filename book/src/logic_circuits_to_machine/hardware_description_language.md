# ステートマシンを中心にしたハードウェア記述言語
生で論理回路を書くのは厳しいので、ここからは論理回路にコンパイルされるような計算モデルを考えて、それで論理回路の記述としたい。
ほしい機能を入れたらとても長くなった。

- 回路を流れる値
  - Value :=
    - \(\true\) | \(\false\) ... Boolean
    - | \([v_1, ..., v_n]\) where \(v_i\): Value ... Array of Value
    - | \(\{s_1: v_1, ..., s_n: v_n\}\) where \(n \in \Nat, s_i: \String, v_i\): Value ... struct of Value
    - | \(\langle s \rangle\) where \(s \in S\) and \(S\) is finite set of \(\String\) ... type of finite set of \(\String\)
- 値につく型
  - Type :=
    - \(\Bool\)
    - | \(t[n ]\) where \(t\): Type, \(n \in \Nat\)
    - | \(\{s_1: t_1, ..., t_n: t_n\}\) where \(n \in \Nat\), \(s_i\): \(\String\), \(t_i\): Type
    - | \(\langle S \rangle\) where \(S\) is finite set
- 型付け規則
    - 見ての通り、
    - ただし、 \(s \in S_1 \cap S_2\) なら \(s: S_1\) かつ \(s: S_2\) であるので、型は一意ではない。
    - 判定はアルゴリズム的にできそう。

型は記述には使うが意味論には使わない。

組み合わせ回路の記述にはループのない手続き型言語みたいなものを用いる。

- 組み合わせ回路の式
  - Exp :=
    - \(v\) where \(v\): Value
    - | \(x\) where \(x\): Variable
    - | Exp `||` Exp | Exp `&&` Exp | `!`Exp ... bit の表現
    - | `if` Exp `then` Exp `else` Exp ... if 文
    - | \(e[n ]\) where exp: Exp, \(n \in \Nat\)
    - | \([e_1, \ldots, e_n]\) where \(n \in \Nat\), \(e_i\): Exp
    - | \(e.s\) where exp: Exp, \(s: \String\)
    - | \(\{s_1: e_1, ..., s_n: e_n\}\) where \(n \in \Nat\), \(s_i \in \String\), \(e_i\): Exp
    - | `switch` exp `case` \(s_1: e_1\), ..., `case` \(s_n: e_n\) where \(s_i: \String, e_i\): Exp
    - | `seq` \(x_1 := e_1\) `;` ... \(x_n := e_n\) `;` e where \(x_i: \text{Variable}, e_i, e\): Exp
    - | `comb` \(s\)(exp) where \(s: \String\), exp: Exp

組み合わせ回路の内部で別の組み合わせ回路を呼び出すことができるので、その意味論には現在宣言されている組み合わせ回路の一覧が必要になる。
また、変数を扱うためその宣言についての環境も必要である。
それを踏まえると意味は次のように与えることができる。
ここで、 CombEnv は List (\(\String\), Exp) のこととする。
また、 VarEnv は List (Variable, Value) のこととする。

- 組み合わせ回路の意味
  - eval-comb: CombEnv \(\times\) VarEnv \(\times\) Exp \(\partfunction\) Value :=  eval-comb E G exp \(|->\)
    - \(v\) if
      - \(v\) = exp where \(v\): Value
    - \(v\) if
      - \(x\) = exp where \(x\): Variable
      - \((x, v) in G\)
    - \(v_1 \vee v_2\) if
      - e_1 `||` e_2 = exp
      - eval-comb E G e_i = \(v_i\)
      - \(v_i \in \Bool\)
    - \(v_1 \wedge v_2\) if
      - e_1 `&&` e_2 = exp
      - eval-comb E G \(e_i = v_i\)
      - \(v_i \in \Bool\)
    - \(\neg v\) if
      - `!` e = exp
      - eval-comb E G \(e^\prime = v\)
    - \(v\)
      - `if` e_1 `then` e_2 `else` \(e_3\) = exp
      - eval-comb E G \(e_1 = \true\)
      - eval-comb E G \(e_2 = v\)
    - \(v\)
      - `if` e_1 `then` e_2 `else` \(e_3\) = exp
      - eval-comb E G \(e_1 = \false\)
      - eval-comb E G \(e_3 = v)
    - \(v_i\)
      - \(e^\prime[i ] = e\)
      - eval-comb E G \(e^\prime = [v_1, \ldots, v_n]\)
      - \(v_i\) if \(i \leq n\)
    - \([v_1, \ldots, v_n]\)
      - \([e_1, \ldots, e_n] = e\)
      - eval-comb E G \(e_i = v_i\)
    - \(v_i\)
      - \(e^\prime. s\) = exp
      - eval-comb E G \(e^\prime = \{s_1: v_1, \ldots , s_n: v_n\}\)
      - \(s = s_i\)
    - \(\{s_1: v_1, \ldots, s_n: v_n\}\)
      - \(\{s_1: e_1, \ldots, s_n: e_n\} = e\)
      - eval-comb E G \(e_i = v_i\)
    - \(v\)
      - `switch` \(e^\prime\) `case` \(s_1\): e_1, ..., `case` \(s_n\): \(e_n\) = exp
      - eval-comb E G \(e^\prime = s_i\)
      - eval-comb E G \(e_i = v\)
    - \(v\)
      - `seq` \(x_1 := e_1\) `;` ... \(x_n := e_n\) `;` e = exp
      - \(G_1 = G\)
      - \(\text{eval-comb} E G_i e_i = v_i\), \((X_i, \_) \not \in G_i\), let \(G_{i+1} = [G_i, (x_i, v_i)]\)
      - \(\text{eval-comb} E G_{n+1} e = v\)
    - \(v_1\)
      - `comb` \(s\) (\(e^\prime\)) = exp
      - eval-comb E G \(e^\prime = v\)
      - \((s, e_1) \in E\)
      - \(\text{eval-comb} E [("IN", v)] e_1 = v_1\)

そしたら、組み合わせ回路の宣言自体は次のようになる。
- \(s\): `comb` (`IN`: \(t_I\)`;` `OUT`: \(t_O\)) exp where \(s: \String, t_I, t_O: \text{Type}, e\): Exp

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
  - \(s\): `state` (`IN`: \(t_I\)`;` `STATE`: \((v_0: t_S)\)`;`, `OUT`: \(t_O\)) `transition=`e_1`;` `output`=e_2`;` where \(s: \String, t_I, t_S, t_O: \text{Type}, e_i\): Exp, \( v_0: \text{Value}\)
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
  - \(s\): `graph` (`IN`: \(t_I\)`;` `MACHINE`: \((s_1: N_1, ..., s_n: N_n)\)`;`, `OUT`: \(t_O\)) `transition=`e_1`;` `output`=e_2`;` where \(s: \String, t_I, t_S, t_O: \text{Type}, e_i\): Exp, \( v_0: \text{Value}\)
  - つまりグラフ構成 GraphSM は \((\String, \text{Set of} \, (\String \times \String), \text{Exp}, \text{Exp})\) のこと
- 意味論 ... \(\text{toSM}\): CombEnv \(\times\) SMEnv \(\times\) GraphSM \(\partfuncion\) ステートマシン := \(\text{toSM} E S (s, (s_i: S_i), e_1, e_2) |->\)
  - \((S_i, v_i, \delta_i, r_i)\) := \((N_i, (S_i, v_i, \delta_i, r_i)) \in S\)
  - 状態の全体 := \((S_1, \ldots, S_n)\)
  - 初期状態 := \((v_1, \ldots, v_n)\)
  - 遷移関数 := \(((v_i)_i, i) |-> (v^\prime_i)_i\) where
    - \(o_i\) := \(r_i v_i\)
    - \(v\) := eval-comb E [("IN", \(i\)), (\(s_i\), \(o_i\)) ] e_1
    - \({s_i: i_i} = v\)
    - \(v^\prime_i = \delta_i (s_i, i_i)\)
  - 出力 := \((v_i)_i |-> o\) where
    - \(o_i\) := \(r_i v_i\)
    - \(o\) := eval-comb E [, (\(s_i\), \(o_i\)) ] e_2

ステートマシンの繰り返し
- 記述
  - \(s\): `iter`: (`IN`: \(t_I\); `MACHINE`: \(N\), `OUT`: \(t_O\)) `input`=\(e_\text{in}\); `transition` = \(e\); `output`=\(e_{\text{ot}}\)
- 意味論
  - 同様の型を持つ toSM を定義する。
  - \((S, v, \delta, r)\) := \((N, (S, v, \delta, r)) \in S\)
  - 状態の全体 := \((S)_{i \in \Nat}\)
  - 初期状態 := \((v)_{i \in \Nat}\)
  - 遷移関数 := \(((v_i)_{i \in \Nat}, i) |-> (v^\prime_i)_i\) where
    - \(o_{-1}\) := eval-comb E [("IN", \(i\))] \(e_{\text{in}}\)
    - \(o_i\) := \(r v_i\)
    - \(i_i\) := eval-comb \(E\) [("PREV", \(o_{i-1}\)), ("THIS", \(o_i\)), ("NEXT", \(o_{i+1}\))] \(e\)
    - \(v^\prime_i\) := \(\delta (v_i, i_i)\)
  - 出力 := \((v_i)_{i \in \Nat} |-> o\)
    - \(o_0\) := \(r v_0\)
    - \(o\) := eval-comb \(E\) [("OUT", \(o_0\))] \(e_2\)

# コンパイルについて
compile-comb と compile-state の 2 つを作る。
compile-comb は組み合わせ回路へのコンパイルを行い、 compile-state は同期回路へのコンパイルを行う。
実際に動かす際には遅延を考えなくてはいけないので、遅延を計算する time-comb と time-state の 2 つも同時に定義する必要がある。
合成時にはそこまで気にしなくてよい。
ただし、型は気にする。
特に、型付きのもの以外はコンパイルできなくてかまわない。

すでにコンパイル結果の計算された組み合わせ回路のリストとして、
CombEnvCompile := List (\(\String\), LogicCircuits) を与える必要がある。

compile-comb: CombEnvCompile Exp \(\function\) LogicCircuits := compile-comb \(E\) \(e\) = 
  - まあ想像できる通りみたいな感じでいけそう
