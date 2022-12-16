use organum::premade::bus::BusPort;
use std::net::{IpAddr, Ipv4Addr};

use crate::components::cpu;
use organum::core::*;
use organum::{error::Error, premade::memory, sys};
use std::vec;
mod components;

const IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const PORT: u16 = 65432;

fn main() -> Result<(), Error> {
    let mut system = sys::System::new();

    let ram = memory::MemoryBlock::new(vec![0; 1 << 20]);
    let data = memory::MemoryBlock::new(vec![0; 1 << 16]);
    let serial_block = memory::MemoryBlock::new(vec![0; 4]);

    system.add_addressable_device(0, wrap_transmutable(ram))?;
    system.add_addressable_device(0x100001, wrap_transmutable(serial_block))?;

    system.add_addressable_device_data(0, wrap_transmutable(data))?;

    let main_port = BusPort::new(0, 24, 32, system.bus.clone());
    let data_port = BusPort::new(0, 16, 32, system.bus_data.clone());

    let serial = components::tty::Serial::new(0x100001, main_port.clone());
    let mut com = components::tty::Tty::new(IP, PORT, &serial);

    com.server.run();

    system.add_device("communications-device", wrap_transmutable(com))?;
    let mut cpu = cpu::state::Sirius::new(
        components::TaleaCpuType::SiriusType,
        10_000_000,
        main_port,
        data_port,
    );

    cpu.port_d.write_beu32(0, 0x1000);

    cpu.state.psr.set_priority(1);

    cpu.add_breakpoint(8);
    system.add_interruptable_device("cpu", wrap_transmutable(cpu))?;
    system.enable_debugging();

    // Run forever
    system.run_for(u64::MAX)?;

    Ok(())
}
