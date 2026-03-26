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
- 注意点：
  - "grounding" がどこか？...`rust` 言語で実装されていることに帰着する。
  - \(\to\) が全て原始再帰の範囲内でしか書けない（ `rust` でいうと、必ず停止することが機械的にわかる範囲）ことが、計算の定義として上を採用することの妥当性？

- 表現したいもの
  - I diagram: \(M^h\) (host)で書かれた \(M^g\) (guest)言語の interpreter ?
    - host 言語のコード \(i_h: C_h\) がある
    - host-guest の AFIO の encode/decode がある
    - 任意の guest 言語のコードの host 言語の ahead-of-time input への encode がある
    - host 言語側の ahead-of-time input で paring ができる
  - T diagram: \(M^h\) (host)で書かれた \(M^s\) (source)言語を \(M^t\) (target)言語に変換する compiler ?
    - host 言語のコード \(c_h: C_h\) がある
    - ainput と foutput が解釈できる
