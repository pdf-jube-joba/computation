# while-minus 言語（ざっくり）
プログラミング言語として while-minus 言語というものを見てみる。
（これは適当に While 言語というものから要素をひいて作った。）
この言語は、変数を 
- 初期化（０を入れる）する
- 1 だけ加算する
- 1 だけ減算する
- 値をコピーする

ような単純な文に加え、
`while` と呼ばれる、 **指定された変数が `0` でない場合は** 中に含まれる文達を実行し続ける制御構造からなる。

## 具体例

1つめの例は \(x * y\) を計算するプログラムで、
2つめの例は \(\text{gcd}(x, y)\) を計算するプログラム

<script type="module">
    import { load, WhileMinusLangViewModel } from "../assets/generated/while_minus_lang/while_minus_lang_glue.js";
    import { TextAreaSource, TextDefinedSource, UserControls } from "../assets/utils.js";
    await load();

    let res1 = await fetch("../assets/component/models_of_computation/while_minus_lang_multiplication.txt");
    let txt1 = await res1.text();

    let code_input1 = new TextDefinedSource(txt1);
    let tape_input1 = new TextDefinedSource("x=5, y=3");
    let control1 = new UserControls("control1");

    let view1 = new WhileMinusLangViewModel(code_input1, tape_input1, control1, "view1");

    let res2 = await fetch("../assets/component/models_of_computation/while_minus_lang_gcd.txt");
    let txt2 = await res2.text();

    let code_input2 = new TextDefinedSource(txt2);
    let tape_input2 = new TextAreaSource("user_defined");
    let control2 = new UserControls("control2");

    let view2 = new WhileMinusLangViewModel(code_input2, tape_input2, control2, "view2");
</script>

<div id="machine1">
    <div id="control1"></div>
    <div id="view1">
    </div>
</div>

<div id="machine2">
    <div id="control2"></div>
    <textarea id="user_defined" rows="1" cols="20"> x = 6, y = 4 </textarea>
    <div id="view2">
    </div>
</div>

# while-minus 言語（ちゃんとした解説）

## 定義
変数の集合 \(\Lambda\) を固定しておく。
while-minus 言語は次の形で与えられる。

> **Definition**
> while-minus 言語の文は以下のものとする。
> - 変数 \(s\) に対して、 increment \(s\), decrement \(s\), reset \(s\)
> - \(2\) つの変数 \(s_1, s_2\) に対して、 copy \(s_1, s_2\)
> - 変数 \(s\) と文の並び \(S\) に対して、 while-not-zero \(s\) \(S\) end;
> 
> while-minus 言語、あるいは while プログラムとは、 while 言語の文の並びのこととする。文の並びに対して各文を ";" で区切ってあらわす。

> **Note**
> 適当に作ったから余計なものが入っていると思う。
> 具体例の部分でわかるように、copy はいらなそう。

各変数を自然数へ対応付ける写像を環境という。
環境 \(s\) と変数 \(x\) および自然数 \(n\) に対して、環境 \(s[x \rightarrow n]\) を、変数 \(x\) に対しては \(n\) としそれ以外の変数 \(y\) では \(s(y)\) として定める。

while minus 言語に意味を与える。
直感的には以下のものでよさそうに思える。

> **Wrong** Definition
> 文 \(S\) と環境 \(e\) に対して環境 \([e]S\) を次で定める。
> - \([e](\text{increment} x) = e[x \rightarrow e(x) + 1]\)
> - \([e](\text{decrement} x) = e[x \rightarrow e(x) - 1]\)
> - \([e](\text{reset} x) = e[x \rightarrow 0]\)
> - \([e](\text{copy} x y) = e[x \rightarrow e(y)]\)
> - \([e](\text{while-not-zero} x (s_1; \ldots s_n;)) = e\) if \(e(x) = 0\) 
> - \([e](\text{while-not-zero} x (s_1; \ldots s_n;)) = [ \cdots [ [ [e]s_1] s_2] \cdots s_n] (\text{while-note-zero} x (s_1; \ldots s_n))\) if \(e(x) \not = 0\)
> 
while minus はこれで全てである、と言いたいところだが、これでは"関数"の定義になっているかどうかが怪しい。
なぜなら、 \([e]\text{while-not-zero} x S\) が「必ず何らかの値に落ち着くこと」が保証されていないためである。
そのため、定義する際は次のように関係式として定義するのが普通である。

> **Definition**
> 文 \(S\) と環境 \(e_1, e_2\) に対して、関係 \(\langle e_1, S \rangle \rightarrow e_2\) を次で定める。
> - \(\langle e, \text{increment} x \rangle \rightarrow e[x \rightarrow e(x) + 1]\)
> - \(\langle e, \text{decrement} x \rangle \rightarrow e[x \rightarrow e(x) - 1]\)
> - \(\langle e, \text{reset} x \rangle\rightarrow e[x \rightarrow 0]\)
> - \(\langle e, \text{copy} x y \rangle \rightarrow e[x \rightarrow e(y)]\)
> - \(\langle e, \text{while-not-zero} x (s_1; \ldots s_n;) \rangle \rightarrow e\) if \(e(x) = 0\) 
> - \(\langle e, \text{while-not-zero} x (s_1; \ldots s_n;) \rangle \rightarrow e^\prime\) if
>   - \(e(x) \not = 0\)
>   - \(\langle e_i, s_i \rangle \rightarrow e_{i+1}\) for \(i = 1, \ldots , n-1\) ただし \(e_1 = e\)
>   - \(\langle e_{n+1}, \text{while-not-zero} x (s_1; \ldots s_n;) \rangle \rightarrow e^\prime\)

こうして得られた関係が文 \(S\) の意味を表すことになる。
この関係は部分関数的になることがわかる（示してない命題）。

> **Remark**
> この関係は普通は \(\langle S, e_1 \rangle \downarrow e_2\) みたいに書きそう。

> **Note**
> この意味論の定義は big semantics になりそう。
> small step semantics との関係について後で述べることをいう。

プログラム言語に意味を与える場では、しばしば関数的に定義することができない場面に直面するため、
部分関数として意味を与えるか、関係として意味を与えることが多い。
ラムダ計算もそうであった。

## play ground

<component id="while_minus_lang_playground">

## 再帰関数の計算

ところでこの while-minus 言語の計算能力も気になるが、実はこれはチューリングマシンと等価であることがわかる。

ここではそれを見る。
ただし、実用上（？）は入力に使う変数と出力に使う変数を定めておいたほうが良い。

変数 \(x_1, \ldots x_n\) と \(y\) に着目し、while-minus プログラム \(S\) が与えられたとき
\(y = S(x_1, \ldots x_n)\) とあらわされる自然数部分関数を次のように定める。
\((a_1, \ldots a_n) \in \mathbb{N}^n\) に対して環境 \(e\) を \(\{x_i \mapsto a_i\}\) により定義する。
もし \(\langle e, S \rangle \rightarrow e^\prime\) となるものが存在するとき \(S(a_1, \ldots, a_n) = e(y)\) とし、
そうでない場合は \((a_1, \ldots a_n) \not \in \text{domain} (y = S(x_1, \ldots x_n))\) とする。

さらに、プログラム \(S\) が環境に与える影響をみつもるため、ラムダ計算のように変数が束縛されているかどうかを見極めたほうが良い。
変数の集合 \(\text{SV}(S)\) を次のように定める。
- \(\text{SV}(\text{increment} x) = \{x\}\),\(\text{SV}(\text{decrement} x) = \{x\}\),\(\text{SV}(\text{reset} x) = \{x\}\),
- \(\text{SV}(\text{copy} x y) = \{x\}\)
- \(\text{SV}(\text{while-not-zero} x (s_1; \cdots s_n;)) = \bigcup_i \text{SV} s_i)

> **Note**
> 束縛と代入は気を付けたほうがいい語彙である（確か過去にネットで話題になったやつ）。
> 多分束縛というよりは代入のほうがニュアンスが近いのか？

あとは変数の総入れ替えを考えればよい？

作ったのは以下になる

<component id="recursive_function_to_while_minus_lang">
