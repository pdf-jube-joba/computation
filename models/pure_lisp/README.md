Lisp の面白いところは quote と eval による meta circular とか homoiconic みたいに言われるところだと思うので、
それを中心に pure lisp と呼ばれるところだけ持ってくる。
あと、 dynamic scope なところも面白いので、そのように定義したい。

nil は `()` で表すことが多いらしい。

\[\begin{aligned}
\NT{nil} &\defeq \T{\LP \RP} \\
\NT{func} &\defeq \T{atom} | \T{eq} | \T{car} | \T{cdr} | \T{cons} \\
\NT{spec} &\defeq \T{quote} | \T{eval} | \T{if} | \T{lambda} \\
\NT{atom} &\defeq \NT{string} \\
\NT{term} &\defeq \NT{nil} \\
  &\defeq \NT{atom} \\
  &\defeq \T{\LP} \NT{term} \NT{term} \T{\RP} \\
\end{aligned}\]

これに対して、単に interpreter を定義するのはいいが、
ラムダ計算に対する cek みたいな感じで進むことになる。
one step で進めるのはかなりめんどくさい。

環境としては初期に変数としての \(\NT{atom}\) に対して、
そのうち \(\NT{func}\) については最初からある種の関数が束縛されている。
また、これらの関数は未定義な領域があるので、それにあたるとエラーをはくのが典型的な動作だが、
ここでは普通に部分関数として動作していくことにする。

自由な変数に対して、普通は closure として環境を含めて評価できるようにしておくが、
これをせずに、呼び出し側の環境の下で変数を考えるのが dynamic なスコープになる。

アルファ変換みたいなものは必要らしい。
ここでは特にアルファ変換を定義しないが、代入の時にはそれを使っていい感じにする。

- \(V\) := \(\NT{term}\) ... これがすごくて、簡約されていないプログラムがここに入りうる。
  ただし、以降では気持ちの整理のために、分けて書いたりする。
- \(E\) := \(\NT{atom} \to V\) ... これについても、クロージャをつくらないので lambda もこのまま入れられる。
- 以下は組み込みとして定義されている関数
  - \(\text{atom}\): \(V \to V\) := これは atom かどうかを判定して、 "T" か nil を返す。 
  - \(\text{eq}\): \(V \times V \pfun V\) := これは **両辺が atom の場合に** 同じかどうか判定して、上と同じように返す。
  - \(\text{car}, \text{cdr}, \text{cons}\) なども適切に定義する
- \(K\) := \(E\) and enum of
  - \([] \NT{term}\)
  - \(\text{eval}\)
  - \(\text{if}(V, V)\)
  - \(\text{lambda}(x \in \NT{atom}, M)\)
  - \(\text{cont}(\NT{func})

この状態で、次のように動く
\[\begin{aligned}
(\NT{nil}, E, K) &\to \text{return}(\NT{nil}, K) \\
(a \in \NT{atom}, E, K) &\to \text{return}(E[a], K) \\
\begin{aligned}\]
以降は、リストの先頭を見るという構文的な形で対応する。
リストの先頭が特定の形式をしていなかった場合はそこを評価する。
評価し終わって戻ってきたときにちゃんとした形じゃなかったら終わり。
\[\begin{aligned}
((\T{quote} M), E, K) &\to \text{return}(M, K) \\
((\T{eval} M), E, K) &\to (M, E, (\text{eval}, E):: K) \\
((\T{if} ( M (M _ 1 M _ 2) )) , E, K) &\to (M, (\text{if}(M _ 1, M _ 2), E)::K) \\
(((\T{lambda} (x \in \NT{atom}) M) B), E, K) &\to (B, E, (\text{lambda}(x, M), E)::K) \\
((\T{lambda} M), E, K) &\to \text{return}((\T{lambda} M), K) \\
\\
((A B), E, K) &\to (B, E, (\text{cont}(A), E)::K) &\text{if \(A\) is in \NT{func}} \\
((A B), E, K) &\to (A, E, (([] B), E)::K) &\text{if \(A\) is not in \NT{func}} \\
\end{aligned}\]
return してきた値を次のようにして継続で処理する。
\[\begin{aligned}
\text{return}(v, ([] T, E)::K) &\to ((v T), E, K) \\
\text{return}(v, (\text{eval}, E)::K) &\to (v, E, K) \\
\text{return}(v, (\text{if}(M _ 1, M _ 2), E)::K) &\to (M _ 1, E, K) &\text{if \(v\) is \T{T}} \\
\text{return}(v, (\text{if}(M _ 1, M _ 2), E)::K) &\to (M _ 2, E, K) &\text{if \(v\) is not \T{T}} \\
\text{return}(v, (\text{lambda}(x, M), E)::K) &\to (M, E[x \mapsto v], K) \\
\\
\text{return}(v, (\text{cont}(eq), E)::K) &\to (v, E, K) \\
\text{return}(v, (\text{cont}(atom), E)::K) &\to (v, E, K) \\
\end{aligned}\]
