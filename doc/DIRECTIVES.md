# `foof`'s directive system
Using `foof` is simple. Just create a `.rs` file contaning your code,
headlined by a special comment called a *directive*. Directives look like this:
```
// foof: use rand 0.8.5
// foof: use tokio latest with (full)
// foof: toolchain nightly
```
As you can see, they're just basic Rust comments written in almost plain English.
They are used to control `foof`'s behaviour when parsing your source file.

# Directive list

## `use`
The `use` directive will signal to `foof` to include a crate into your file.
The directive has two forms: basic, and feature-specifying.

Basic usage of `use` looks like this:
```
// foof: use rand latest
// ^ this includes the `rand` crate at the latest version fetched from crates.io
// (cache is updated hourly)
```
To specify one or more cargo features to be enabled, simply do this:
```
// foof: use tokio latest with (rt rt-multi-thread)
```

Syntax: `use CRATE_NAME VERSION [with (FEATURE_LIST)]`.

`VERSION` can be a SemVer number (`0.8.5`) or the special string `latest`,
which will cause `foof` to grab the latest version of the crate available on crates.io.

## `toolchain`
The `toolchain` directive specifies... well, what Rust toolchain you want to compile the file on.
It can also specify a build target.

Usage:
```
// foof: toolchain nightly
// (^ the build target is assumed to be the same as the host)

// foof: toolchain nightly targeting armv6k-nintendo-3ds
// (^ specifying a different build target than the host)
```

Syntax: `toolchain TOOLCHAIN_NAME [targeting TARGET_NAME]`.

`foof` will generate a `rust-toolchain.toml` file while constructing the `cargo` project it internally uses.

## `optimize`
The `optimize` directive specifies whether to build the executable in debug or release mode.

Usage:
```
// foof: optimise release
// foof: optimize release
// (they both work)
```

Syntax: `optimi(s|z)e debug/release`.

## `edition`
The `edition` directive controls what Rust edition to build your project with. This should likely be left alone.

Usage:
```
// foof: edition 2021
// (which is the default so this'd do nothing)
```

Syntax: `edition 2015/2018/2021`.

