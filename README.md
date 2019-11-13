# efi

A framework for writing UEFI applications in Rust. It is intended to act like the Rust standard library on the UEFI platform with support for things like:

- Console I/O
- Containers such as `Vec` and `String` via a custom allocator
- Macros like `println!`, `write!`, `format!` etc.
- Rust I/O primitives as `Read` and `Write` traits and the related types
- UDP and TCP sockets similar to those in stdlib
- Implementation of `IpAddr` and its supporting types
- Domain name resolution so that you can connect sockets using a hostname

In addition, it offers an ergonomic API for UEFI-specific functionality such as:

- Loading and starting images
- DHCP
- PXE
- Device paths

Thirdly it exposes an API for doing raw FFI with the UEFI platform. It's the same FFI API that is used to implement the above functionality.

WARNING: this crate is still a work in progress and the API surface can change without notice.

## How to Use

To see how to use take a look at the sample application  [`examples/sample_efi_app.rs`](examples/sample_efi_app.rs) created using this library.

To build `sample_efi_app.rs`:

1. Unless already installed, install `cargo-xbuild` by running `cargo install cargo-xbuild`.
2. Switch to nightly Rust.
3. Execute the build by running `cargo xbuild --target x86_64-unknown-uefi --example sample_efi_app`

When the build complete the resulting EFI application `sample_efi_app.efi` will be found in `target\x86_64-unknown-uefi\examples\`. Load this up in qemu and run it via EFI shell. You will need the OVMF firmware for this. Google `using ovmf in qemu`for details.