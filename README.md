# badge-2021-rs

This is a `no_std` application targeting Espressif's **ESP32** (Xtensa).

It was generated using this template:

```bash
$ cargo generate https://github.com/esp-rs/esp-template
```

[cargo-generate]: https://github.com/cargo-generate/cargo-generate

We are using an ESP32-S3 chip with the current module on <https://github.com/das-labor/badge-2021>.

## Preparation

Follow the [toolchain installation instruction from the esp-rs book](
https://esp-rs.github.io/book/installation/installation.html#xtensa).

## Usage

To build, flash and monitor the app, run `make flash`.

## Resources

Check out the [Embedded Rust Book](https://docs.rust-embedded.org/book/)
and have a look at [Ferrous Systems' book on Rust for Espressif
chips](https://espressif-trainings.ferrous-systems.com/).

See <https://github.com/shanemmattner/ESP32-C3_Rust_Tutorials>
for concrete examples.

## License

MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
