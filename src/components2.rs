use crate::components::memory::RandomAccessMemory;

trait Addressable {
    fn get_address_space(&self) -> (u16, u16);
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

pub struct Bus {
    devices : Vec<Box<dyn Addressable>>,
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

pub mod memory {
    use crate::components::Addressable;

    const MEMORY_SIZE : usize = 0x7fff;

    pub struct RandomAccessMemory {
        address : u16,
        data : [u8; MEMORY_SIZE],
    }

    impl RandomAccessMemory {
        pub fn new(address : u16) -> RandomAccessMemory {
            return RandomAccessMemory { address, data : [0; MEMORY_SIZE] };
        }
    }

    impl Addressable for RandomAccessMemory {
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

    pub fn new(memory : RandomAccessMemory) -> Bus {
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
