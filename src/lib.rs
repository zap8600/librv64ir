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

use serde::{Serialize, Deserialize};

#[no_mangle]
pub extern fn rv64ir_init(cfile: *const c_char, cdisk: *const c_char) -> *const c_char {
    let file_path = unsafe { CStr::from_ptr(cfile) };
    let disk_path = unsafe { CStr::from_ptr(cdisk) };

    let mut file = File::open(&file_path);
    let mut binary = Vec::new();
    file.read_to_end(&mut binary);

    let mut disk_image = Vec::new();
    let mut file = File::open(&disk_path);
    file.read_to_end(&mut disk_image);

    let mut cpu = Cpu::new(binary, disk_image);

    let serialized = serde_json::to_string(&cpu).unwrap();
    let ccpu = CStr::new(serialized).expect("Failed to serialize Cpu struct!");
    unsafe { ccpu.as_ptr() }
}

#[no_mangle]
pub extern fn rv64ir_cycle(cstate: *const c_char) -> *const c_char {
    let ccpu = unsafe { CStr::from_ptr(cstate) };

    let mut cpu: mut Cpu = Point = serde_json::from_str(&ccpu).unwrap();

    let inst = match cpu.fetch() {
        Ok(inst) => inst,
        Err(exception) => {
            exception.take_trap(&mut cpu);
            if exception.is_fatal() {
                cpu.error = true;
            }
        }
    };

    cpu.pc += 4;

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

    cpu.pc -= 4;

    let serialized = serde_json::to_string(&cpu).unwrap();
    let ccpu = CStr::new(serialized).expect("Failed to serialize Cpu struct!");

    cpu.pc += 4;

    unsafe { ccpu.as_ptr() }
}
