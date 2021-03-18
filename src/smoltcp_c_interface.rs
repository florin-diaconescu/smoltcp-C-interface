#![allow(unused_imports)]
use super::smoltcp_stack::{SmolSocket, SocketType, Stack};
use::smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, Ipv6Address, IpEndpoint, EthernetFrame};
use::smoltcp::phy::wait as phy_wait;
use::smoltcp::phy::{Device, RxToken, RawSocket};
use::smoltcp::time::Instant;
use::smoltcp::socket::{SocketHandle};

// defining ip address structs
#[repr(C)]
pub struct Ipv4AddressC {
    pub ip_address: [u8; 4],
}

#[repr(C)]
pub struct Ipv4CidrC {
    pub ip_address: Ipv4AddressC, // ipv4 address
    pub mask: u8 // network mask
}

#[repr(C)]
pub struct Ipv6CidrC {
    pub ip_address: Ipv6AddressC, // ipv4 address
    pub mask: u8 // network mask
}


#[repr(C)]
pub struct Ipv6AddressC {
    pub ip_address: [u16; 8],
}

// abstraction for IpAddress required for passing either Ipv4 or Ipv6
#[repr(C)]
pub struct IpAddressC {
    pub ipv4_address: Ipv4AddressC,
    pub ipv6_address: Ipv6AddressC
}

// https://doc.rust-lang.org/std/convert/trait.From.html
// value-to-value-conversion while consuming the input value
impl Into<Ipv4Address> for Ipv4AddressC {
    fn into(self) -> Ipv4Address {
        Ipv4Address::new(self.ip_address[0],
                        self.ip_address[1],
                        self.ip_address[2],
                        self.ip_address[3])
    }
}

impl Into<Ipv6Address> for Ipv6AddressC {
    fn into(self) -> Ipv6Address {
        Ipv6Address::new(self.ip_address[0],
            self.ip_address[1],
            self.ip_address[2],
            self.ip_address[3],
            self.ip_address[4],
            self.ip_address[5],
            self.ip_address[6],
            self.ip_address[7])
    }
}

#[no_mangle]
pub extern "C" fn add_socket (smol_stack: &mut Stack) -> SmolSocket{
    let stack = Stack::new();
    SmolSocket {
        socket_type: SocketType::UDP,
        socket_handle: Default::default(),
    }
}