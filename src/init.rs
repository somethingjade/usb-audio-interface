use crate::{consts, global};
use cortex_m::{peripheral, singleton};
use stm32f4xx_hal::{
    adc, dma,
    gpio::GpioExt,
    interrupt, otg_fs, pac,
    prelude::{_fugit_RateExtU32, _stm32f4xx_hal_rcc_RccExt},
    rcc,
    timer::TimerExt,
};
use usb_device::{bus, device};
use usbd_audio::{AudioClassBuilder, Format, StreamConfig, TerminalType};

pub fn init_shared<'a>() -> global::Shared<'a> {
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hsi()
            .sysclk(consts::SYSCLK_MHZ.MHz())
            .require_pll48clk(),
    );
    let gpioa = dp.GPIOA.split(&mut rcc);
    let adc_pin_pa4 = gpioa.pa4.into_analog();
    let dm_pin_pa11 = gpioa.pa11.into_alternate();
    let dp_pin_pa12 = gpioa.pa12.into_alternate();
    let mut counter = dp.TIM2.counter_hz(&mut rcc);
    counter.set_master_mode(pac::tim2::cr2::MMS::Update);
    let adc_config = adc::config::AdcConfig::default().external_trigger(
        adc::config::TriggerMode::RisingEdge,
        adc::config::ExternalTrigger::Tim_2_trgo,
    );
    let mut adc = adc::Adc::new(dp.ADC1, true, adc_config, &mut rcc);
    adc.configure_channel(
        &adc_pin_pa4,
        adc::config::Sequence::One,
        adc::config::SampleTime::Cycles_480,
    );
    let dma = dma::StreamsTuple::new(dp.DMA2, &mut rcc);
    let dma_config = dma::config::DmaConfig::default()
        .transfer_complete_interrupt(true)
        .memory_increment(true)
        .double_buffer(true);
    let first_buffer = singleton!(: [u16; consts::DMA_BUFFER_SIZE] = [0; consts::DMA_BUFFER_SIZE]);
    let second_buffer = singleton!(: [u16; consts::DMA_BUFFER_SIZE] = [0; consts::DMA_BUFFER_SIZE]);
    let transfer = dma::Transfer::init_peripheral_to_memory(
        dma.0,
        adc,
        first_buffer.unwrap(),
        second_buffer,
        dma_config,
    );
    let new_buffer = singleton!(: [u16; consts::DMA_BUFFER_SIZE] = [0; consts::DMA_BUFFER_SIZE]);
    let ep_memory =
        singleton!(: [u32; consts::EP_MEMORY_SIZE] = [0; consts::EP_MEMORY_SIZE]).unwrap();
    let usb = otg_fs::USB::new(
        (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
        (dm_pin_pa11, dp_pin_pa12),
        &rcc.clocks,
    );
    let usb_bus = otg_fs::UsbBus::new(usb, ep_memory);
    let usb_bus_ref =
        singleton!(: bus::UsbBusAllocator<otg_fs::UsbBus<otg_fs::USB>> = usb_bus).unwrap();
    let usb_audio = AudioClassBuilder::new()
        .input(
            StreamConfig::new_discrete(
                Format::S16le,
                1,
                &[consts::USB_AUDIO_RATE],
                TerminalType::InMicrophone,
            )
            .unwrap(),
        )
        .build(usb_bus_ref)
        .unwrap();
    let usb_device = device::UsbDeviceBuilder::new(usb_bus_ref, device::UsbVidPid(0x16c0, 0x27dd))
        .max_packet_size_0(64)
        .unwrap()
        .composite_with_iads()
        .build(); // VID and PID from stm32f4xx_hal USB examples
    return global::Shared {
        dma: global::Dma {
            buffer: new_buffer,
            transfer,
        },
        usb: global::Usb {
            usb_device,
            usb_audio,
        },
        counter_hz: counter,
    };
}

pub fn start(shared: &mut global::Shared) {
    shared.dma.transfer.start(|_| {});
    shared
        .counter_hz
        .start(consts::USB_AUDIO_RATE.Hz())
        .unwrap();
}

pub fn enable_interrupts() {
    unsafe {
        peripheral::NVIC::unmask(interrupt::DMA2_STREAM0);
    }
}
