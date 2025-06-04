#![no_main]
#![no_std]

use core::hash::Hasher;
// Use the abstracted log interface for console output
use log::info;

// Import a bunch of commonly-used UEFI symbols exported by the crate
use uefi::prelude::*;
use core::fmt::Write; // <-- Import this trait

mod os;

use os::memory;

// Tell the uefi crate that this function will be our program entry-point
#[entry]
// Declare "hello_main" to accept two arguments, and use the type definitions provided by uefi
// Notice that this no longer needs to be declared "extern" or "pub"
fn hello_main(_image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // In order to use any of the services (input, output, etc...), they need to be manually
    // initialized by the UEFI program
    uefi_services::init(&mut system_table).unwrap();

    // Display an INFO message of "Hello world!" to the default UEFI console (typically the screen)
    let stdout = system_table.stdout();
    _ = stdout.clear();
    _ = stdout.write_str("Booting OS\n");
    
    memory::print_memory_map(&system_table);

    loop {
        // do something in here.
    }

    // Tell the UEFI firmware we exited without error
    Status::SUCCESS
}