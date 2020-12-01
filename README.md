# efi

[![Crates.io](https://img.shields.io/crates/v/efi)](https://crates.io/crates/efi)

A framework for writing UEFI applications in Rust. Acts like Rust standard library on the UEFI platform with support for things like:

- Console I/O
- Containers such as `Vec` and `String` via a custom allocator
- Macros like `println!`, `write!`, `format!` etc.
- Rust I/O primitives as `Read` and `Write` traits and the related types
- UDP and TCP sockets similar to those in stdlib
- Implementation of `IpAddr` and its supporting types
- Domain name resolution so that you can connect sockets using a hostname

Also offers an ergonomic API for UEFI-specific functionality such as:

- Loading and starting images
- DHCP
- PXE
- Device paths

Lastly, also exposes the raw underlying API to do FFI with the UEFI platform. Itself uses the same FFI API to implement above functionality.

## Limitations

- Is a work in progress. API surface can change without notice.
- Currently only `x64` architecture is supported.
- Tested to compile only with Rust nightly version `nightly-2020-10-30`. May not compile with others. You must force this version using a `rust-toolchain` file (as shown in the following section)

## Writing a UEFI Application

To write a UEFI application using this framework follow the below steps:

1. Create a new crate for your application by running `cargo new my_efi_app`, where "my_efi_app" is the name of the application
2. Add `efi = "0.2"` under `[dependencies]` in `Cargo.toml`
3. Add a file named "rust-toolchain" containing the text `nightly-2020-10-30` at the root of the crate. This will ensure that the crate is always built with nightly-2020-10-30.
4. Add the below code in `my_efi_app/src/main.rs`. Comments in the code explain each part:

```rust
#![no_std] // Indicates to the Rust compiler that the app does not depend on the standard library but is a 'standalone' application.
#![no_main] // Indicates that this application does not have a "main" function typically found in a Linux or Windows application (although it does have its own "main" function "efi_main" as declared below)
#![feature(alloc_error_handler)] // Needed for the alloc error handler function declared below since this feature is unstable.

// Externs for efi and alloc crates (alloc crate is the one that contains definitions of String and Vec etc.)
#[macro_use] extern crate efi;
#[macro_use] extern crate alloc;


// EFI entrypoint or main function. UEFI firmware will call this function to start the application.
// The signature and the name of this function must be exactly as below.
#[no_mangle]
pub extern "win64" fn efi_main(image_handle: efi::ffi::EFI_HANDLE, sys_table : *const efi::ffi::EFI_SYSTEM_TABLE) -> isize {
    efi::init_env(image_handle, sys_table); // Call to init_env must be the first thing in efi_main. Without it things like println!() won't work

    println!("Welcome to UEFI");

    // Your business logic here

    0
}

// A handler to respond to panics in the code. Required by the Rust compiler
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// A handler to respond to allocation failures. Required by the Rust compiler
#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    loop {}
}
```

### Building

Build the application by running `cargo build -Z build-std=core,alloc --target x86_64-unknown-uefi`. When the build completes the resulting EFI application `my_efi_app.efi` will be found in `target\x86_64-unknown-uefi\debug\`

### Running

Run the UEFI appliction in a qemu virtual machine by following the below steps:

1. Download and install qemu
2. Google for `ovmf.fd` and download that binary (This is the OVMF firmware under which we will run the application)
3. Start qemu by running this commandline: `<path where qemu is installed>/qemu-system-x86_64 -pflash <path where you downloaded ovmf.fd>/ovmf.fd -hda fat:rw:<path to your uefi application crate>/target/x86_64-unknown-efi/debug`
4. Qemu will boot into `ovmf.fd` firmware and start the EFI shell
5. Wait for EFI shell command prompt. When it appears enter the application's name `my_efi_app.efi` and press `ENTER`
6. The application will run and print "Welcome to UEFI" on the qemu screen

### Example Application

For a sample application see [`examples/sample_efi_app.rs`](examples/sample_efi_app.rs). Build it by running `cargo build -Z build-std=core,alloc --target x86_64-unknown-uefi --example sample_efi_app`. The resulting binary `sample_efi_app.efi` will be found in `target\x86_64-unknown-uefi\debug\examples\`. You can run it in qemu the same way as above, but it performs some network communication. Therefore you have to start qemu with network support. To do that:

1. Install a TAP adapter of your choice. Note the name of the newly-created TAP adapter
2. In the qemu commandline include the `-tap` option to add the TAP adapter to the qemu virtual machine. The full commandline would be something like this: `<path where qemu is installed>/qemu-system-x86_64 -pflash <path where you downloaded ovmf.fd>/ovmf.fd -hda fat:rw:<path to your uefi application crate>/target/x86_64-unknown-efi/debug -net tap,ifname=<name of your TAP adapter> -net nic`
