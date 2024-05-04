pub trait Addressable {
    fn get_address_space(&self) -> (u16, u16);
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}