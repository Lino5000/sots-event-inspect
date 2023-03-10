# SotS Event Inspector

A tool to more easily inspect the NPC and Event data stored in the `.asset`
files of the game [Signs of the Sojourner](https://www.echodoggames.com/).

## Usage

The `PATH` that you provide should be the path to the folder that contains all
the `.asset` and `.asset.meta` files extracted from the game.

Linux:
```bash
/path/to/sots-event-inspect <PATH>
```

Windows:
```
\path\to\sots-event-inspect.exe <PATH>
```
**Important:** Make sure you run this command in `cmd.exe`, rather than
PowerShell, as the latter seems to do something strange to the path that you
provide that breaks the program.

### Display Issues

This tool makes use of some unicode characters when displaying cards, but not
every terminal is capable of displaying them. If you encounter issues then use
the `display_compat` version of the program, which replaces the symbols with
letters.

## Building

To build this tool from source, you'll need to be able to compile Rust code
with Cargo. See [here](https://www.rust-lang.org/tools/install) for
instructions on installing both Rust and Cargo.

First, clone this repo to get the source code:
```bash
git clone https://github.com/Lino5000/sots-event-inspect.git
```

Then, to build the standard version (using symbols), use the command:

```bash
cargo build --release
```

For the `display_compat` version, use the command:

```bash
cargo build --release --features display_compat
```

Whichever version you build, the resulting program can be found in the
`target/release` folder that Cargo creates.
