// Testing... I don't want to mess with compile-time non-constant struct sizes yet...
const MEMORY_SIZE: usize = 0x7fff;

pub struct RandomAccessMemory {
    address: u16,
    data: [u8; MEMORY_SIZE],
}

impl RandomAccessMemory {
    pub fn new(address : u16) -> RandomAccessMemory {
        return RandomAccessMemory { address, data: [0; MEMORY_SIZE] };
    }
}

impl crate::components::device::Addressable for RandomAccessMemory {
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