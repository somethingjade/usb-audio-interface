#![no_std]
#![no_main]

use cortex_m::interrupt::free;
use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut shared = audio_interface::init::init_shared();
    audio_interface::init::start(&mut shared);
    free(|cs| {
        audio_interface::global::G_SHARED
            .0
            .borrow(cs)
            .replace(Some(shared));
    });
    audio_interface::init::enable_interrupts();
    loop {}
}
