pub mod cpu6502;
pub mod bus;
mod device;
pub mod memory;

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (cpu6502::CPU6502, bus::Bus) {
        let cpu : cpu6502::CPU6502 = cpu6502::CPU6502::new();
        let memory : memory::RandomAccessMemory = memory::RandomAccessMemory::new(0x0000);
        let bus : bus::Bus = bus::Bus::new(memory);

        return (cpu, bus);
    }

    #[test]
    fn test_absolute_addressing() {
        const ADDRESS: u16 = 0x0100;

        let (mut cpu, mut bus) = setup();
        bus.write(ADDRESS, 0b00000010);
        bus.write(0x0000, 0x0E); // ASL Absolute
        bus.write(0x0001, ADDRESS.to_be_bytes()[0]);
        bus.write(0x0002, ADDRESS.to_be_bytes()[1]);
        cpu.tick(&mut bus);

        println!("{}", bus.read(ADDRESS));
        assert_eq!(0b00000100, bus.read(ADDRESS));
    }

    #[test]
    fn test_adc() {
        let (mut cpu, mut bus) = setup();
        bus.write(0x0000, 0x69); // ADC Immediate Mode
        bus.write(0x0001, 0x01); // Value '0x01'
        cpu.tick(&mut bus);
    }
}




















