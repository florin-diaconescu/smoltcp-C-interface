
use std::vec::Vec;
use std::collections::VecDeque;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(all(feature = "alloc", not(feature = "rust-1_28")))]
use alloc::collections::VecDeque;
#[cfg(all(feature = "alloc", feature = "rust-1_28"))]
use alloc::VecDeque;

use smoltcp::Result;
//use smoltcp::time::Instant;
use smoltcp::phy::{DeviceCapabilities, Device, Medium};
use smoltcp::phy;
use smoltcp::time::Instant;

/// A UkNetdevInterface device.
#[derive(Debug)]
pub struct UkNetdevInterface {
    rx_buffer: [u8; 1536],
    tx_buffer: [u8; 1536],
}

#[allow(clippy::new_without_default)]
impl UkNetdevInterface {
    /// Creates a UkNetdevInterface device.
    ///
    /// Every packet transmitted through this device will be received through it
    /// in FIFO order.
    pub fn new() -> UkNetdevInterface {
        UkNetdevInterface {
            rx_buffer: [0; 1536],
            tx_buffer: [0; 1536],
        }
    }
}

impl<'a> Device<'a> for UkNetdevInterface {
    type RxToken = UkNetdevRxToken<'a>;
    type TxToken = UkNetdevTxToken<'a>;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        Some((UkNetdevRxToken(&mut self.rx_buffer[..]),
              UkNetdevTxToken(&mut self.tx_buffer[..])))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(UkNetdevTxToken(&mut self.tx_buffer[..]))
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.max_burst_size = Some(1);
        caps.medium = Medium::Ethernet;
        caps
    }
}


pub struct UkNetdevRxToken<'a>(&'a mut [u8]);

impl<'a> phy::RxToken for UkNetdevRxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        // TODO: receive packet into buffer
        let result = f(&mut self.0);
        println!("rx called");
        result
    }
}

pub struct UkNetdevTxToken<'a>(&'a mut [u8]);

impl<'a> phy::TxToken for UkNetdevTxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        let result = f(&mut self.0[..len]);
        println!("tx called {}", len);
        // TODO: send packet out
        result

    }
}
