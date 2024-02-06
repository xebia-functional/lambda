# Setup

The build process relies on [Cargo](https://github.com/rust-lang/cargo) and
[`just`](https://github.com/casey/just). Please follow the official instructions
to [install](https://www.rust-lang.org/tools/install) Rust, which will ensure
that Cargo is available.

Then, you can install `just` via cargo:

```shell
cargo install just
```

You can install the remaining tools, mainly [`cargo-lambda`](https://github.com/cargo-lambda/cargo-lambda), using `just`:

```shell
just tools
```

# Building

Change the current working directory to this directory, i.e., the one enclosing
this very [`README.md`](./README.md). Then you can build all services thus:

```shell
cd $PROJECT/rust && just build
```

# Deploying

You can deploy directly to AWS lambda using `just`.

## `https-a`

Change the current working directory to this directory. Then you can deploy the
eponymous service like this:


```shell
just deploy https-a
```

## `events-a`

Change the current working directory to this directory. Then you can deploy the
eponymous service like this:


```shell
just deploy events-a
```

## `events-b`

Change the current working directory to this directory. Then you can deploy the
eponymous service like this:


```shell
just deploy events-b
```
