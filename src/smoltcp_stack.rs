#![allow(unused_imports)]
use::smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, Ipv6Address, IpEndpoint, EthernetFrame};
use::smoltcp::phy::wait as phy_wait;
use::smoltcp::phy::{Device, RxToken, RawSocket};
use::smoltcp::time::Instant;
use::smoltcp::socket::{SocketSet};
use super::smoltcp_c_interface::{Ipv4AddressC, Ipv4CidrC};
use smoltcp::socket::SocketHandle;
use std::vec::Vec;

// enum for socket type
#[derive(PartialEq, Clone)]
#[repr(C)]
pub enum SocketType {
    UDP,
    TCP,
}

// struct for smoltcp's stack
pub struct Stack<'a> {
    socket_set: SocketSet<'a>,
}

impl<'a> Stack<'a> {
    pub fn new () -> Stack<'a>{
        let socket_set = SocketSet::new(vec![]);
        Stack {
            socket_set,
        }
    }
}

#[repr(C)]
pub struct SmolSocket {
    pub(crate) socket_type: SocketType,
    pub(crate) socket_handle: SocketHandle,
}

impl SmolSocket {
    pub fn new (socket_type: SocketType,
                socket_handle: SocketHandle) -> SmolSocket {
        SmolSocket {
            socket_type,
            socket_handle,
        }
    }
}