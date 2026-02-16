# 論理回路
## ざっくりとした説明と動く見本
> [!WARNING] 
> 動く見本は今はありません。

オフとオンのみがある信号を考える（ \(\{0,1\}\) と思ってよい）。
ゲートと呼ばれる基本要素は、複数の信号を入力に、一つの出力を決定する。
論理回路はこのゲートの入力と出力を組み合わせて信号を伝番させて計算を行う。
ゲートは、 AND, OR, NOT などがある。
ちゃんと知りたい場合は調べればすぐに出る。
ここではそれに加えていくつゲートを追加しているので注意。

下の例は、 XOR と呼ばれる機能を実現する論理回路の例。
"IN0" とかをクリックするとオン（緑）とオフ（赤）が切り替えられる。
`Step` を押せば次の状態に移る。

<!--
<script type="module">
    import { load, LogicCircuitViewModel } from "../assets/generated/logic_circuit/logic_circuit_glue.js";
    import { TextAreaSource, TextDefinedSource, UserControls } from "../assets/utils.js";
    await load();

    let res1 = await fetch("../assets/component/logic_circuit_to_machine/XOR.txt");
    let txt1 = await res1.text();

    let code_input1 = new TextDefinedSource(txt1);
    let control1 = new UserControls("control1");

    let default_placement = new Map();
    default_placement.set("B0", { x: 0, y: 70 });
    default_placement.set("B1", { x: 200, y: 70 });
    default_placement.set("D0", { x: 0, y: 140 });
    default_placement.set("N0", { x: 100, y: 140 });
    default_placement.set("D1", { x: 200, y: 140 });
    default_placement.set("N1", { x: 300, y: 140 });
    default_placement.set("A0", { x: 100, y: 210 });
    default_placement.set("A1", { x: 300, y: 210 });
    default_placement.set("O", { x: 200, y: 280 });

    let view1 = new LogicCircuitViewModel(code_input1, control1, "view1", default_placement);
</script>

```XORのメモ
graph: main {
    in {IN0 IN1}
    out {OUT=O.OUT}
    B0, BR-F {IN=IN0}
    B1, BR-F {IN=IN1}
    D0, DLY-F {IN=B0.OUT0}
    N1, NOT-T {IN=B1.OUT0}
    A0, AND-F {IN0=D0.OUT IN1=N1.OUT}
    N0, NOT-T {IN=B0.OUT1}
    D1, DLY-F {IN=B1.OUT1}
    A1, AND-F {IN0=N0.OUT IN1=D1.OUT}
    O, OR-F {IN0=A0.OUT IN1=A1.OUT}
}
```

<div id="machine1">
    <div id="control1"></div>
    <div id="view1"></div>
</div>
-->
