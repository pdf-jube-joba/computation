use std::{collections::HashMap, fmt::Display};

use yew::{html, Component, Properties};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegisterIndex(usize);

#[derive(Debug, Clone)]
struct Number(usize);
impl Number {
    fn inc(self) -> Self {
        Number(self.0 + 1)
    }
    fn dec(self) -> Self {
        if self.0 != 0 {
            Number(self.0 - 1)
        } else {
            Number(0)
        }
    }
    fn clr() -> Self {
        Number(0)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ProgramIndex(usize);
impl ProgramIndex {
    fn next(&mut self) {
        self.0 += 1;
    }
    fn goto(&mut self, index: ProgramIndex) {
        todo!()
    }
}

enum Operation {
    Inc(RegisterIndex),
    Dec(RegisterIndex),
    Clr(RegisterIndex),
    Ifz(RegisterIndex, ProgramIndex),
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inc(index) => write!(f, "INC register:{}", index.0),
            Self::Dec(index) => write!(f, "DEC register:{}", index.0),
            Self::Clr(index) => write!(f, "CLR register:{}", index.0),
            Self::Ifz(r_index, p_index) => {
                write!(f, "IFZ register:{} program:{}", r_index.0, p_index.0)
            }
            _ => todo!(),
        }
    }
}

struct Code(Vec<Operation>);

struct Registers(HashMap<RegisterIndex, Number>);
impl Registers {
    fn get(&self, index: &RegisterIndex) -> Number {
        match self.0.get(index) {
            Some(num) => num.clone(),
            None => Number(0),
        }
    }
    fn set(&mut self, index: RegisterIndex, num: Number) {
        match self.0.get_mut(&index) {
            Some(target) => {
                *target = num;
            }
            None => {
                self.0.insert(index, num);
            }
        }
    }
}

struct CounterMachine {
    code: Code,
    program_counter: ProgramIndex,
    registers: Registers,
}

impl CounterMachine {
    fn is_terminate(&self) -> bool {
        self.code.0.len() <= self.program_counter.0
    }
    fn step(&mut self) {
        if !self.is_terminate() {
            let operation = &self.code.0[self.program_counter.0];
            match operation {
                Operation::Inc(index) => {
                    let num = self.registers.get(index);
                    self.registers.set(index.clone(), num.inc());
                    self.program_counter.next();
                }
                Operation::Dec(index) => {
                    let num = self.registers.get(index);
                    self.registers.set(index.clone(), num.dec());
                    self.program_counter.next();
                }
                _ => todo!(),
            }
        }
    }
}

#[derive(Default)]
struct CounterMachineView {
    machine: Option<CounterMachine>,
}

#[derive(Debug, Default, Clone, PartialEq, Properties)]
struct CounterMachineProp {}

struct CounterMachineMsg {}

impl Component for CounterMachineView {
    type Message = CounterMachineMsg;
    type Properties = CounterMachineProp;
    fn create(ctx: &yew::Context<Self>) -> Self {
        Self::default()
    }
    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let html1 = if let Some(machine) = &self.machine {
            let code_html: yew::Html = (&machine.code.0)
                .into_iter()
                .enumerate()
                .map(|(i, s)| {
                    let v = if machine.program_counter == ProgramIndex(i) {
                        "selected"
                    } else {
                        "not selected"
                    };
                    html! {
                    <>
                        <div class={v}>
                            {s}
                        </div> <br/>
                    </>}
                })
                .collect();
            html! {
                <>
                    {"machine"}
                    {code_html}
                </>
            }
        } else {
            html! {
                <>
                    {"not found"}
                </>
            }
        };

        html! {
            {html1}
        }
    }
}
