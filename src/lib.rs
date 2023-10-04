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

use std::ffi::CStr;
use std::os::raw::c_char;
use std::fs::File;

let mut rcpu;

#[repr(C)]
pub struct CpuState {
    pub regs: [u64; 32],
    pub pc: u64,
    pub csrs: [u64; 4096],
    pub error: bool,
}

impl CpuState {
    pub fn new(cpu: &mut Cpu, is_error: bool) -> Self {
        Self {
            regs: cpu.regs,
            pc: cpu.pc,
            csrs: cpu.csrs,
            error: is_error,
        }
    }
}

#[no_mangle]
pub extern fn rv64ir_init(cfile: *const c_char, cdisk: *const c_char) {
    let file_path = unsafe { CStr::from_ptr(cfile) };
    let disk_path = unsafe { CStr::from_ptr(cdisk) };

    let mut file = File::open(&file_path);
    let mut binary = Vec::new();
    file.read_to_end(&mut binary);

    let mut disk_image = Vec::new();
    let mut file = File::open(&disk_path);
    file.read_to_end(&mut disk_image);

    rcpu = Cpu::new(binary, disk_image);
}

#[no_mangle]
pub extern fn rv64ir_cycle() -> Box<CpuState> {
    let mut has_error;

    let inst = match rcpu.fetch() {
        Ok(inst) => inst,
        Err(exception) => {
            exception.take_trap(&mut rcpu);
            if exception.is_fatal() {
                has_error = true;
            }
        }
    };

    rcpu.pc += 4;

    if !has_error {
        match rcpu.execute(inst) {
            Ok(_) => {}
            Err(exception) => {
                exception.take_trap(&mut rcpu);
                if exception.is_fatal() {
                    has_error = true;
                }
            }
        }
    }

    if !has_error {
        match rcpu.check_pending_interrupt() {
            Some(interrupt) => interrupt.take_trap(&mut rcpu),
            None => {}
        }
    }

    let cpu_state = CpuState::new(
        &rcpu,
        if has_error {
            true
        } else {
            false
        },
    );

    Box::new(cpu_state)
}
