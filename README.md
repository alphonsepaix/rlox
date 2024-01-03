`rlox` is a Rust implementation of the Lox interpreter studied in the book
"[Crafting Interpreters][]".

[crafting interpreters]: http://craftinginterpreters.com

## Building

### Prerequisites

The development and testing primarily take place on a Linux machine,
but the project is designed to work on any system.
Everything is written in [Rust][] and orchestrated by `cargo`.
Instructions to install Rust are [here][install]. Once you have `cargo`
installed and on your path, run:

```sh
$ cargo build --release
```

[rust]: https://www.rust-lang.org/
[install]: https://www.rust-lang.org/learn/get-started

This downloads all of the packages used by the project and builds it
in release mode.

### Running

Once you've build the project, run it:

```sh
$ cargo run --release [script]
```

If everything is working, this command will either launch the REPL
(Read-Eval-Print Loop) or directly execute the script if provided.

## Testing

To ensure the correctness of the implementation and prevent regressions,
run the test suite using the following command:

```sh
$ cargo test
```