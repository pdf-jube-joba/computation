2026/03/21 の現状

- Model の定義
  - \((A, C, F, S, I, O)\) という集合5つ
  - \(m\): \((A, C) \to S\)
  - \(s\): \((S, I) \to (S, O) + F\)
- encode/decode
  - \(e_A\): \(A^1 \to A^2\), \(e_I\): \(I^1 \to I^2\)
  - \(d_O\): \(O^2 \to O^1\), \(d_F\): \(F^2 \to F^1\)
- Compiler の定義
  - \(M^i = (A^i, C^i, F^i, S^i, I^i, O^i, m^i, s^i)\)
  - \(c\): \(C^1 \to C^2\)
- 表現したいもの
  - I diagram: \(M^h\) (host)で書かれた \(M^g\) (guest)言語の interpreter ?
  - T diagram: \(M^h\) (host)で書かれた \(M^s\) (source)言語を \(M^t\) (target)言語に変換する compiler ?
