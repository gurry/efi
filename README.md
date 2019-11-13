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

WARNING: this crate is still a work in progress and the API surface can change without notice. Currently only `x64` architecture is supported

## Writing a UEFI Application

To write a UEFI application using this framework follow the below steps:

1. Install `cargo-xbuild` by running `cargo install cargo-xbuild`
2. Switch to nightly Rust by running `rustup default nightly`
3. Create a new crate for your application by running `cargo new my_efi_app`, where "my_efi_app" is the name of the application
4. Add `efi = "0.2"` under `[dependencies]` in `Cargo.toml`
5. Add the below code in `my_efi_app/src/main.rs`. Comments in the code explain each part:

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

6. Build the application by running `cargo xbuild --target x86_64-unknown-uefi`
7. When the build complete the resulting EFI application `my_efi_app.efi` will be found in `target\x86_64-unknown-uefi\debug\`

Load the applicationin qemu and run it via EFI shell. You will need the OVMF firmware for this. Google `using ovmf in qemu`for details.

### Example

For a sample application see [`examples/sample_efi_app.rs`](examples/sample_efi_app.rs). Build it by running `cargo xbuild --target x86_64-unknown-uefi --example sample_efi_app`. The resulting binary `sample_efi_app.efi` will be found in `target\x86_64-unknown-uefi\debug\examples\`.
