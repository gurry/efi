#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use] extern crate efi;
#[macro_use] extern crate alloc;

use efi::ffi;
use efi::{
    SystemTable,
    net,
    init_env,
    io::{self, Read, BufRead},
    EfiErrorKind,
};

use alloc::string::String;
use core::panic::PanicInfo;


// EFI entry point. This function is the one that the UEFI platform calls when this image is loaded.
#[no_mangle]
pub extern "win64" fn efi_main(image_handle: ffi::EFI_HANDLE,
                                sys_table : *const ffi::EFI_SYSTEM_TABLE) -> isize {

    init_env(image_handle, sys_table);
    let mut sys_table = SystemTable::new(sys_table).expect("Failed to initialize system table");

    if let Err(msg) = run(&mut sys_table) {
        println!("Exiting: {}", msg);
    };

    0
}

fn run(_sys_table: &mut SystemTable) -> Result<(), String> {
    println!("Hello from UEFI");
    println!("");

    let pxe_protocols = net::pxebc::PxeBaseCodeProtocol::get_all_mut()
            .map_err(|_| "error while locating PXE protocols")?;
    
    let pxe_protocol = &pxe_protocols[0];
    
    if pxe_protocol.cached_dhcp_config().unwrap_or(None).is_none() { // If there's cached config then DHCP has already happend. Otherwise we start it.
        println!("Performing DHCP...");
        let dhcp_config = pxe_protocol.run_dhcp().map_err(|e| format!("Dhcp failed - {}", e))?;

        println!("    Your IP: {}, Subnet mask: {}", dhcp_config.ip(), dhcp_config.subnet_mask());
        if let Some(server_ip) =  dhcp_config.dhcp_server_addr() {
            println!("    Server IP: {}", server_ip);
        }
    }

    println!("");
    println!("Testing TCP by sending HTTP request to the given addr");

    print!("Enter addr to connect to (<host>:<port>): ");
    let stdin = efi::stdin();
    let addr = stdin.lines().next().unwrap().unwrap();

    println!("Connecting to {}...", addr);

    net::TcpStream::connect(addr)
        .and_then(|mut stream| {
            println!("Connected!");
            let buf = "GET / HTTP/1.1".as_bytes();
            use io::Write;

            stream.write(&buf).unwrap();
            stream.write("\r\n".as_bytes()).unwrap();
            stream.write("Content-Length: 0\r\n".as_bytes()).unwrap();
            stream.write("\r\n".as_bytes()).unwrap();

            println!("Req sent");

            println!("");
            println!("Received resp: ");
            let mut rbuf = [0_u8; 2048];

            let read = stream.read(&mut rbuf).unwrap();

            if read == 0 {
                return Err(EfiErrorKind::NoResponse.into())
            }

            let resp = String::from_utf8_lossy(&rbuf[..read]).into_owned();

            println!("{}", resp);

            println!("");

            Ok(())
        })
        .or_else(|e| {
            Err(format!("Failed to connect. Status code: {:?}", e))
        })?;

    Ok(())
}

// The below code is required to make Rust compiler happy. Without it compilation will fail.
// But if you want you can use these functions to handle panics and allocation failures.
#[panic_handler]
fn panic(_panic: &PanicInfo) -> ! {
    loop {}
}

#[alloc_error_handler]
fn alloc_error(_: core::alloc::Layout) -> ! {
    loop {}
}