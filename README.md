# Seri

A way too much over-engineered timetable language compiler.

## Running Seri

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
Usage: seri [OPTIONS] [FILE]

Arguments:
  [FILE]  File to compile. If not present, will read from standard input

Options:
  -f, --format <FORMAT>      Output format [default: tikz] [possible values: tikz, html]
  -t, --template <TEMPLATE>  Template to use, if any
  -o, --output <FILE>        Output file. If not present, will output to stdout
  -s, --save-tmp             Keep intermediate files
  -h, --help                 Print help
  -V, --version              Print version
```

## Seri language

See https://github.com/Lugrim/seri/issues/26

## TODO

(To avoid duplicates, this section will be fully deleted soon, please use issues instead)

- [ ] Add backends
	- [x] HTML
		- [x] minimal
		- [ ] Automatically compute the number or days/hours of the seminar
		- [ ] Handle event not starting on round hours
		- [ ] Display the date on top of the planning
		- [ ] Display the abstract next to the planning
