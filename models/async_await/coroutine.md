## 構文

\[
\begin{aligned}
\NT{program} &\defeq \NT{func-decl}+ \\
\NT{func-decl} &\defeq \T{fn} \NT{f-name} \T{\LCB} \NT{stmt}+ \T{\RCB} \\
\NT{stmt} &\defeq \\
  &| \NT{var} \T{<-} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{goto} \NT{offset} \\
  &| \T{goto} \NT{offset} \\
  &| \T{call} \NT{f-name} \\
  &| \T{run} \NT{f-name} \T{->} \NT{id} \\
  &| \T{yield} \\
\NT{aexp} &\defeq \NT{number} | \NT{var} | \NT{aexp} \NT{op} \NT{aexp} \\
\NT{bexp} &\defeq \NT{uop} \NT{bexp} | \NT{aexp} \NT{rop} \NT{aexp} | \NT{bexp} \NT{cop} \NT{bexp} \\
  &| \T{done} \NT{id} \\

\NT{var} &\defeq \NT{string} \\
\NT{id} &\defeq \T{\$} \NT{string} \\
\NT{bop} &\defeq \T{+} | \T{-} | \T{*} \\
\NT{rop} &\defeq \T{==} | \T{!=} | \T{<} \\
\NT{uop} &\defeq \T{!} \\
\NT{cop} &\defeq \T{\&\&} | \T{||} \\
\NT{offset} &\defeq (\T{+} | \T{-}) \NT{number} \\
\end{aligned}
\]

## 意味論
変数は全てグローバルにしておく。goto にしたのは、 pc の数え方が楽だから。
関数呼び出しはないとちょっとわかりにくかった。 async await がないので、ただの coroutine っぽいものになっている？

- \(E\) := \(\NT{var} \cup \NT{id} \pfun \N\) ... \(\NT{var}\) のグローバル変数環境
  - \(\NT{var}\) の初期値は全部 \(0\) とする
  - \(\NT{id}\) の初期値は `None` とする
- \(F\) := \([\text{func}:\NT{f-name}, \text{pc}: \N]\) ... 各実行タスクのスタック
- \(S\) := \([F]\)
- \(A\): 集合 ... エージェントの集合なので、なんでもよい。大きさは実行中ずっと固定。スレッドプールのようなもの。
- \(M\) := \(A \pfun \N\) ... 各エージェントがどのタスクを処理しているか
- \(Q\) := \([\N]\) ... 実行待ちのキュー
- \(\text{eval-aexp}(a: \NT{aexp}, e: E): \N\) := ... これはよくあるやつなのでいい。
- \(\text{eval-bexp}(b: \NT{bexp}, e: E, s: S): \mathbb{B}\) :=
  - done 以外は普通にやる
  - done については、  \(E\) 経由で \(i: \N\) を得て、 \(S[i]\) が empty かどうかの判定とする。

これらのもとで、 \((M, Q, S, E)\) を1ステップで動かす。
ただし、どのエージェントがどう動くかはわからないので非決定的（ small step だが、遷移先が複数ある）になる。
この添え字の部分が、どのエージェントが動くかを表している。


### エージェントの更新
\[
\begin{aligned}
(M, Q, S, E) &\to^{i \in A} (M', Q', S, E) \\
  &\text{if \(M(i) = \bot\) \(\lvert Q \rvert \neq 0\)}\\
  &\text{where \((q, Q') := \text{pop}(Q)\)} \\
  &\text{where \(M' := M[i \mapsto q]\)} \\
(M, Q, S, E) &\to^{i \in A} (M', Q, S, E) \\
  &\text{if \(M(i) = j\), \(S[j] = \text{nil}\)} \\
  &\text{where \(M' := M[i \mapsto \bot]\)} \\
(M, Q, S, E) &\to^{i \in A} (M, Q, S', E) \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(p = \text{length}(f)\)} \\
  &\text{where \(S' := S[j \mapsto F]\)} \\
\end{aligned}
\]

### 普通の環境更新
\[
\begin{aligned}
(M, Q, S, E) &\to^{i \in A} (M, Q, S', E') \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(f[p] = x \T{<-} a\)} \\
  &\text{where \(E' := E[x \mapsto l], l = \text{eval-aexp}(a, E)\)} \\
  &\text{where \(S' := S[j \mapsto (f, p + 1)::F]\)} \\
(M, Q, S, E) &\to^{i \in A} (M, Q, S', E) \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(f[p] = \T{if} b \T{goto} r\)} \\
  &\text{where \(r' = \text{if \(\text{eval-bexp}(b, E, S)\) then \(r\) else \(1\)}\)} \\
  &\text{where \(S' := S[j \mapsto (f, p + r')::F]\)} \\
(M, Q, S, E) &\to^{i \in A} (M, Q, S', E) \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(f[p] = \T{goto} r\)} \\
  &\text{where \(S' := S[j \mapsto (f, p + r)::F]\)} \\
\end{aligned}
\]

### coroutine 部分
\[
\begin{aligned}
(M, Q, S, E) &\to^{i \in A} (M, Q, S', E) \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(f[p] = \T{call} f'\)} \\
  &\text{where \(S' := S[j \mapsto (f', 0)::(f, p + 1)::F]\)} \\
(M, Q, S, E) &\to^{i \in A} (M, Q', S', E') \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(f[p] = \T{run} f' \T{->} v\)} \\
  &\text{where l := \text{length}(S)} \\
  &\text{where \(E' := E[v \mapsto l]\)}\\
  &\text{where \(S_1 := S[j \mapsto (f, p + 1)::F]\)} \\
  &\text{where \(S' := S_1@[(f', 0)]\)} \\
  &\text{where \(Q' := Q@l\)} \\
(M, Q, S, E) &\to^{i \in A} (M', Q', S', E) \\
  &\text{if \(M(i) = j\), \(S[j] = (f, p)::F\)} \\
  &\text{if \(f[p] = \T{yield}\)} \\
  &\text{where \(S' := S[j \mapsto (f, p + 1)::F]\)} \\
  &\text{where \(M' := M[i \mapsto \bot]\)} \\
  &\text{where \(Q' := Q@j\)} \\
\end{aligned}
\]