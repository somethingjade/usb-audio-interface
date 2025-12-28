use cortex_m::interrupt::free;
use stm32f4xx_hal::{
    ClearFlags, ReadFlags,
    dma::{self, traits::DmaFlagExt},
    interrupt,
};

use crate::{global, usb};

#[interrupt]
fn DMA2_STREAM0() {
    free(|cs| {
        if let Some(shared) = global::G_SHARED.0.borrow(cs).borrow_mut().as_mut() {
            let global::Dma {
                buffer: new_buffer_option,
                transfer,
            } = &mut shared.dma;
            let flags = transfer.flags();
            if flags.is_transfer_complete() {
                let new_buffer = new_buffer_option.take().unwrap();
                let current_buffer = transfer.next_transfer(new_buffer).unwrap().0;
                usb::poll(&mut shared.usb);
                let _ = usb::write_buffer(&mut shared.usb, current_buffer);
                *new_buffer_option = Some(current_buffer);
            }
            transfer.clear_flags(dma::DmaFlag::FifoError | dma::DmaFlag::TransferComplete);
        }
    });
}
