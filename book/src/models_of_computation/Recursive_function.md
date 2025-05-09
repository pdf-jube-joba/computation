# 再帰的関数
自然数全体を \(\mathbb{N}\) と書き、 \(0\) を含めるものとする。
この節では再帰関数と呼ばれる関数たちを定義する。
自然数の組から自然数への関数のうち、"計算できる"っぽいものを再帰関数という。


## 再帰関数の定義

あまりうまい説明ができないので、定義をそのまま載せる。

> **definition**
> 次のような帰納的定義で定義される、自然数の組を受け取り自然数を返す"全域"関数のクラスを再帰関数と呼ぶ。
> - 次の関数は自然数関数である
>   - ゼロ定数関数 \(() \in \mathbb{N}^0 \mapsto 0 \in \mathbb{N}\)
>   - 後者関数 \(n \in \mathbb{N}^1 \mapsto n+1 \in \mathbb{N}\)
>   - \(k\) 引数の \(i\) 番目への射影関数 \((n_1, \ldots, n_k) \in \mathbb{N}^k \mapsto n_i \in \mathbb{N}\)
> - 次のようにして得られる合成関数は再帰関数である
>   - \(f\): \(n\) 引数の再帰関数, \(g_1, \ldots g_n\): 全て \(m\) 引数の再帰関数をとる
>   - 合成関数 \((x_1, \ldots, x_m) \in \mathbb{N}^m \mapsto f(g_0(x_1, \ldots, x_m), \ldots g_n(x_1, \ldots, x_m)) \in \mathbb{N}\)
> - 次のようにして得られる原始再帰関数は再帰関数である
>   - \(f\): \(n\) 引数の再帰関数, \(g\): \(n+2\) 引数の再帰関数をとる
>   - 原始再帰関数 \(h (x_0, x_1, \ldots, x_n) =\)
>       - \(x_0 = 0\) のとき \(f(x_1, \ldots, x_n)\)
>       - \(x_0 = 1 + x\) のとき \(g(h(x, x_1, \ldots x_n), x_1, \ldots, x_n)\)
> - 次のようにして得られる \(\mu\) 再帰関数のうち、"全域関数となる"関数は再帰関数である
>   - \(f\): \(n+1\) 引数の再帰関数をとる
>   - \((\mu f)(x_1, \ldots, x_n) = \text{min} \{y \in \mathbb{N} \mid f(y, x_1, \ldots, x_n) = 0\}\)

中身は思ったよりも"計算"っぽい。
例としては恒等写像、足し算、掛け算などの基本的な関数は当然含まれる。

## 具体例
ここでは実際の計算の過程を見せている。
足し算の定義には、原始再帰を使う。

<script type="module">
    import { load, RecursiveFunctionViewModel } from "../assets/generated/recursive_function/recursive_function_glue.js";
    import { TextAreaSource, TextDefinedSource, UserControls } from "../assets/utils.js";
    await load();

    let code_input1 = new TextDefinedSource("let zf = PROJ[1,0]. let sf = COMP[SUCC: (PROJ[3,0])]. let add = PRIM[z: zf s: sf]. add");
    let tuple_input1 = new TextAreaSource("user_defined1");
    let control1 = new UserControls("control1");

    let view1 = new RecursiveFunctionViewModel(code_input1, tuple_input1, control1, "view1");
</script>

<div id="machine1">
    <div id="control1"></div>
    <textarea id="user_defined1" rows="1" cols="20"> (3, 4) </textarea>
    <div id="view1">
    </div>
</div>
