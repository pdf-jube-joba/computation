Krivine による call by name なラムダ計算をやるための機械
ラムダ計算は De Bruijn index で考える。
そのため、変数は \(n \in \N\) でスタックに積まれているものにアクセスする。
wiki がすごいわかりやすい。

- \(M\) := ラムダ項
- \(S\) := \(\text{list}(M, E)\)
- \(E\) := \(\text{list}(M, E)\)

\[
\begin{aligned}
  (M _ 1 M _ 2, p, e) &\to (M _ 1, (M _ 2, e) :: p, e) \\
  (\lambda M, (t, e) :: p , e') &\to (M, p, (t, e) :: e') \\
  (0, p, (t, e) :: e') &\to (t, p, e') \\
  (n + 1, p, _ :: e) &\to (n, p, e)
\end{aligned}
\]
