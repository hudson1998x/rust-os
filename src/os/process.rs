/// Represents the current execution state of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// The process has been created but not yet admitted to the ready queue.
    New,

    /// The process is ready to be scheduled on the CPU but is not currently executing.
    Ready,

    /// The process is actively running on a CPU core.
    Running,

    /// The process is waiting for some event to complete (e.g., I/O, timer, semaphore).
    Blocked,

    /// The process has been suspended (paused) by the system or user.
    Suspended,

    /// The process has finished execution and is awaiting cleanup or parent acknowledgment.
    Terminated,
}


/// A Process Control Block (PCB) that tracks all kernel-managed state for a user or kernel process.
/// Each `Process` is a complete, schedulable execution unit tracked by the kernel scheduler.
#[derive(Debug)]
pub struct Process {
    // =========================================================================
    // Process Identification
    // =========================================================================

    /// The unique identifier for this process (PID).
    /// Typically assigned sequentially or reused after termination.
    pub pid: u64,

    /// The PID of the parent process that created this process via fork/clone.
    /// Used for signaling, hierarchy tracking, and reparenting on exit.
    pub ppid: u64,

    // =========================================================================
    // Metadata
    // =========================================================================

    /// Fixed-length ASCII name for the process (e.g., "init", "shell", "myprog").
    /// Null-padded if shorter than 32 bytes. Not necessarily null-terminated.
    pub name: [u8; 32],

    // =========================================================================
    // State and Scheduling
    // =========================================================================

    /// Current scheduling state of the process (Ready, Running, Blocked, etc.).
    pub state: ProcessState,

    /// Scheduling priority (0 = highest priority). Used by priority schedulers.
    pub priority: u8,

    /// Time slice allocated to the process by the scheduler (in ticks or ms).
    /// Reset on each schedule to manage fairness and preemption.
    pub timeslice: u32,

    /// Exit code returned by the process on termination (if any).
    /// Set when the process finishes, useful for waitpid() or diagnostics.
    pub exit_code: Option<i32>,

    // =========================================================================
    // Memory Layout (Virtual Address Space)
    // =========================================================================

    /// Virtual base address of the code (text) segment.
    /// Typically read-only and executable. May be shared between processes.
    pub code_base: usize,

    /// Maximum size of the code segment in bytes.
    pub code_size: usize,

    /// Virtual base address of the data segment (initialized globals).
    /// Typically read-write. Allocated after the code segment.
    pub data_base: usize,

    /// Maximum size of the data segment in bytes.
    pub data_size: usize,

    /// Virtual base address of the heap segment (malloc, dynamic memory).
    /// Grows upward as memory is allocated.
    pub heap_base: usize,

    /// Maximum size of the heap segment in bytes.
    /// May grow at runtime via sbrk/heap allocator logic.
    pub heap_size: usize,

    /// Virtual base address of the stack segment.
    /// Usually grows downward from this address.
    pub stack_base: usize,

    /// Maximum stack size in bytes. Enforced by guard pages or memory maps.
    pub stack_size: usize,

    // =========================================================================
    // Memory Management (Paging)
    // =========================================================================

    /// Physical address of the root page table.
    /// For x86_64 this would be the address of the PML4.
    /// Used when switching to this process’s address space.
    pub page_table_root: usize,

    // =========================================================================
    // CPU Context (for context switching)
    // =========================================================================

    /// General-purpose and system registers saved during a context switch.
    /// This includes all CPU state not automatically saved by hardware on trap/interrupt.
    pub regs: [u64; 32],

    /// Instruction pointer (PC or RIP). Set to resume execution after scheduling.
    pub pc: usize,

    /// Current stack pointer value (SP or RSP). Points to the top of user stack.
    pub sp: usize,

    /// Saved flags register (EFLAGS/RFLAGS). Captures CPU status (interrupts, zero/carry, etc.).
    pub flags: u64,

    // =========================================================================
    // Scheduling and Blocking
    // =========================================================================

    /// If `Some`, the resource or entity this process is blocked/waiting on.
    /// For example: another PID, an I/O device, or a semaphore.
    pub waiting_on: Option<WaitTarget>,

    /// If `Some`, indicates the timestamp when this sleeping process should wake up.
    /// Used by timer-based wait mechanisms (e.g., `sleep()`).
    pub wakeup_time: Option<u64>,

    // =========================================================================
    // Interprocess Communication / File System
    // =========================================================================

    /// Simulated file descriptor table: up to 64 open files/pipes/devices per process.
    /// Each entry contains a file descriptor index or ID. `None` means unused slot.
    pub file_descriptors: [Option<u32>; 64],

    // =========================================================================
    // Signals (UNIX-like)
    // =========================================================================

    /// Bitmap representing pending signals.
    /// Bit `n` is set if signal `n` is pending and has not yet been handled.
    pub signal_bitmap: u64,

    /// Array of function pointers or virtual addresses of user-defined signal handlers.
    /// If `signal_handlers[n]` is non-zero, it's the handler for signal `n`.
    pub signal_handlers: [usize; 32],

    // =========================================================================
    // Time Accounting
    // =========================================================================

    /// Kernel tick count when the process was created.
    /// Used for diagnostics, aging, and lifetime metrics.
    pub created_at: u64,

    /// Total CPU time consumed by this process (in ticks).
    /// Updated on every deschedule or preemption.
    pub cpu_time: u64,

    /// Timestamp of the last time this process was scheduled to run.
    /// Useful for scheduling policies and profiling.
    pub last_scheduled: u64,

    // =========================================================================
    // Miscellaneous / Kernel State
    // =========================================================================

    /// Virtual base address of this process’s kernel stack (for syscall, interrupts).
    /// Used during privilege transitions and stored in TSS or equivalent structure.
    pub kernel_stack: usize,
}

/// Enum representing entities that a process may be blocked waiting for.
/// Used by the scheduler and blocking primitives to resume the process.
#[derive(Debug, Clone, Copy)]
pub enum WaitTarget {
    /// Waiting for a specific process to terminate or change state (e.g., waitpid).
    PID(u64),

    /// Waiting on an I/O device (e.g., disk, terminal, network card).
    IODevice(u32),

    /// Waiting for a timer or timeout to expire (e.g., sleep()).
    Timer,

    /// Waiting on one named or indexed semaphore to become available.
    Semaphore(u32),

    /// Waiting on a message to arrive in a queue or IPC channel.
    MessageQueue(u32),
}
