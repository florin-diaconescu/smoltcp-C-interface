#![allow(unused_imports)]
use::smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, Ipv6Address, IpEndpoint, EthernetFrame};
use::smoltcp::phy::wait as phy_wait;
use::smoltcp::phy::{Device, RxToken, RawSocket};
use::smoltcp::time::Instant;
use::smoltcp::socket::{SocketSet};
use super::smoltcp_c_interface::{Ipv4AddressC, Ipv4CidrC};
use smoltcp::socket::{SocketHandle, TcpSocketBuffer, TcpSocket};
use std::vec::Vec;
use std::collections::HashMap;
use nohash::{NoHashHasher, BuildNoHashHasher};
use std::hash::BuildHasherDefault;

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
    current_socket_handle: u8,
    // we need a mapping between uint8_t socket handle and SocketHandle
    handle_map: HashMap::<u8, SocketHandle, BuildNoHashHasher<u8>>,
}

impl<'a> Stack<'a> {
    pub fn new () -> Stack<'a> {
        let socket_set = SocketSet::new(vec![]);
        Stack {
            socket_set,
            current_socket_handle: 0,
            handle_map: HashMap::with_hasher(BuildNoHashHasher::default()),
        }
    }
    pub fn add_socket_to_stack(stack: &mut Stack, smol_socket: SmolSocket) -> u8 {
        match smol_socket.socket_type {
            SocketType::TCP => {
                let rx_buffer =
                    TcpSocketBuffer::new(vec![0; smol_socket.rx_buffer]);
                let tx_buffer =
                    TcpSocketBuffer::new(vec![0; smol_socket.tx_buffer]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let socket_handle = stack.socket_set.add(socket);
                stack.handle_map.insert(stack.current_socket_handle, socket_handle);
                0
            }
            SocketType::UDP => {
                0
            }
        }
    }

    // generate a new socket handle
    pub fn get_handle(stack: &mut Stack) -> u8 {
        stack.current_socket_handle = stack.current_socket_handle + 1;
        stack.current_socket_handle
    }
}

pub struct SmolSocket {
    pub(crate) socket_type: SocketType,
    pub(crate) socket_handle: u8,
    pub rx_buffer: usize,
    pub tx_buffer: usize,
}

impl SmolSocket {
    pub fn new (socket_type: SocketType,
                socket_handle: u8,
                rx_buffer: usize,
                tx_buffer: usize) -> SmolSocket {
        SmolSocket {
            socket_type,
            socket_handle,
            rx_buffer,
            tx_buffer,
        }
    }
}