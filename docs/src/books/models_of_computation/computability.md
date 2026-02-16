# 計算モデルとエミュレーション
ちゃんとした文章だとチューリングマシンを導入した後にすぐやるような内容だけど、
面白さが伝わりにくいのでここに書いた。

- チューリングマシン ・ goto lang ・ ラムダ計算はいずれも再帰関数を計算することができる
- また、再帰関数自体もそれぞれの計算モデルを再現することができる。

- [名古屋大学の計算可能性についてのノート](https://www.math.mi.i.nagoya-u.ac.jp/~kihara/pdf/teach/computability2017fall-final.pdf)
- [とりマセ](https://recursion-theory.blogspot.com/2018/11/q.html)

いずれの場合にせよ

## 再帰関数を goto 言語で再現する

- [] ここに recursive_function-goto_lang をはる

## 再帰関数をチューリングマシンで再現する

- [] ここに recursive_function-turing_machine をはる

## 再帰関数をラムダ計算で再現する

ラムダ計算もまた帰納関数の計算を"埋め込む"ことができる。
ここでは、以前と同じ形で自然数をラムダ項に直す。
目標は、各再帰関数 \(f\) に対して、ラムダ項 \(F\) であって、
「全部の自然数 \(n\) で、\(F(\text{自然数} \(n\) \text{に対応するラムダ項})\) に対してたくさん step を押すと \(f(n)\) に一致している」ようなものを探すことである。

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
