# 計算モデルとエミュレーション
チューリングマシン ・ goto lang ・ ラムダ計算はいずれも再帰関数を計算することができる
また、再帰関数自体もそれぞれの計算モデルを再現することができる。
いずれの場合にせよ入出力の対象は異なるので、
どういう入力が来たらどう解釈するかを決めておく必要がある。

再帰関数が再現できるというときは、
自然数 \(n\) を表していると（無理なく）解釈できるような入出力の翻訳方法を決めたとき、
各再帰関数と同じようなふるまいをしているものが対象となる計算モデルの中に存在していれば、
再帰関数を計算できるといえる。

## 再帰関数を goto 言語で再現する

- [] ここに recursive_function-goto_lang をはる

## 再帰関数をチューリングマシンで再現する

- [] ここに recursive_function-turing_machine をはる

## 再帰関数をラムダ計算で再現する

ラムダ計算もまた帰納関数の計算を"埋め込む"ことができる。
ここでは、以前と同じ形で自然数をラムダ項に直す。
目標は、各再帰関数 \(f\) に対して、ラムダ項 \(F\) であって、
「全部の自然数 \(n\) で、\(F(\text{自然数} n \text{に対応するラムダ項})\) に対してたくさん step を押すと \(f(n)\) に一致している」ようなものを探すことである。

以下で変換器が書いてある。
<div data-model="recursive_function-lambda_calculus">
<script type="text/plain" class="default-code">
let zf = PROJ[1,0].
let sf = COMP[SUCC: PROJ[3,0]].
let add = PRIM[z: zf s: sf].
add
</script>
<script type="text/plain" class="default-ainput">
(2, 3)
</script>
</div>
