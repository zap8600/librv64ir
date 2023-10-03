mod bus;
mod clint;
mod cpu;
mod dram;
mod plic;
mod trap;
mod uart;
mod virtio;

use crate::cpu::*;
use crate::trap::*;

#[repr(C)]
pub struct CpuState {
    pub regs: [u64; 32],
    pub pc: u64,
    pub csrs: [u64; 4096],
}

#[no_mangle]
pub extern fn rv64i_cycle() -> CpuState {
    println!("Hello from Rust");
}
