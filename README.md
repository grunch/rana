# Rana 🐸

![Rana](rana.png)

Mine public keys that can be used with nostr.

This is based on [nip13](https://github.com/ok300/nostr-rs/blob/master/examples/nip13.rs) example.

Provide the desired difficulty or the vanity prefix as arguments. See below.

## Requirements:

0. You need Rust version 1.64 or higher to compile.

## Install

Using Cargo to install (requires ~/.cargo/bin to be in PATH)

```bash
$ cargo install rana
```

### Compile and execute it:

To compile on Ubuntu/Pop!\_OS/Debian, please install [cargo](https://www.rust-lang.org/tools/install), then run the following commands:

```bash
$ sudo apt update
$ sudo apt install -y cmake build-essential
```

Then clone the repo, build and run:

```bash
$ git clone https://github.com/grunch/rana.git
$ cd rana
$ cargo run --release
```

By default it will generate a public key with a difficulty of `10` but you can customize its difficulty or vanity prefix with the proper parameters.

Usage:

```
Options:
  -d, --difficulty <DIFFICULTY>
          Enter the number of starting bits that should be 0. [default: 10]
  -v, --vanity-prefix <VANITY_PREFIX>
          Enter the prefix your public key should have when expressed
          as hexadecimal. This can be combined with --vanity-n,
          but beware of extra calculations required. [default: ]
  -n, --vanity-npub-prefix <VANITY_NPUB_PREFIX>
          Enter the prefix your public key should have when expressed
          in npub format (Bech32 encoding).
          This can be combined with --vanity, but beware of extra
          calculations required. [default: ]
```

Examples:

```bash
$ cargo run --release -- --difficulty=20

$ cargo run --release -- --vanity=dead

$ cargo run --release -- --vanity-n=rana
```

If you have it installed with `cargo install`:

```bash
$ rana --difficulty=20

$ rana --vanity=dead

$ rana --vanity-n=rana
```

Keep in mind that you cannot specify a difficulty and a vanity prefix at the same time.
Also, the more requirements you have, the longer it will take to reach a satisfactory public key.
