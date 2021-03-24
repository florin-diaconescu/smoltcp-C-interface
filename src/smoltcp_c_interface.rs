#![allow(unused_imports)]
use super::smoltcp_stack::{SmolSocket, SocketType, Stack};
use::smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, Ipv6Address, IpEndpoint, EthernetFrame};
use::smoltcp::phy::wait as phy_wait;
use::smoltcp::phy::{Device, RxToken, RawSocket};
use::smoltcp::time::Instant;
use::smoltcp::socket::{SocketHandle};
use smoltcp::socket::Socket;
use std::net::IpAddr;

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
// returns the socket handle
pub extern "C" fn add_socket (stack: *mut Stack, socket_type: u8) -> u8{
    let stack = unsafe {
        assert!(!stack.is_null());
        &mut *stack
    };
    let socket_type = match socket_type {
        0 => SocketType::TCP,
        1 => SocketType::UDP,
        2 => SocketType::RAW,
        _ => panic!("Socket type not supported!"),
    };
    Stack::add_socket_to_stack(stack, SmolSocket {
        socket_type,
        socket_handle: Default::default(),
        rx_buffer: 65535,
        tx_buffer: 65535,
    })
}

#[no_mangle]
pub extern "C" fn add_socket_with_buffer (stack: *mut Stack, socket_type: u8,
                                          rx_buffer: usize, tx_buffer: usize) -> u8 {
    let stack = unsafe {
        assert!(!stack.is_null());
        &mut *stack
    };
    let socket_type = match socket_type {
        0 => SocketType::TCP,
        1 => SocketType::UDP,
        _ => panic!("Socket type not supported!"),
    };
    Stack::add_socket_to_stack(stack, SmolSocket {
        socket_type,
        socket_handle: Default::default(),
        rx_buffer,
        tx_buffer,
    })
}

#[no_mangle]
pub extern "C" fn add_ipv4_address (stack: *mut Stack, a0: u8, a1: u8,
                                    a2: u8, a3: u8, netmask: u8) -> u8 {
    let stack = unsafe {
        assert!(!stack.is_null());
        &mut *stack
    };
    Stack::add_ip_address(stack, IpAddress::v4(a0, a1, a2, a3), netmask)
}

#[no_mangle]
pub extern "C" fn add_ipv6_address(stack: *mut Stack, a0: u16, a1: u16,
                                   a2: u16, a3: u16, a4: u16, a5: u16,
                                   a6: u16, a7: u16, netmask: u8) -> u8 {
    let stack = unsafe {
        assert!(!stack.is_null());
        &mut *stack
    };
    Stack::add_ip_address(stack, IpAddress::v6(a0, a1, a2, a3, a4, a5, a6, a7), netmask)
}

#[no_mangle]
pub extern "C" fn init_stack<'a>() -> *mut Stack<'a> {
    Box::into_raw(Box::new(Stack::new()))
}

#[no_mangle]
pub extern "C" fn destroy_stack<'a>(stack: *mut Stack){
    if stack.is_null() {
        return;
    }

    // unsafe, because a double free may occur
    unsafe {
        Box::from_raw(stack);
    }
}