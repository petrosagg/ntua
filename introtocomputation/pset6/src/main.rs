use std::collections::{HashMap, HashSet, VecDeque};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

/// The starting state of a machine
const START: State<'static> = State("S");

/// The halting state of a machine
const HALT: State<'static> = State("H");

/// The symbol for a blank spot on the tape
const BLANK: Symbol<'static> = Symbol("_");

/// The wildcard symbol to faciliate describing states that mostly traverse the tape and only take
/// action when they find a particular symbol
const WILDCARD: Symbol<'static> = Symbol("*");

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
    delta: HashMap<(State<'a>, Symbol<'a>), HashSet<(State<'a>, Symbol<'a>, Direction)>>,
}

/// The type of move a Turing machine can make
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
            //
            let Some(((q0, s0), ((q1, s1), m))) = q0.zip(s0).zip(q1.zip(s1).zip(m)) else {
                return Err("invalid δ transition entry".into());
            };

            if s0 == WILDCARD {
                for &symbol in &gamma {
                    let mut transitions = delta.entry((q0, symbol)).or_insert_with(HashSet::new);
                    if transitions.is_empty() {
                        transitions.insert((q1, symbol, m));
                    }
                }

            } else {
                delta.entry((q0, s0)).or_insert_with(HashSet::new).insert((q1, s1, m));
            }
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
        // if !missing.is_empty() {
        //     for (state, symbol) in missing {
        //         println!("{} {}", state.0, symbol.0);
        //     }
        //     return Err("the transitions above are missing from δ transition function".into());
        // }

        Ok(Self {
            sigma,
            gamma,
            q,
            delta,
        })
    }

    // Instantiate this machine with the given input
    fn instantiate<'m>(&'m self, input: &'a str) -> MultiConfiguration<'m, 'a> {
        let mut configurations = VecDeque::new();
        let mut suffix: VecDeque<_> = input.split_whitespace().map(Symbol).collect();
        // TODO: check that the input contains only symbols from Σ
        let cur = suffix.pop_front().unwrap_or(BLANK);
        configurations.push_back(Configuration {
            prefix: VecDeque::new(),
            head: (START, cur),
            suffix,
        });
        MultiConfiguration {
            machine: self,
            configurations,
        }
    }
}

/// The compound configuration of a non-deterministic Turing machine
#[derive(Debug)]
struct MultiConfiguration<'m, 'a> {
    machine: &'m MachineDescription<'a>,
    configurations: VecDeque<Configuration<'a>>,
}

impl MultiConfiguration<'_, '_> {
    fn step(&mut self) -> bool {
        let mut made_progress = false;
        for _ in 0..self.configurations.len() {
            let conf = self.configurations.pop_front().unwrap();
            let Some(actions) = self.machine.delta.get(&conf.head) else {
                continue;
            };
            for &(new_state, new_symbol, dir) in actions {
                made_progress = true;
                let mut conf = conf.clone();
                conf.step(new_state, new_symbol, dir);
                self.configurations.push_back(conf);
            }
        }
        made_progress
    }

    fn display<W: WriteColor>(&self, mut w: W) -> std::io::Result<()> {
        for conf in self.configurations.iter() {
            conf.display(&mut w)?;
        }
        Ok(())
    }

    fn halted(&self) -> bool {
        self.configurations.iter().all(|c| c.halted())

    }
}

#[derive(Debug, Clone)]
struct Configuration<'a> {
    prefix: VecDeque<Symbol<'a>>,
    head: (State<'a>, Symbol<'a>),
    suffix: VecDeque<Symbol<'a>>,
}

impl<'a> Configuration<'a> {
    fn step(&mut self, new_state: State<'a>, new_symbol: Symbol<'a>, dir: Direction) {
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
            if symbol.0.len() > 1 {
                let mut state_color = ColorSpec::new();
                state_color
                    .set_fg(Some(Color::Black))
                    .set_bg(Some(Color::Blue))
                    .set_bold(true);
                w.set_color(&state_color)?;
            }
            w.write_all(symbol.0.as_bytes())?;
            w.reset()?;
        }
        let mut state_color = ColorSpec::new();
        state_color
            .set_fg(Some(Color::Black))
            .set_bg(Some(Color::Red))
            .set_bold(true);
        w.set_color(&state_color)?;
        w.write_all(self.head.0 .0.as_bytes())?;
        state_color
            .set_fg(Some(Color::Black))
            .set_bg(Some(Color::Cyan))
            .set_bold(true);
        w.set_color(&state_color)?;
        w.write_all(self.head.1 .0.as_bytes())?;
        w.reset()?;
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
        if !configuration.step() {
            break;
        }
        configuration
            .display(&mut stdout)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
