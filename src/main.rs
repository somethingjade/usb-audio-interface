#![no_std]
#![no_main]

use cortex_m::interrupt;
use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let shared = audio_interface::init::init();
    interrupt::free(|cs| {
        audio_interface::global::G_SHARED
            .0
            .borrow(cs)
            .replace(Some(shared));
    });
    loop {}
}
