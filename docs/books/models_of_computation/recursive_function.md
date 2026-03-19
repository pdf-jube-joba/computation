# 再帰的関数
自然数全体を \(\mathbb{N}\) と書き、 \(0\) を含めるものとする。
自然数の組から自然数への関数のうち、次のような"計算できる"っぽいものを再帰関数という。
（ちゃんとした定義は `models/` の方へ行ってください。）
- 定数関数、 \(n \mapsto n + 1\) 、射影関数（自然数の組から一つだけ取り出す。）
- 原始再帰関数： 「\(n + 1\) とあらわされる場合は _なんとかかんとか_ 、 \(0\) の場合は _また別の定義_ 」という形式で書かれた関数
- \(\mu\) 再帰関数：\(n, m\) に対して定義されている関数 \(f\) があったときに、「 \(m\) に対して \(f(n, m) = 0\) となる最小の \(n\) を結び付ける 」ことで定義される関数

## 具体例
ここでは実際の計算の過程を、足し算を例にしている。
足し算の定義には、原始再帰を使う。

<div data-model="recursive_function">
<script type="text/plain" class="default-code">
let zf = PROJ[1,0].
let sf = COMP[SUCC: PROJ[3,0]].
let add = PRIM[z: zf s: sf].
add
</script>
<script type="text/plain" class="default-ainput">
(3, 4)
</script>
</div>
