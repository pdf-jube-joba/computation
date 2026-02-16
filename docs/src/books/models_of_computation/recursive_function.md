# 再帰的関数
自然数全体を \(\mathbb{N}\) と書き、 \(0\) を含めるものとする。
この節では再帰関数と呼ばれる関数たちを定義する。
自然数の組から自然数への関数のうち、"計算できる"っぽいものを再帰関数という。
ちゃんとした定義はあります。

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
