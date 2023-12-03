use std::collections::{HashMap, HashSet, VecDeque};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// The starting state of a machine
const START: State<'static> = State("S");

/// The halting state of a machine
const HALT: State<'static> = State("H");

/// The symbol for a blank spot on the tape
const BLANK: Symbol<'static> = Symbol("_");

/// An element of the set Σ
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct State<'a>(&'a str);

/// An element of the set Γ
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Symbol<'a>(&'a str);

/// A description of a Turing machine
#[derive(Debug)]
struct MachineDescription<'a> {
    /// The set Σ
    sigma: HashSet<Symbol<'a>>,
    /// The set Γ
    gamma: HashSet<Symbol<'a>>,
    /// The set of states Q
    q: HashSet<State<'a>>,
    /// The transition function δ
    delta: HashMap<(State<'a>, Symbol<'a>), (State<'a>, Symbol<'a>, Direction)>,
}

/// The type of move a Turing machine can make
#[derive(Debug, Clone, Copy)]
enum Direction {
    /// A move to the left
    Left,
    /// A move to the right
    Right,
}

impl<'a> MachineDescription<'a> {
    fn parse(input: &'a str) -> Result<Self, String> {
        // Skip all comments and empty lines
        let mut lines = input
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty());

        let sigma: HashSet<_> = match lines.next() {
            Some(l) => l.split_whitespace().map(Symbol).collect(),
            None => return Err("unexpected EOF, could not read set Σ".into()),
        };

        let gamma: HashSet<_> = match lines.next() {
            Some(l) => l.split_whitespace().map(Symbol).collect(),
            None => return Err("unexpected EOF, could not read set Γ".into()),
        };

        if !gamma.contains(&BLANK) {
            return Err("missing blank symbol \"_\" from Γ".into());
        }

        let q: HashSet<_> = match lines.next() {
            Some(l) => l.split_whitespace().map(State).collect(),
            None => return Err("unexpected EOF, could not read set Q".into()),
        };

        if !q.contains(&START) {
            return Err("missing initial state \"S\" from Q".into());
        }

        if !q.contains(&HALT) {
            return Err("missing halting state \"H\" from Q".into());
        }

        let mut delta = HashMap::new();
        for line in lines {
            let mut parts = line.split_whitespace();
            let q0 = parts.next().map(State);
            let s0 = parts.next().map(Symbol);
            let q1 = parts.next().map(State);
            let s1 = parts.next().map(Symbol);
            let m = parts.next().and_then(|s| match s {
                "L" => Some(Direction::Left),
                "R" => Some(Direction::Right),
                _ => None,
            });
            // TODO: check that q0 and q1 are from Q and that s0 and s1 are from Σ

            let Some(((q0, s0), ((q1, s1), m))) = q0.zip(s0).zip(q1.zip(s1).zip(m)) else {
                return Err("invalid δ transition entry".into());
            };
            if parts.next().is_some() {
                return Err("invalid δ transition entry".into());
            }
            delta.insert((q0, s0), (q1, s1, m));
        }

        // TODO: check that the input contains only symbols from Σ

        let mut missing = vec![];
        for &state in &q {
            if state == HALT {
                continue;
            }
            for &symbol in &gamma {
                if !delta.contains_key(&(state, symbol)) {
                    missing.push((state, symbol));
                }
            }
        }
        if !missing.is_empty() {
            for (state, symbol) in missing {
                println!("{} {}", state.0, symbol.0);
            }
            return Err("the transitions above are missing from δ transition function".into());
        }

        Ok(Self {
            sigma,
            gamma,
            q,
            delta,
        })
    }

    // Instantiate this machine with the given input
    fn instantiate<'m>(&'m self, input: &'a str) -> Configuration<'m, 'a> {
        let mut suffix: VecDeque<_> = input.split_whitespace().map(Symbol).collect();
        // TODO: check that the input contains only symbols from Σ
        let cur = suffix.pop_front().unwrap_or(BLANK);
        Configuration {
            machine: self,
            prefix: VecDeque::new(),
            head: (START, cur),
            suffix,
        }
    }
}

#[derive(Debug)]
struct Configuration<'m, 'a> {
    machine: &'m MachineDescription<'a>,
    prefix: VecDeque<Symbol<'a>>,
    head: (State<'a>, Symbol<'a>),
    suffix: VecDeque<Symbol<'a>>,
}

impl Configuration<'_, '_> {
    fn step(&mut self) {
        let (new_state, new_symbol, dir) = self.machine.delta[&self.head];
        self.head = (new_state, new_symbol);
        match dir {
            Direction::Left => {
                self.suffix.push_front(self.head.1);
                self.head.1 = self.prefix.pop_back().unwrap_or(BLANK);
            }
            Direction::Right => {
                self.prefix.push_back(self.head.1);
                self.head.1 = self.suffix.pop_front().unwrap_or(BLANK);
            }
        }
    }

    fn halted(&self) -> bool {
        self.head.0 == HALT
    }

    fn display<W: WriteColor>(&self, mut w: W) -> std::io::Result<()> {
        for symbol in &self.prefix {
            w.write_all(symbol.0.as_bytes())?;
        }
        let mut state_color = ColorSpec::new();
        state_color
            .set_fg(Some(Color::Black))
            .set_bg(Some(Color::Red))
            .set_bold(true);
        w.set_color(&state_color)?;
        w.write_all(self.head.0 .0.as_bytes())?;
        w.reset()?;
        w.write_all(self.head.1 .0.as_bytes())?;
        for symbol in &self.suffix {
            w.write_all(symbol.0.as_bytes())?;
        }
        w.write_all(b"\n")?;
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let name = args.next().unwrap();
    let Some(machine) = args.next() else {
        return Err(format!("usage: {name} <MACHINE> <INPUT>"));
    };
    let Some(input) = args.next() else {
        return Err(format!("usage: {name} <MACHINE> <INPUT>"));
    };
    let machine = std::fs::read_to_string(machine).map_err(|e| e.to_string())?;
    let machine = MachineDescription::parse(&machine)?;

    let input = std::fs::read_to_string(input).map_err(|e| e.to_string())?;
    let mut configuration = machine.instantiate(&input);

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    configuration
        .display(&mut stdout)
        .map_err(|e| e.to_string())?;
    while !configuration.halted() {
        configuration.step();
        configuration
            .display(&mut stdout)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
