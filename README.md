# Ulmer Kernel

This repository contains my homebrew Rust OS kernel. While I plan to run this on my Pentium III PC
PC, I am still trying to make this as portable as possible so that I can eventually support more processor architectures and platforms in the future.

## Building

The following prerequisites are necessary before you can build the kernel:

- Working installation of [Rust](https://www.rust-lang.org/tools/install)
- Add nightly toolchain: `rustup default nightly`
- Install std sources: `rustup component add rust-src`

For emulator testing:

- Install QEMU: `apt install qemu-system-x86`

To build the kernel binary, just run

```
cargo build
```

## Building the Documentation

Check out the documentation by running

```
cargo doc --open
```

## License: GPLv3

Ulmer Operating System Kernel
Copyright (C) 2023-2024 Alexander Ulmer

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
