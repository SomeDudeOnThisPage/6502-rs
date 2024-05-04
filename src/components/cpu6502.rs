macro_rules! dbg {
    ($($x:tt)*) => {
        {
            #[cfg(debug_assertions)]
            {
                std::println!($($x)*)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($x)*)
            }
        }
    }
}

use crate::components::bus::Bus;

pub enum Flags {
    Carry = 0b00000001,
    Zero = 0b00000010,
    InterruptDisable = 0b00000100,
    DecimalMode = 0b00001000,
    BreakCommand = 0b00010000,
    Overflow = 0b00100000,
    Negative = 0b01000000,
}

#[derive(Debug, Copy, Clone)]
pub enum IAMSubMode {
    N, X, Y
}

// For easier writing -> Refactor to InstructionAddressingMode later...
#[derive(Debug, Copy, Clone)]
pub enum IAM {
    Accumulator,
    Immediate,
    ZeroPage(IAMSubMode),
    Absolute(IAMSubMode),
    Indirect(IAMSubMode),
    Relative,
    Implied,
}

#[derive(Debug, Copy, Clone)]
pub struct OperationCode {
    instruction : Instruction,
    mode: IAM,
    bytes: u8,
    cycles: u8,
}

#[derive(Debug)]
pub struct Registers {
    program_counter: u16,
    stack_pointer: u8,
    accumulator: u8,
    idx_x: u8,
    idx_y: u8,
    status_flags: u8,
}

#[derive(Debug)]
pub struct CPU6502 {
    registers: Registers,
    instructions: [Option<OperationCode>; 0xFF],
}

impl Registers {
    pub fn new() -> Registers {
       return Registers {
           program_counter: 0,
           stack_pointer: 0,
           accumulator: 0,
           idx_x: 0,
           idx_y: 0,
           status_flags: 0,
       } 
    }

    pub fn get_flag(&mut self, flag: Flags) -> bool {
        return (self.status_flags & flag as u8) != 0;
    }

    pub fn set_flag(&mut self, flag: Flags, set: bool) {
        match set {
            true => { self.status_flags |= flag as u8 }
            false => { self.status_flags &= flag as u8 }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
}

enum Address {
    A,
    M(u16)
}

impl CPU6502 {
    // return: absolute address, addressed value, additional cycles needed
    fn fetch(&mut self, opcode: &OperationCode, bus: &mut Bus) -> (Address, u8, u8) {
        fn add(mode: IAMSubMode, idx_x: u8, idx_y: u8) -> u16 {
            return match mode {
                IAMSubMode::N => {0x00}
                IAMSubMode::X => {idx_x as u16}
                IAMSubMode::Y => {idx_y as u16}
            };
        }

        return match opcode.mode {
            IAM::Accumulator => {
                return (Address::A, self.registers.accumulator, 0x00);
            },
            IAM::Immediate => {
                let address: u16 = self.next(bus) as u16;
                return (Address::M(address), bus.read(address), 0x00);
            },
            IAM::ZeroPage(sub_mode) => {
                let address: u16 = (self.next(bus) as u16) + add(sub_mode, self.registers.idx_x, self.registers.idx_y);
                return (Address::M(address), bus.read(address), 0x00);
            },
            IAM::Absolute(sub_mode) => {
                let address: u16 = (self.next(bus) as u16) << 8 + (self.next(bus) as u16) + add(sub_mode, self.registers.idx_x, self.registers.idx_y);
                return (Address::M(address), bus.read(address), 0x00);
            },
            IAM::Indirect(_sub_mode) => {
                panic!("// TODO");
            },
            IAM::Relative => {
                panic!("// TODO");
            },
            IAM::Implied => {
                panic!("// TODO");
            },
        };
    }

    fn adc(&mut self, opcode: OperationCode, bus: &mut Bus) {
        let accumulator: u8 = self.registers.accumulator;
        let (_, addressed, _additional_cycles) = self.fetch(&opcode, bus);
        let result: u16 = (accumulator as u16) + (addressed as u16) + ((self.registers.status_flags & 0x01) as u16);

        let [overflow, result]: [u8; 2] = result.to_be_bytes();

        self.registers.set_flag(Flags::Carry, overflow >= 0x01);
        self.registers.set_flag(Flags::Overflow, (accumulator ^ result) & !(accumulator ^ result) == 0x01);
        self.registers.set_flag(Flags::Zero, result == 0x00);
        self.registers.set_flag(Flags::Negative, result & 0b10000000 != 0);

        self.registers.accumulator = result;

        dbg!("ADC\t{:#04x}+{:#04x}={:#04x}\tA={:#04x}\tFlags={:#010b}", accumulator, addressed, result, self.registers.accumulator, self.registers.status_flags);
    }

    fn and(&mut self, opcode: OperationCode, bus: &mut Bus) {
        let accumulator = self.registers.accumulator;
        let (_, addressed, _additional_cycles) = self.fetch(&opcode, bus);
        let result = accumulator & addressed;

        self.registers.set_flag(Flags::Zero, result == 0x00);
        self.registers.set_flag(Flags::Negative, result & 0b10000000 != 0);

        self.registers.accumulator = result;

        dbg!("AND\t{:#04x}&{:#04x}={:#04x}\tA={:#04x}\tFlags={:#010b}", accumulator, addressed, result, self.registers.accumulator, self.registers.status_flags);
    }

    fn asl(&mut self, opcode: OperationCode, bus: &mut Bus) {
        let (address, addressed, _additional_cycles) = self.fetch(&opcode, bus);
        let carry = (addressed & 0b10000000) >> 7 == 0x01;
        match address {
            Address::A => {
                self.registers.accumulator = addressed << 1;
                dbg!("ASL\t{:#04x}<<0x01={:#04x}\tFlags={:#010b}", addressed, self.registers.accumulator, self.registers.status_flags);
            }
            Address::M(address) => {
                bus.write(address, addressed << 1);
                dbg!("ASL\t{:#04x}<<0x01={:#04x}\tFlags={:#010b}", addressed, bus.read(address), self.registers.status_flags);
            }
        }

        self.registers.set_flag(Flags::Carry, carry);
    }

    fn branch(&mut self, bus: &Bus) {
        self.registers.program_counter as i16 += self.next(bus) as i16;
    }

    fn next(&mut self, bus: &Bus) -> u8 {
        self.registers.program_counter += 1;
        return bus.read(self.registers.program_counter);
    }

    fn execute(&mut self, bus: &mut Bus, opcode: OperationCode) {
        match opcode.instruction {
            Instruction::ADC => { self.adc(opcode, bus) }
            Instruction::AND => { self.and(opcode, bus) }
            Instruction::ASL => { self.asl(opcode, bus) }
            Instruction::BCC => { if !self.registers.get_flag(Flags::Carry) { self.branch(bus) } }
            Instruction::BCS => { if self.registers.get_flag(Flags::Carry) { self.branch(bus) } }
            Instruction::BEQ => { if self.registers.get_flag(Flags::Zero) { self.branch(bus) } }
            Instruction::BIT => { /* TODO... */ }
            Instruction::BMI => { if self.registers.get_flag(Flags::Negative) { self.branch(bus) } }
            Instruction::BNE => { if !self.registers.get_flag(Flags::Zero) { self.branch(bus) } }
            Instruction::BPL => { if !self.registers.get_flag(Flags::Negative) { self.branch(bus) } }
            Instruction::BRK => { /* TODO... */ }
            Instruction::BVC => { if !self.registers.get_flag(Flags::Overflow) { self.branch(bus) } }
            Instruction::BVS => { if self.registers.get_flag(Flags::Overflow) { self.branch(bus) } }
            Instruction::CLC => { self.registers.set_flag(Flags::Carry, false) }
            Instruction::CLD => { self.registers.set_flag(Flags::DecimalMode, false) }
            Instruction::CLI => { self.registers.set_flag(Flags::InterruptDisable, false) }
            Instruction::CLV => { self.registers.set_flag(Flags::Overflow, false) }
            Instruction::CMP => {}
        }
    }

    pub fn dump_registers(&self) {
        println!("{:?}", self.registers);
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        let byte: u8 = bus.read(self.registers.program_counter);

        match self.instructions[byte as usize] {
            Some(opcode) => {
                self.execute(bus, opcode);
                self.next(bus);
            }
            None => {
                panic!("instruction {:#06x}@{:#06x} does not exist", byte, self.registers.program_counter);
            }
        };
    }

    pub fn new() -> CPU6502 {
        let mut cpu: CPU6502 = CPU6502 {
            registers: Registers::new(),
            instructions: [None; 0xFF],
        };

        // Add With Carry (ADC)
        {
            // ADC Immediate
            cpu.instructions[0x69] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::Immediate,
                bytes: 2,
                cycles: 2
            });

            // ADC ZeroPage
            cpu.instructions[0x65] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::ZeroPage(IAMSubMode::N),
                bytes: 2,
                cycles: 3
            });

            // ADC ZeroPageX
            cpu.instructions[0x75] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::ZeroPage(IAMSubMode::X),
                bytes: 2,
                cycles: 4
            });

            // ADC Absolute
            cpu.instructions[0x6D] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::Absolute(IAMSubMode::N),
                bytes: 3,
                cycles: 4
            });

            // ADC AbsoluteX
            cpu.instructions[0x7D] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::Absolute(IAMSubMode::X),
                bytes: 3,
                cycles: 4
            });

            // ADC AbsoluteY
            cpu.instructions[0x79] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::Absolute(IAMSubMode::Y),
                bytes: 3,
                cycles: 4
            });

            // ADC IndirectX
            cpu.instructions[0x61] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::Indirect(IAMSubMode::X),
                bytes: 2,
                cycles: 6
            });

            // ADC AbsoluteY
            cpu.instructions[0x71] = Some(OperationCode {
                instruction : Instruction::ADC,
                mode: IAM::Indirect(IAMSubMode::Y),
                bytes: 2,
                cycles: 5
            });
        }

        // Logical And (AND)
        {
            cpu.instructions[0x29] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::Immediate,
                bytes: 2,
                cycles: 2,
            });

            cpu.instructions[0x25] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::ZeroPage(IAMSubMode::N),
                bytes: 2,
                cycles: 3,
            });

            cpu.instructions[0x35] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::ZeroPage(IAMSubMode::X),
                bytes: 2,
                cycles: 4,
            });

            cpu.instructions[0x2D] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::Absolute(IAMSubMode::N),
                bytes: 3,
                cycles: 4,
            });

            cpu.instructions[0x3D] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::Absolute(IAMSubMode::X),
                bytes: 3,
                cycles: 4,
            });

            cpu.instructions[0x39] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::Absolute(IAMSubMode::Y),
                bytes: 3,
                cycles: 4,
            });

            cpu.instructions[0x21] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::Indirect(IAMSubMode::X),
                bytes: 2,
                cycles: 6,
            });

            cpu.instructions[0x31] = Some(OperationCode {
                instruction : Instruction::AND,
                mode: IAM::Indirect(IAMSubMode::Y),
                bytes: 2,
                cycles: 5,
            });
        }

        // Arithmetic Shift Left (ASL)
        {
            cpu.instructions[0x0A] = Some(OperationCode {
                instruction : Instruction::ASL,
                mode: IAM::Accumulator,
                bytes: 1,
                cycles: 2,
            });

            cpu.instructions[0x06] = Some(OperationCode {
                instruction : Instruction::ASL,
                mode: IAM::ZeroPage(IAMSubMode::N),
                bytes: 2,
                cycles: 5,
            });

            cpu.instructions[0x16] = Some(OperationCode {
                instruction : Instruction::ASL,
                mode: IAM::ZeroPage(IAMSubMode::X),
                bytes: 2,
                cycles: 6,
            });

            cpu.instructions[0x0E] = Some(OperationCode {
                instruction : Instruction::ASL,
                mode: IAM::Absolute(IAMSubMode::N),
                bytes: 3,
                cycles: 6,
            });

            cpu.instructions[0x1E] = Some(OperationCode {
                instruction : Instruction::ASL,
                mode: IAM::Absolute(IAMSubMode::X),
                bytes: 3,
                cycles: 7,
            });
        }

        // Branch if Carry Clear (BCC)
        {
            cpu.instructions[0x90] = Some(OperationCode {
                instruction : Instruction::BCC,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Branch if Carry Set (BCS)
        {
            cpu.instructions[0xB0] = Some(OperationCode {
                instruction : Instruction::BCS,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Branch if Equal (BEQ)
        {
            cpu.instructions[0xF0] = Some(OperationCode {
                instruction : Instruction::BEQ,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Bit Test (BIT)
        {
            cpu.instructions[0x24] = Some(OperationCode {
                instruction : Instruction::BIT,
                mode: IAM::ZeroPage(IAMSubMode::N),
                bytes: 2,
                cycles: 3,
            });

            cpu.instructions[0x2C] = Some(OperationCode {
                instruction : Instruction::BIT,
                mode: IAM::Absolute(IAMSubMode::Y),
                bytes: 3,
                cycles: 4,
            });
        }

        // Branch if Minus (BMI)
        {
            cpu.instructions[0x30] = Some(OperationCode {
                instruction : Instruction::BMI,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Branch if Not Equal (BNE)
        {
            cpu.instructions[0xD0] = Some(OperationCode {
                instruction : Instruction::BNE,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Branch if Positive (BPL)
        {
            cpu.instructions[0x10] = Some(OperationCode {
                instruction : Instruction::BPL,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Force Interrupt (BRK)
        {
            cpu.instructions[0x00] = Some(OperationCode {
                instruction : Instruction::BRK,
                mode: IAM::Implied,
                bytes: 1,
                cycles: 7,
            });
        }

        // Branch if Overflow Clear (BVC)
        {
            cpu.instructions[0x50] = Some(OperationCode {
                instruction : Instruction::BVC,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Branch if Overflow Set (BVS)
        {
            cpu.instructions[0x70] = Some(OperationCode {
                instruction : Instruction::BVS,
                mode: IAM::Relative,
                bytes: 2,
                cycles: 2,
            });
        }

        // Clear Carry Flag (CLC)
        {
            cpu.instructions[0x18] = Some(OperationCode {
                instruction : Instruction::CLC,
                mode: IAM::Implied,
                bytes: 1,
                cycles: 2,
            });
        }

        // Clear Decimal Mode (CLD)
        {
            cpu.instructions[0xD8] = Some(OperationCode {
                instruction : Instruction::CLD,
                mode: IAM::Implied,
                bytes: 1,
                cycles: 2,
            });
        }

        // Clear Interrupt Disable (CLI)
        {
            cpu.instructions[0x58] = Some(OperationCode {
                instruction : Instruction::CLI,
                mode: IAM::Implied,
                bytes: 1,
                cycles: 2,
            });
        }

        // Clear Overflow Flag (CLV)
        {
            cpu.instructions[0xB8] = Some(OperationCode {
                instruction : Instruction::CLV,
                mode: IAM::Implied,
                bytes: 1,
                cycles: 2,
            });
        }

        // Compare (CMP)
        {
            cpu.instructions[0xC9] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::Immediate,
                bytes: 2,
                cycles: 2,
            });

            cpu.instructions[0xC5] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::ZeroPage(IAMSubMode::N),
                bytes: 2,
                cycles: 3,
            });

            cpu.instructions[0xD5] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::ZeroPage(IAMSubMode::X),
                bytes: 2,
                cycles: 4,
            });

            cpu.instructions[0xCD] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::Absolute(IAMSubMode::N),
                bytes: 3,
                cycles: 4,
            });

            cpu.instructions[0xDD] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::Absolute(IAMSubMode::X),
                bytes: 3,
                cycles: 4,
            });

            cpu.instructions[0xD9] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::Absolute(IAMSubMode::Y),
                bytes: 3,
                cycles: 4,
            });

            cpu.instructions[0xC1] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::Indirect(IAMSubMode::X),
                bytes: 2,
                cycles: 6,
            });

            cpu.instructions[0xD1] = Some(OperationCode {
                instruction : Instruction::CMP,
                mode: IAM::Indirect(IAMSubMode::Y),
                bytes: 2,
                cycles: 5,
            });
        }

        return cpu;
    }
}
