#![feature(coroutines, coroutine_trait, step_trait)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(dead_code)] // remove this

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pulse {
    Low,
    High,
}

enum Kind {
    FlipFlop { on: bool },
    Conjunction { last_pulse_by_module: HashMap<String, Pulse> },
    Broadcaster,
    Button,
}

impl Kind {
    fn new_flip_flop() -> Self { Self::FlipFlop { on: false } }

    fn new_conjunction() -> Self {
        Self::Conjunction { last_pulse_by_module: HashMap::default() }
    }

    fn connected(&mut self, src: String) {
        if let Self::Conjunction { last_pulse_by_module } = self {
            last_pulse_by_module.insert(src, Pulse::Low);
        }
    }
}

impl Kind {
    fn receive_pulse(&mut self, src: &str, pulse: Pulse) -> Option<Pulse> {
        match self {
            Self::FlipFlop { on } => match pulse {
                Pulse::Low => {
                    *on = !*on;
                    Some(if *on { Pulse::High } else { Pulse::Low })
                }
                Pulse::High => None,
            },
            Self::Conjunction { last_pulse_by_module } => {
                last_pulse_by_module.insert(src.to_owned(), pulse);
                if last_pulse_by_module.values().all(|p| *p == Pulse::High) {
                    Some(Pulse::Low)
                } else {
                    Some(Pulse::High)
                }
            }
            Self::Broadcaster => Some(pulse),
            Self::Button => Some(Pulse::Low),
        }
    }
}

struct Module {
    name: String,
    kind: Kind,
    dst: Vec<Rc<RefCell<Module>>>,
}

impl Module {
    fn new(name: String, kind: Kind) -> Self {
        Self { name, kind, dst: Vec::default() }
    }

    fn new_rc(name: String, kind: Kind) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(name, kind)))
    }

    fn connect(&mut self, dst: Rc<RefCell<Module>>) {
        self.dst.push(dst.clone());
        dst.borrow_mut().kind.connected(self.name.clone())
    }

    fn receive_pulse(&mut self, src: &str, pulse: Pulse, queue: &mut Queue) {
        if let Some(pulse) = self.kind.receive_pulse(src, pulse) {
            for dst in &self.dst {
                queue.push_back((dst.borrow_mut().name.clone(), pulse));
            }
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) { self.dst.clear() }
}

type Queue = VecDeque<(String, Pulse)>;

fn main() {
    let button = Module::new_rc("button".to_owned(), Kind::Button);
    let broadcaster =
        Module::new_rc("broadcaster".to_owned(), Kind::Broadcaster);

    let a = Module::new_rc("a".to_owned(), Kind::new_flip_flop());
    let b = Module::new_rc("b".to_owned(), Kind::new_flip_flop());
    let c = Module::new_rc("c".to_owned(), Kind::new_flip_flop());
    let inv = Module::new_rc("inv".to_owned(), Kind::new_conjunction());

    button.borrow_mut().connect(broadcaster.clone());

    broadcaster.borrow_mut().connect(a.clone());
    broadcaster.borrow_mut().connect(b.clone());
    broadcaster.borrow_mut().connect(c.clone());

    a.borrow_mut().connect(b.clone());
    b.borrow_mut().connect(c.clone());
    c.borrow_mut().connect(inv.clone());
    inv.borrow_mut().connect(a.clone());

    let mut queue = Queue::new();
    button.borrow_mut().receive_pulse("", Pulse::Low, &mut queue);
}
