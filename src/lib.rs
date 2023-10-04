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
use std::os::raw::c_void;
use std::fs::File;

#[repr(C)]
pub struct CpuState {
    pub regs: [u64; 32],
    pub pc: u64,
    pub csrs: [u64; 4096],
    pub error: bool,
}

impl CpuState {
    pub fn new(cpu: &mut Cpu) -> Self {
        Self {
            regs: cpu.regs,
            pc: cpu.pc,
            csrs: cpu.csrs,
            error: cpu.error,
        }
    }
}

#[no_mangle]
pub extern fn rv64ir_init(cfile: *const c_char, cdisk: *const c_char) -> *mut c_void {
    let file_path = unsafe { CStr::from_ptr(cfile) };
    let disk_path = unsafe { CStr::from_ptr(cdisk) };

    let mut file = File::open(&file_path);
    let mut binary = Vec::new();
    file.read_to_end(&mut binary);

    let mut disk_image = Vec::new();
    let mut file = File::open(&disk_path);
    file.read_to_end(&mut disk_image);

    let mut cpu = Cpu::new(binary, disk_image);

    std::mem::transmute::<&mut Cpu, *mut c_void>(&mut cpu)
}

#[no_mangle]
pub extern fn rv64ir_cycle(cpu_ptr: *mut c_void) -> *mut c_void {
    let mut cpu = unsafe { std::mem::transmute<*mut c_void, &mut Cpu>(cpu_ptr) }

    let inst = match cpu.fetch() {
        Ok(inst) => inst,
        Err(exception) => {
            exception.take_trap(&mut cpu);
            if exception.is_fatal() {
                cpu.error = true;
            }
        }
    };

    rcpu.pc += 4;

    if !cpu.error {
        match cpu.execute(inst) {
            Ok(_) => {}
            Err(exception) => {
                exception.take_trap(&mut cpu);
                if exception.is_fatal() {
                    cpu.error = true;
                }
            }
        }
    }

    if !cpu.error {
        match cpu.check_pending_interrupt() {
            Some(interrupt) => interrupt.take_trap(&mut cpu),
            None => {}
        }
    }

    std::mem::transmute::<&mut Cpu, *mut c_void>(&mut cpu)
}

#[no_mangle]
pub extern fn rv64ir_get_state(cpu_ptr: *mut c_void) -> Box<CpuState> {
    let mut cpu = unsafe { std::mem::transmute<*mut c_void, &mut Cpu>(cpu_ptr) };
    let state = CpuState::new(&mut cpu);
    Box::new(state)
}
