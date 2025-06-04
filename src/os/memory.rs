use uefi::table::{Boot, SystemTable};               // Import UEFI Boot and SystemTable types
use uefi::table::boot::{MemoryType};                 // Import MemoryType enum to classify memory regions
use log::info;                                       // Import logging macro to output info-level messages

// Define the size of the buffer to hold the memory map, 16 KiB (4 pages of 4 KiB each)
const MAP_SIZE: usize = 4096 * 4;

// Define a struct that wraps the buffer and forces 8-byte alignment (required by UEFI)
#[repr(C, align(8))]
struct AlignedBuffer([u8; MAP_SIZE]);

// Declare a static mutable buffer with enforced alignment to hold the memory map data
// This buffer lives for the entire program runtime
static mut MEMORY_MAP_BUFFER: AlignedBuffer = AlignedBuffer([0; MAP_SIZE]);

// Function to print the UEFI memory map to the console via logging
// Takes a reference to the system table, which gives access to boot services
pub fn print_memory_map(system_table: &SystemTable<Boot>) {
    // Get a reference to UEFI Boot Services, which provide many runtime services including memory map retrieval
    let bt = system_table.boot_services();

    // Query UEFI to get the current memory map size and related info
    // This helps us ensure our buffer is big enough to hold the map
    let mem_map_size = bt.memory_map_size();

    // Calculate how much buffer space we need by adding a safety margin:
    // map_size is the current size, plus space for up to 8 additional MemoryDescriptor structs
    // This accounts for potential changes to the map between calls (UEFI spec recommendation)
    let needed = mem_map_size.map_size + 8 * core::mem::size_of::<uefi::table::boot::MemoryDescriptor>();

    // Create a mutable slice (&mut [u8]) from our static buffer, using all bytes
    // Unsafe is required because we are accessing a mutable static variable
    let buffer: &mut [u8] = unsafe { &mut MEMORY_MAP_BUFFER.0[..] };

    // Assert that our buffer is large enough to hold the memory map data; panic if not
    assert!(buffer.len() >= needed, "UEFI memory map buffer too small");

    // Call UEFI Boot Services to fill our buffer with the current system memory map
    // This returns a MemoryMap object representing all memory regions known by firmware
    // Panic with error message if retrieving the memory map fails
    let memory_map = bt
        .memory_map(buffer)
        .expect("Failed to retrieve UEFI memory map");

    // Iterate over each MemoryDescriptor entry in the memory map
    for desc in memory_map.entries() {
        // Check if this memory region is CONVENTIONAL RAM (i.e., usable system memory)
        if desc.ty == MemoryType::CONVENTIONAL {
            // Extract physical start address of this region
            let start = desc.phys_start;
            // Extract number of 4 KiB pages this region contains
            let pages = desc.page_count;
            // Calculate total size in bytes (pages * 4 KiB)
            let size = pages * 4096;

            // Log info to UEFI console with formatted output:
            // Show start and end physical addresses in hex, and size in KiB (1024 bytes)
            info!(
                "Usable region: {:#010x} - {:#010x} ({} KiB)",
                start,
                start + size,
                size / 1024
            );
        }
    }
}
