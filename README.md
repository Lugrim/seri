# Seri

A way too much over-engineered timetable language compiler.

## Running Seri

Seri is written in Rust, so you need to install the Rust toolchain to run it.

To generate a schedule from a file, run:
```bash
cargo run data/example.seri
```

You can also generate a schedule from stdin:
```bash
cat data/example.seri > cargo run
```

## Seri compiler command-line interface

```
Usage: seri [FILE] [OUTPUT_FORMAT]

Arguments:
  [FILE]           File to compile. If not present, will read from standard input
  [OUTPUT_FORMAT]  Output format. Default  [default: tikz]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Seri language

TODO : Write a proper language specification

## TODO

- [x] Refactor as a compiler structure
	- [x] Refactor base code
	- [x] Add passes chaining
		- [x] Trait bound pass chaining (#3)
		- [x] Find a cleaner way to define chaining (#5)
	- [x] Stop being stupid and put frontend / backend in the right order
- [x] Actually load data
	- [x] From stdin
	- [x] From file (CLAP)
	- [x] I/O Error management (#1)
- [ ] Enhance frontend
	- [x] Parse more headers
		- [x] speakers
		- [x] timestamp
		- [x] duration
	- [ ] Parse description as markdown
- [ ] Add backends
	- [ ] tikz
		- [x] minimal
		- [x] Print day in header
		- [ ] load preamble and postamble from files
	- [ ] HTML
		- [ ] minimal
- [x] Documentation
- [x] Put clippy in giga chad mode
