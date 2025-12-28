use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use stm32f4xx_hal::{adc, dma, otg_fs, pac, timer};
use usb_device::device;
use usbd_audio::AudioClass;

use crate::consts;

pub struct Global<T>(pub Mutex<RefCell<Option<T>>>);

impl<T> Global<T> {
    const fn init() -> Self {
        Self(Mutex::new(RefCell::new(None)))
    }
}

pub struct Dma<'a> {
    pub buffer: Option<&'a mut [u16; consts::DMA_BUFFER_SIZE]>,
    pub transfer: dma::Transfer<
        dma::StreamX<pac::DMA2, 0>,
        0,
        adc::Adc<pac::ADC1>,
        dma::PeripheralToMemory,
        &'a mut [u16; consts::DMA_BUFFER_SIZE],
    >,
}

pub struct Usb<'a> {
    pub usb_device: device::UsbDevice<'a, otg_fs::UsbBus<otg_fs::USB>>,
    pub usb_audio: AudioClass<'a, otg_fs::UsbBus<otg_fs::USB>>,
}

pub struct Shared<'a> {
    pub dma: Dma<'a>,
    pub usb: Usb<'a>,
    pub counter_hz: timer::counter::CounterHz<pac::TIM2>,
}

pub static G_SHARED: Global<Shared> = Global::init();
