#![allow(warnings)]

mod assembler;

mod components {
    use std::ptr::null;

    const MEMORY_SIZE : usize = 0x7fff;

    mod instructions {
        use crate::components::{Bus, ScottCPU};

        trait Instruction {
            //fn execute(cpu: &ScottCPU, bus: &Bus, args : &[u8]);
        }

        struct SPU {}

        impl Instruction for SPU {
            //fn execute(cpu: &mut ScottCPU, bus: &Bus, args : &[u8]) {
                //bus.write(cpu.stack_top += 1, args[0]);
                //bus.write(cpu.stack_top += 1, args[1]);
            //}
        }
    }

    trait Addressable {
        fn get_address_space(&self) -> (u16, u16);
        fn read(&self, address: u16) -> u8;
        fn write(&mut self, address: u16, data: u8);
    }

    pub struct Memory {
        address : u16,
        data : [u8; MEMORY_SIZE],
    }

    pub struct Bus {
        devices : Vec<Box<dyn Addressable>>,
    }

    impl Bus {

        pub fn write(&mut self, address: u16, data: u8) {
            for mut attached_device in &mut self.devices {
                let address_space: (u16, u16) = attached_device.get_address_space();
                if (address_space.0 <= address) && (address_space.1 > address) {
                    attached_device.write(address, data);
                    return;
                }
            }
            println!("NoDeviceFound@{:#06x} - TODO: ERROR", address);
        }

        pub fn read(&self, address: u16) -> (u8) {
            let mut device= &self.devices[0];

            for attached_device in &self.devices {
                let address_space: (u16, u16) = attached_device.get_address_space();
                if (address_space.0 <= address) && (address_space.1 > address) {
                    device = attached_device;
                    break;
                }
            }

            return device.read(address);
        }

        pub fn new(memory : Memory) -> Bus {
            return Bus { devices: vec![Box::new(memory)] };
        }
    }

    pub const SPU8 : u8 = 0x01;
    pub const SPO8 : u8 = 0x02;
    pub const ADD8 : u8 = 0x03;
    pub const SUB8 : u8 = 0x04;
    pub const NOT : u8 = 0x05;
    pub const HALT : u8 = 0xFF;



    #[derive(Debug)]
    enum Instruction {
        SPU8 { data: u8 },
        SPO8,
        ADD8,
        SUB8,
        NOT,
        HALT,
        NOP,
    }

    pub struct ScottCPU {
        caez: u8,
        halted: bool,
        stack_base: u16,
        stack_top: u16,
        code_base: u16,
        code_base_top: u16,
        code_pointer: u16,
    }

    impl ScottCPU {
        pub fn new(code_store_base : u16, code_store_size : u16) -> ScottCPU {
            return ScottCPU {
                caez: 0u8,
                halted: false,
                stack_base: 0x0100,
                stack_top: 0x0100,
                code_base: code_store_base,
                code_base_top: code_store_base + code_store_size,
                code_pointer: code_store_base,
            }
        }

        fn make_instruction(&mut self, bus : &Bus, opcode: u8) -> (Instruction, u16) {
            match opcode {
                SPU8 => return (Instruction::SPU8 { data: bus.read(self.code_pointer + 1)}, 2),
                SPO8 => return (Instruction::SPO8, 1),
                ADD8 => return (Instruction::ADD8, 1),
                SUB8 => return (Instruction::SUB8, 1),
                NOT => return (Instruction::NOT, 1),
                HALT => return (Instruction::HALT, 1),
                _ => (Instruction::NOP, 1),
            }
        }

        fn pop(&mut self, bus: &mut Bus) -> u8 {
            self.stack_top -= 1;
            let value = bus.read(self.stack_top);
            bus.write(self.stack_top, 0);
            return value;
        }

        fn push(&mut self, bus: &mut Bus, data: u8) {
            bus.write(self.stack_top, data);
            self.stack_top += 1;
        }

        fn execute_instruction(&mut self, bus: &mut Bus, instruction: &Instruction) {
            match instruction {
                Instruction::SPU8 { data } => {
                    self.push(bus, *data);
                },
                Instruction::SPO8 => {
                    self.pop(bus);
                },
                Instruction::ADD8 => {
                    let x = self.pop(bus);
                    let y = self.pop(bus);
                    // todo: carry
                    println!("add: {} + {} = {}", x, y, x + y);
                    self.push(bus, x + y);
                },
                Instruction::NOT => {
                    bus.write(self.stack_top, bus.read(self.stack_top) ^ 0b11111111);
                },
                Instruction::SUB8 => {
                    let y = self.pop(bus);
                    let x = self.pop(bus);
                    // todo: carry, over/underflow
                    println!("sub: {} + {} = {}", x, y, x - y);
                    self.push(bus, x - y);
                },
                Instruction::HALT => {
                    self.halted = true;
                },
                Instruction::NOP => {},
            }
        }

        pub fn is_halted(&self) -> bool {
            return self.halted;
        }

        pub fn tick(&mut self, bus: &mut Bus) {
            // fetch opcode
            let opcode = bus.read(self.code_pointer);
            println!("opcode@{:#06x}: {:#010b}", self.code_pointer, opcode);

            // match opcode to instruction
            let (instruction, size) = self.make_instruction(bus, opcode);
            println!("instruction@{:#06x} {:?}", self.code_pointer, instruction);

            // execute instruction
            self.execute_instruction(bus, &instruction);

            // increment code pointer
            self.code_pointer += size;
        }
    }

    impl Memory {
        pub fn new(address : u16) -> Memory {
            return Memory { address, data : [0; MEMORY_SIZE] };
        }
    }

    impl Addressable for Memory {
        fn get_address_space(&self) -> (u16, u16) {
            return (self.address, 0x7fff);
        }

        fn read(&self, address: u16) -> u8 {
            return self.data[address as usize];
        }

        fn write(&mut self, address: u16, data: u8) {
            self.data[address as usize] = data;
        }
    }
}

fn main() {
    let mut cpu = components::ScottCPU::new(0x0800, 0x0100);
    let memory = components::Memory::new(0x0010);
    let mut bus = components::Bus::new(memory);

    // manually write program as bytecode for now - no assembler
    bus.write(0x0800, components::SPU8);
    bus.write(0x0801, 0x01);
    bus.write(0x0802, components::SPU8);
    bus.write(0x0803, 0x01);
    bus.write(0x0804, components::ADD8); // = 2
    bus.write(0x0805, components::SPU8);
    bus.write(0x0806, 0x01);
    bus.write(0x0807, components::SUB8); // = 1
    bus.write(0x0808, components::HALT);

    for _ in 0..255 {
        if !cpu.is_halted() {
            cpu.tick(&mut bus);
        } else {
            println!("\n=== CPU halted ===");
            println!("data@{:#06x}: {:#010b}", 0x0100, bus.read(0x0100));
            println!("data@{:#06x}: {:#010b}", 0x0101, bus.read(0x0101));
            break;
        }
    }
}
