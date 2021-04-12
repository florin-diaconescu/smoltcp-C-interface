#![allow(unused_imports)]
#![allow(dead_code)]
use::smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, Ipv6Address, IpEndpoint};
use smoltcp::wire::{EthernetAddress, EthernetFrame};
use::smoltcp::phy::wait as phy_wait;
use::smoltcp::phy::{Device, RxToken, RawSocket, Medium};
use::smoltcp::time::Instant;
use::smoltcp::socket::{SocketSet};
use::smoltcp::iface::{InterfaceBuilder, NeighborCache};
use super::smoltcp_c_interface::{Ipv4AddressC, Ipv4CidrC};
use smoltcp::socket::{SocketHandle, TcpSocketBuffer, TcpSocket};
use std::vec::Vec;
use std::collections::{HashMap, BTreeMap};
use nohash::{NoHashHasher, BuildNoHashHasher};
use std::hash::BuildHasherDefault;
use smoltcp::phy::{TunTapInterface, Loopback};
use std::io::Error;
use std::borrow::{Borrow, BorrowMut};
use smoltcp::iface::Interface;
use std::mem;

mod mock {
    use smoltcp::time::{Duration, Instant};
    use core::cell::Cell;

    #[derive(Debug)]
    pub struct Clock(Cell<Instant>);

    impl Clock {
        pub fn new() -> Clock {
            Clock(Cell::new(Instant::from_millis(0)))
        }

        pub fn advance(&self, duration: Duration) {
            self.0.set(self.0.get() + duration)
        }

        pub fn elapsed(&self) -> Instant {
            self.0.get()
        }
    }
}

// enum for socket type
#[derive(PartialEq, Clone)]
#[repr(C)]
pub enum SocketType {
    UDP,
    TCP,
    RAW,
}

pub enum StackType<'a, 'b> {
    Tap(Stack<'a, 'b, TunTapInterface>),
    Loopback(Stack<'a, 'b, Loopback>),
}

impl<'a, 'b> StackType<'a, 'b> {
    pub fn new_tap_stack(interface_name: &str) -> StackType<'a, 'b> {
        println!("{}", interface_name);
        let device = TunTapInterface::new(interface_name, Medium::Ethernet).unwrap();
        let stack = Stack::new(device);
        StackType::Tap(stack)
    }
    pub fn new_loopback_stack() -> StackType<'a, 'b> {
        let device = Loopback::new(Medium::Ethernet);
        let stack = Stack::new(device);
        println!("Loopback stack created!");
        StackType::Loopback(stack)
    }
}

// struct for smoltcp's stack
// https://doc.rust-lang.org/nomicon/hrtb.html
pub struct Stack<'a, 'b: 'a, DeviceT>
    where DeviceT: for<'d> Device<'d>{
    clock: mock::Clock,
    device: Option<DeviceT>,
    socket_set: SocketSet<'a>,
    current_socket_handle: u8,
    // we need a mapping between uint8_t socket handle and SocketHandle
    handle_map: HashMap::<u8, SocketHandle, BuildNoHashHasher<u8>>,
    // list for remembering ip_addresses Added
    ip_addrs: Vec<IpCidr>,
    // ethernet address
    eth_addr: EthernetAddress,
    // neighbour cache
    neigh_cache: Option<NeighborCache<'b>>,
    iface: Option<Interface<'a, DeviceT>>,
}

impl<'a, 'b, 'c, DeviceT> Stack<'a, 'b, DeviceT>
    where DeviceT: for<'d> Device<'d> {
    pub fn new (device: DeviceT) -> Stack<'a, 'b, DeviceT> {
        let socket_set = SocketSet::new(vec![]);
        let ip_addrs = Vec::new();
        Stack {
            clock: mock::Clock::new(),
            device: Some(device),
            socket_set,
            current_socket_handle: 0,
            handle_map: HashMap::with_hasher(BuildNoHashHasher::default()),
            ip_addrs,
            eth_addr: EthernetAddress([0, 0, 0, 0, 0, 0]),
            neigh_cache: None,
            iface: None,
        }
    }
    pub fn add_socket_to_stack(stack: &mut Stack<DeviceT>, smol_socket: SmolSocket) -> u8 {
        match smol_socket.socket_type {
            SocketType::TCP => {
                let rx_buffer =
                    TcpSocketBuffer::new(vec![0; smol_socket.rx_buffer]);
                let tx_buffer =
                    TcpSocketBuffer::new(vec![0; smol_socket.tx_buffer]);
                let socket = TcpSocket::new(rx_buffer, tx_buffer);
                let socket_handle = stack.socket_set.add(socket);
                stack.handle_map.insert(stack.current_socket_handle, socket_handle);
                println!("A new socket was added with handle {}!", stack.current_socket_handle);

                // Example for checking a socket state
                // let socket = stack.socket_set.get::<TcpSocket>(socket_handle);
                //
                // println!("Is active {:#?}", socket.is_active());
                // println!("Is listening {:#?}", socket.is_listening());
                // println!("State {:#?}", socket.state());
                // println!("May send {:#?}", socket.may_send());
                // println!("May recv {:#?}", socket.may_recv());
                // println!("Can recv {:#?}", socket.can_recv());

                // TODO atomic might be needed here
                stack.current_socket_handle = stack.current_socket_handle + 1;
                stack.current_socket_handle - 1
            }
            SocketType::UDP => {
                0
            }
            SocketType::RAW => {
                0
            }
        }
    }
    pub fn add_ip_address(stack: &mut Stack<DeviceT>, ip_address: IpAddress, netmask: u8) -> u8 {
        stack.ip_addrs.push(IpCidr::new(ip_address, netmask));
        println!("{:?}", stack.ip_addrs);
        0
    }

    pub fn add_ethernet_address(stack: &mut Stack<DeviceT>, eth_addr: EthernetAddress) -> u8 {
        stack.eth_addr = eth_addr;
        println!("{:?}", stack.eth_addr);
        0
    }

    pub fn build_interface(stack: &mut Stack<DeviceT>) -> u8 {
        stack.neigh_cache = Some(NeighborCache::new(BTreeMap::new()));
        let iface = InterfaceBuilder::new(stack.device.take().unwrap())
            .ethernet_addr(EthernetAddress::default())
            .neighbor_cache(stack.neigh_cache.take().unwrap())
            .ip_addrs(stack.ip_addrs.clone())
            .finalize();
        stack.iface = Some(iface);
        println!("Interface has been built!");
        println!("{:#?}", stack.handle_map);
        0
    }

    // generate a new socket handle
    pub fn get_handle(stack: &mut Stack<DeviceT>) -> u8 {
        stack.current_socket_handle = stack.current_socket_handle + 1;
        stack.current_socket_handle
    }

    pub fn poll_interface(stack: &mut Stack<DeviceT>) -> u8 {
        let iface = stack.iface.as_mut().unwrap();
        let result =
            iface.poll(stack.socket_set.borrow_mut(), stack.clock.elapsed());
        match result {
            Ok(_) => { println!("{:?}", stack.clock.elapsed()); 0 }
            Err(_) => { 1 }
        }
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