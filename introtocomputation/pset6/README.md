## Usage

```
cargo run <MACHINE> <INPUT>
```

## Examples

A non-determinsitic machine were one strand replaces its input with zeros and the other with ones.

```
cargo run examples/nd-machine.txt examples/nd-input.txt
```

A deterministic machine that doubles a number in unary

```
cargo run examples/unary-double-machine.txt examples/unary-double-input.txt
```
