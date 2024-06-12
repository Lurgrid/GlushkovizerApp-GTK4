# Glushkovizer

Graphical application using the `Glushkovizer library` to transform a regular
expression into an automaton and represent it and its orbits

## Glushkovizer library

For more information on the library, please visit this repository :

> https://github.com/Lurgrid/Glushkovizer

## Usage

Theoretically, the `gtk4` version can run on Windows if all dependencies are
properly installed, but has not been tested. Has only been tested on a Linux

```bash
$ cargo build --release
$ ./target/release/glushkovizer-gtk4
# or
$ cargo build --release --features no-blp
$ ./target/release/glushkovizer-gtk4
```

### Run Dependencies

- `dot 9.0 >=` _(May work on an earlier version, but has not been tested)_

  _Click [here](https://graphviz.org/download/) to install it_

- `gtk4 4.14 >=`

  _(Has also been tested in 4.6.9)_

- `libadwaita 1.5 >=`

  _(Has also been tested in 1.1.7)_

### Build Dependencies

- `blueprint-compiler 0.10 >=` _(May work on an earlier version, but has not been tested)_

  _optional with the feature `no-blp`_

- `gcc 14.1 >=` _(May work on an earlier version, but has not been tested)_

- `gtk4 devel 4.14 >=` For installation, please refer to the [book](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html)

- `libadwaita devel 1.5 >=` For installation, please refer to the [book](https://gtk-rs.org/gtk4-rs/stable/latest/book/libadwaita.html)

## License

GPLv3

---

> GitHub [@Lurgrid](https://github.com/Lurgrid)
