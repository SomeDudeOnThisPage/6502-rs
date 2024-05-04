use crate::components::memory::RandomAccessMemory;

pub struct Bus {
    devices : Vec<Box<dyn crate::components::device::Addressable>>,
}

impl Bus {
    pub fn write(&mut self, address: u16, data: u8) {
        for attached_device in &mut self.devices {
            let address_space: (u16, u16) = attached_device.get_address_space();
            if (address_space.0 <= address) && (address_space.1 > address) {
                attached_device.write(address, data);
                return;
            }
        }
        println!("NoDeviceFound@{:#06x} - TODO: ERROR", address);
    }

    pub fn read(&self, address: u16) -> u8 {
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