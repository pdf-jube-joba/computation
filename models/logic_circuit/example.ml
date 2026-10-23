let code =
  {|\graph: one_shot {
    \in {I}
    \out {O=A.OUT}

    B, \BR_F {IN=I}
    N, \NOT_T {IN=B.OUT0}
    A, \AND_F {IN0=B.OUT1, IN1=N.OUT}
}

\graph: tick_add1 {
    \in {I}
    \out {O=ON.OUT}

    B, \BR_F {IN=I}
    D, \DLY_F {IN=B.OUT0}
    ON, \OR_F {IN0=B.OUT1, IN1=D.OUT}
}

\graph: RS_latch {
    \in {R, S}
    \out {Q=B0.OUT0, nQ=B1.OUT0}

    E0, one_shot {I=R}
    T01, tick_add1 {I=E0.O}
    T02, tick_add1 {I=T01.O}
    T03, tick_add1 {I=T02.O}
    T04, tick_add1 {I=T03.O}
    T05, tick_add1 {I=T04.O}
    T06, tick_add1 {I=T05.O}
    O0, \OR_T  {IN0=T06.O, IN1=B1.OUT1}
    N0, \NOT_F {IN=O0.OUT }
    B0, \BR_F  {IN=N0.OUT }

    E1, one_shot {I=S}
    T11, tick_add1 {I=E1.O}
    T12, tick_add1 {I=T11.O}
    T13, tick_add1 {I=T12.O}
    T14, tick_add1 {I=T13.O}
    T15, tick_add1 {I=T14.O}
    T16, tick_add1 {I=T15.O}
    O1, \OR_F  {IN0=T16.O, IN1=B0.OUT1 }
    N1, \NOT_T {IN=O1.OUT }
    B1, \BR_T  {IN=N1.OUT }
}

\graph: DFF {
    \in {CK, D}
    \out {Q=R.Q, nQ=R.nQ}

    E, one_shot {I=CK}
    B, \BR_F {IN=E.O}
    B2, \BR_F {IN=D}
    D11, \DLY_F {IN=B2.OUT0}
    D12, \DLY_F {IN=D11.OUT}
    D2, \DLY_F {IN=B2.OUT1}
    N2, \NOT_T {IN=D2.OUT}
    A1, \AND_F {IN0=B.OUT0, IN1=D12.OUT}
    A2, \AND_F {IN0=B.OUT1, IN1=N2.OUT}
    R, RS_latch {R=A2.OUT, S=A1.OUT}
}

\graph: XOR {
    \in {IN0 IN1}
    \out {OUT=O.OUT}
    B0, \BR_F {IN=IN0}
    B1, \BR_F {IN=IN1}
    D0, \DLY_F {IN=B0.OUT0}
    N1, \NOT_T {IN=B1.OUT0}
    A0, \AND_F {IN0=D0.OUT IN1=N1.OUT}
    N0, \NOT_T {IN=B0.OUT1}
    D1, \DLY_F {IN=B1.OUT1}
    A1, \AND_F {IN0=N0.OUT IN1=D1.OUT}
    O, \OR_F {IN0=A0.OUT IN1=A1.OUT}
}

\graph: ADDR {
    \in {I0 I1}
    \out {S=X.OUT C=A.OUT}
    B0, \BR_F {IN=I0}
    B1, \BR_F {IN=I1}
    X, XOR {IN0=B0.OUT0 IN1=B1.OUT0}
    A, \AND_F {IN0=B0.OUT1 IN1=B1.OUT1}
}
|}

let utils_map () = Parser.parse code []
