# efi

A framework for writing UEFI applications in Rust.

Nothing here yet. Work in progress.

## Building

Use nightly Rust to build. Has been tested with `nightly-2018-03-30-x86_64-pc-windows-msvc`. May not work with latest nightlies, especially because the allocator API is in flux. We're using a `rust-toolchain` file to pin the rust version to `nightly-2018-03-30`. If this version isn't already installed, `cargo` will automatically download and install it before building.