# sdvxio-pipe

A piped implementation of a BTools `sdvxio` that forwards calls to a child process via standard input/output pipes,
allowing to use 64bit-only libraries in a 32bit environment or vice versa.

## Crates

### sdvxio-pipe

The main crate that compiles to a BTools `sdvxio` compliant library which creates and forwards requests to a child
process via standard input/output pipes.

### sdvxio-pipe-program

A binary executable that interfaces with any `sdvxio` library. It receives and answers requests through standard
input/output pipes.

### sdvxio-pipe-proto

Shared protocol definitions used by both the proxy dll and the child process.

## Building

Build the entire workspace:

```bash
cargo build
```

Build a specific crate:

```bash
cargo build -p sdvxio-pipe
cargo build -p sdvxio-pipe-program
cargo build -p sdvxio-pipe-proto
```
