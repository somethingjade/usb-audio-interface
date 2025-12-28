use crate::{consts, global};
use usbd_audio::Error;

pub fn poll(usb: &mut global::Usb) -> bool {
    usb.usb_device.poll(&mut [&mut usb.usb_audio])
}

pub fn write_buffer(usb: &mut global::Usb, buf: &[u16]) -> Result<usize, Error> {
    let write_buf = &unsafe { *(buf as *const _ as *const [u8; consts::DMA_BUFFER_SIZE * 2]) };
    return usb.usb_audio.write(write_buf);
}
