//#![allow(warnings)]

mod components;
mod assembler;

use crate::components::bus::Bus;
use crate::components::cpu6502::CPU6502;
use crate::components::memory::RandomAccessMemory;

fn main() {
    let mut cpu : CPU6502 = CPU6502::new();
    let memory : RandomAccessMemory = RandomAccessMemory::new(0x0000);
    let mut bus : Bus = Bus::new(memory);

    bus.write(0x00FF, 0x01); // Write '0x01' to memory location '0x0100'

    /*bus.write(0x0000, 0x69); // ADC Immediate Mode
    bus.write(0x0001, 0x01); // Value '0x01'
    bus.write(0x0002, 0x29); // AND Immediate Mode
    bus.write(0x0003, 0xFF); // Value '0x01'*/
    bus.write(0x0004, 0x0E); // ASL Absolute
    bus.write(0x0005, 0xFF); // Address '0xFF'

    for _ in 0..10 {
        cpu.tick(&mut bus);
    }

    cpu.dump_registers();
    println!("Memory@{:#06x}={:#06x}", 0x00FF, bus.read(0x00FF));
}
