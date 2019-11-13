# efi

A framework for writing UEFI applications in Rust. It is intended to act as Rust standard library on the UEFI platform with support for things like:

- Console I/O
- Containers such as `Vec` and `String` via a custom allocator
- Macros like `println!`, `write!`, `format!` etc.
- Rust I/O primitives as `Read` and `Write` traits and the related types
- UDP and TCP sockets similar to those in stdlib
- Implementation of `IpAddr` and its supporting types
- Domain name resolution so that you can connect sockets using a hostname

In addition to the above, it offers an ergonomic API for UEFI-specific functionality such as:

- Loading and starting images
- DHCP
- PXE
- Device paths

Thirdly it exposes an API for doing raw FFI with the UEFI platform as well. It's the same FFI API that is used to implement the above mentioned functionality.

WARNING: this crate is still a work in progress and the API surface can change without notice.

## How to Use

To see how to use take a look at the sample application [`efi_app`](https://github.com/gurry/efi_app) which is built using `efi`.

### Building

1. Install `cargo-xbuild` if not already installed
2. Use nightly Rust and the command line `cargo xbuild --target x86_64-unknown-uefi` to build.