#![no_main]
#![no_std]

mod os;


// Import a bunch of commonly-used UEFI symbols exported by the crate
use uefi::prelude::*;
use core::fmt::Write;

// Tell the uefi crate that this function will be our program entry-point
#[entry]
fn os_main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {

    // To use any of the services (input, output, etc...), they need to be manually
    // initialized by the UEFI program
    uefi_services::init(&mut system_table).unwrap();

    let stdout = system_table.stdout();
    _ = stdout.clear();
    _ = stdout.write_str("Booting OS\n");

    loop {
        // do something in here.
    }

    // Tell the UEFI firmware we exited without error
    Status::SUCCESS
}