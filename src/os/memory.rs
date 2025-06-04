use uefi::table::{Boot, SystemTable};               // Import UEFI Boot and SystemTable types
use uefi::table::boot::{MemoryType};                 // Import MemoryType enum to classify memory regions


// Define a simple struct to hold information about a usable memory region
#[derive(Copy, Clone)]  // <-- Add this line
pub struct MemoryRegion {
    pub start: u64,
    pub size: u64,
}
// Maximum number of memory regions we will store
const MAX_REGIONS: usize = 32;

// Static mutable fixed-size array to store usable memory regions
// Unsafe because mutable globals can cause data races if misused
static mut USABLE_REGIONS: [MemoryRegion; MAX_REGIONS] = [MemoryRegion { start: 0, size: 0 }; MAX_REGIONS];

// Static mutable counter of how many usable regions have been stored
static mut REGION_COUNT: usize = 0;

// Function to scan UEFI memory map and store all usable (CONVENTIONAL) memory regions
// Takes a reference to the UEFI SystemTable (Boot phase) to access boot services
pub fn store_usable_memory_regions(system_table: &SystemTable<Boot>) {
    // Get a reference to UEFI Boot Services from the system table
    let bt = system_table.boot_services();

    // Size of buffer allocated to hold the raw UEFI memory map data (16 KiB)
    const MAP_SIZE: usize = 4096 * 4;

    // Define a buffer type that enforces 8-byte alignment (required by UEFI memory map)
    #[repr(C, align(8))]
    struct AlignedBuffer([u8; MAP_SIZE]);

    // Static mutable buffer to hold memory map data from UEFI firmware
    // Initialized with zeros
    static mut MEMORY_MAP_BUFFER: AlignedBuffer = AlignedBuffer([0; MAP_SIZE]);

    // Query UEFI Boot Services for the current memory map size and related info
    let mem_map_size = bt.memory_map_size();

    // Calculate how much buffer space we actually need to hold the full memory map:
    // Add extra space for up to 8 additional MemoryDescriptors to account for updates between calls
    let needed = mem_map_size.map_size + 8 * core::mem::size_of::<uefi::table::boot::MemoryDescriptor>();

    // Create a mutable byte slice over our static buffer, allowing memory_map() to write into it
    // Unsafe because we're accessing a mutable static variable
    let buffer: &mut [u8] = unsafe { &mut MEMORY_MAP_BUFFER.0[..] };

    // Ensure our buffer is large enough; if not, panic with an error
    assert!(buffer.len() >= needed, "UEFI memory map buffer too small");

    // Call UEFI Boot Services to fill the buffer with the current memory map entries
    // This returns a MemoryMap object wrapping the raw entries
    // Panic with an error message if the call fails
    let memory_map = bt
        .memory_map(buffer)
        .expect("Failed to retrieve UEFI memory map");

    // Unsafe block to modify global mutable state safely
    unsafe {
        // Reset the counter before starting to store new regions
        REGION_COUNT = 0;

        // Iterate over each memory descriptor entry in the memory map
        for desc in memory_map.entries() {
            // Check if the type of the memory region is CONVENTIONAL,
            // which means it is general-purpose usable RAM
            if desc.ty == MemoryType::CONVENTIONAL {
                // Extract the physical start address of this memory region
                let start = desc.phys_start;

                // Extract how many 4 KiB pages this region spans
                let pages = desc.page_count;

                // Calculate the size in bytes (pages * 4096 bytes per page)
                let size = pages * 4096;

                // Check if we still have space in our static array to store this region
                if REGION_COUNT < MAX_REGIONS {
                    // Store the start address and size in the global array at the current index
                    USABLE_REGIONS[REGION_COUNT] = MemoryRegion { start, size };

                    // Increment the count of stored regions
                    REGION_COUNT += 1;
                } else {
                    // If we run out of space, optionally break early or handle overflow here
                    // For now, break the loop to avoid overwriting memory
                    break;
                }
            }
        }
    }
}

/// Returns a slice of all stored usable memory regions
pub fn get_usable_memory_regions() -> &'static [MemoryRegion] {
    unsafe {
        // Return a slice from start of an array up to REGION_COUNT
        &USABLE_REGIONS[..REGION_COUNT]
    }
}
