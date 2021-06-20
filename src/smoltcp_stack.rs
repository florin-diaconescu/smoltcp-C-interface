#![allow(unused_imports)]
#![allow(dead_code)]
use::smoltcp::wire::{IpAddress, IpCidr, Ipv4Address, Ipv4Cidr, Ipv6Address, IpEndpoint};
use smoltcp::wire::{EthernetAddress, EthernetFrame, UdpPacket};
use::smoltcp::phy::wait as phy_wait;
use::smoltcp::phy::{Device, RxToken, RawSocket, Medium};
use::smoltcp::time::Instant;
use::smoltcp::socket::{SocketSet};
use::smoltcp::iface::{InterfaceBuilder, NeighborCache};
use super::smoltcp_c_interface::{Ipv4AddressC, Ipv4CidrC};
use smoltcp::socket::{SocketHandle, TcpSocketBuffer, TcpSocket, UdpSocketBuffer, UdpSocket, UdpPacketMetadata};
use std::vec::Vec;
use std::collections::{HashMap, BTreeMap};
use nohash::{NoHashHasher, BuildNoHashHasher};
use std::hash::BuildHasherDefault;
use smoltcp::phy::{TunTapInterface, Loopback};
use std::io::Error;
use std::borrow::{Borrow, BorrowMut};
use smoltcp::iface::Interface;
use std::mem;
use smoltcp::time::Duration;
use crate::packet_headers::{ether_header, iphdr, udphdr, udp_packet};
use std::mem::{size_of, transmute};
use crate::uknetdev_interface::UkNetdevInterface;
use std::ffi::{c_void, CStr};
use crate::smoltcp_c_interface::{ETH_IPV4, PROTO_UDP, ETH_HEADER_SIZE, IP_HEADER_SIZE, UDP_HEADER_SIZE};
use std::ptr::{copy_nonoverlapping, null, null_mut, slice_from_raw_parts_mut};
use core::ptr;
use std::os::raw::c_char;

extern "C" {
    pub fn packet_handler_wrapper();
    pub fn uknetdev_output_wrapper(packet: *mut c_void);
}

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
    UkNetdevInterface(Stack<'a, 'b, UkNetdevInterface>)
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
        println!("ether_header: {}, iphdr: {}, udphdr: {}", size_of::<ether_header>(), size_of::<iphdr>(), size_of::<udphdr>());
        println!("Loopback stack created!");
        StackType::Loopback(stack)
    }
    pub fn new_uknetdev_stack() -> StackType<'a, 'b> {
        let device = UkNetdevInterface::new();
        let stack = Stack::new(device);
        println!("UkNetdev stack created!");
        StackType::UkNetdevInterface(stack)
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

    pub fn advance_clock(stack: &mut Stack<DeviceT>) -> u8 {
        match stack.iface.as_mut().unwrap().poll_delay(&stack.socket_set.borrow_mut(),
                                                       stack.clock.elapsed()) {
            Some(Duration { millis: 0 }) => {
                println!("resuming");
                0
            },
            Some(delay) => {
                println!("sleeping for {} ms", delay);
                stack.clock.advance(delay);
                0
            },
            None => {
                stack.clock.advance(Duration::from_millis(1));
                0
            }
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
                let rx_buffer =
                    UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY],vec![0; smol_socket.rx_buffer]);
                let tx_buffer =
                    UdpSocketBuffer::new(vec![UdpPacketMetadata::EMPTY],vec![0; smol_socket.tx_buffer]);
                let socket = UdpSocket::new(rx_buffer, tx_buffer);
                let socket_handle = stack.socket_set.add(socket);
                stack.handle_map.insert(stack.current_socket_handle, socket_handle);
                println!("A new socket was added with handle {}!", stack.current_socket_handle);

                stack.current_socket_handle = stack.current_socket_handle + 1;
                stack.current_socket_handle - 1
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
            Ok(_) => { 0 }
            Err(_) => { 1 }
        }
    }

    pub fn listen(stack: &mut Stack<DeviceT>, server_ip: IpAddress,
                  socket: u8, port: u16) -> u8 {
        let err: i32;
        {
            // first, we get the handle from the hashmap
            let handle = stack.handle_map.get(&socket).unwrap();
            // then, we get the TcpSocket from the SocketSet
            let mut socket = stack.socket_set.get::<TcpSocket>(*handle);

            err = {
                if !socket.is_active() && !socket.is_listening() {
                    let endpoint = IpEndpoint::new(server_ip, port);
                    let result = socket.listen(endpoint);
                    match result {
                        Ok(_) => { 0 }
                        Err(_) => { 1 }
                    }
                } else { 1 }
            };
        }
        match err {
            0 => { Stack::advance_clock(stack) }
            1 => { 1 }
            _ => { 1 }
        }
    }

    pub fn connect(stack: &mut Stack<DeviceT>, server_ip: IpAddress,
                   server_port: u16, client_socket: u8, client_port: u16) -> u8 {
        let err: i32;
        {
            // first, we get the handle from the hashmap
            let handle = stack.handle_map.get(&client_socket).unwrap();
            // then, we get the TcpSocket from the SocketSet
            let mut socket = stack.socket_set.get::<TcpSocket>(*handle);

            err = {
                if !socket.is_open() {
                    // server endpoint will have the Ip Address specified
                    // client endpoint doesn't require an IpAddress, therefore it is Unspecified
                    let server_endpoint = IpEndpoint::new(server_ip, server_port);
                    let client_endpoint = IpEndpoint::new(IpAddress::Unspecified, client_port);
                    let result = socket.connect(server_endpoint, client_endpoint);
                    match result {
                        Ok(_) => { 0 }
                        Err(_) => { 1 }
                    }
                } else { 1 }
            }

        }
        match err {
            0 => { Stack::advance_clock(stack) }
            1 => { 1 }
            _ => { 1 }
        }
    }

    pub fn bind(stack: &mut Stack<DeviceT>, socket: u8, port: u16) -> u8 {
        let err: i32;
        {
            let handle = stack.handle_map.get(&socket).unwrap();
            let mut socket = stack.socket_set.get::<UdpSocket>(*handle);
            let endpoint = IpEndpoint::new(IpAddress::Unspecified, port);

            err = {
                if !socket.is_open() {
                    let result = socket.bind(endpoint);
                    match result {
                        Ok(_) => {
                            println!("Successful bind!");
                            0
                        }
                        Err(_) => {
                            println!("Error on bind!");
                            1
                        }
                    }
                } else { 1 }
            }
        }
        match err {
            0 => { Stack::advance_clock(stack) }
            1 => { 1 }
            _ => { 1 }
        }
    }

    pub fn send(stack: &mut Stack<DeviceT>, client_socket: u8, message: &str) -> u8 {
        let handle = stack.handle_map.get(&client_socket).unwrap();
        let mut socket = stack.socket_set.get::<TcpSocket>(*handle);
        if socket.can_send() {
            println!("Socket sending!");
            socket.send_slice(message.as_ref()).unwrap();
            0
        }
        else {
            println!("Socket can't send!");
            1
        }
    }

    /* uknetdev_output equivalent */
    pub unsafe fn uk_send(stack: &mut Stack<DeviceT>, packet: *mut c_void) -> u8 {
        let eth_hdr = &mut {
            *(packet as *mut ether_header)
        };

        /* eth_hdr.ether_type is 8 for IPv4 encapsulated package and 56710 for IPv6 */
        if eth_hdr.ether_type == ETH_IPV4 {
            println!("ETHERNET {:?} {:?}", eth_hdr.ether_shost, eth_hdr.ether_dhost);
            let ip_hdr = &mut {
                *(packet.add(ETH_HEADER_SIZE)
                    as *mut iphdr)
            };

            /* ip_hdr.protocol is 17 (0x11) for UDP packages */
            if ip_hdr.protocol == PROTO_UDP {
                println!("UDP PACKET!!!!");
                let udp_hdr = &mut {
                    *(packet.add(ETH_HEADER_SIZE).
                        add(IP_HEADER_SIZE)
                        as *mut udphdr)
                };

                /* We need to swap bytes, to account for network byte order */
                let src_port = udp_hdr.source.swap_bytes();
                let dst_port = udp_hdr.dest.swap_bytes();
                let length = udp_hdr.len.swap_bytes();
                println!("UDP Ports {} {} length {}", src_port, dst_port, length);
                let handle = stack.handle_map.get(&(src_port as u8));
                let ret = match handle {
                    None => { 1 }
                    Some(_) => {
                        let payload_size = (length - 8) as usize;
                        let packet_size = ETH_HEADER_SIZE + IP_HEADER_SIZE + UDP_HEADER_SIZE + payload_size;

                        /* Get a Rust buffer from the packet; we're only interested in the last
                         * `payload_size` bytes
                         */
                        let buf = std::slice::from_raw_parts_mut(packet as *mut u8, packet_size);
                        let payload_buf = &buf[packet_size - payload_size..packet_size];
                        println!("{:x?}", payload_buf);
                        Stack::send(stack, src_port as u8, std::str::from_utf8(payload_buf).unwrap())
                    }
                };

                return ret
            }
        }
        //uknetdev_output_wrapper(new_packet);
        1
    }

    pub unsafe fn uk_recv(_stack: &mut Stack<DeviceT>) -> u8 {
        packet_handler_wrapper();
        0
    }

    pub fn recv(stack: &mut Stack<DeviceT>, server_socket: u8) -> u8 {
        let handle = stack.handle_map.get(&server_socket).unwrap();
        let mut socket = stack.socket_set.get::<TcpSocket>(*handle);
        if socket.can_recv() {
            println!("Socket receiving!");
            println!("got {:?}", socket.recv(|buffer| {
                (buffer.len(), std::str::from_utf8(buffer).unwrap())
            }));
            0
        }
        else {
            println!("Socket can't receive!");
            1
        }
    }

    pub fn close(stack: &mut Stack<DeviceT>, socket: u8){
        let handle = stack.handle_map.get(&socket).unwrap();
        let mut socket = stack.socket_set.get::<TcpSocket>(*handle);

        socket.close();
    }
}

pub unsafe fn send_udp_packet(mut udp_packet: udp_packet) {
    let udp_ptr = &mut udp_packet as *mut _ as *mut c_void;
    uknetdev_output_wrapper(udp_ptr);
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