#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write};

use esp32_hal::{
    gpio::{Gpio12, Gpio2, Gpio5, IO},
    pac::{self, Peripherals, UART0},
    prelude::*,
    Delay, RtcCntl, Serial, Timer,
};
use esp_hal_common::{
    gpio::{Event, Pin},
    interrupt, Cpu, Floating, Input, InputPin, PullDown, PullUp,
};
use panic_halt as _;
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::entry;

static SERIAL: SpinLockMutex<RefCell<Option<Serial<UART0>>>> =
    SpinLockMutex::new(RefCell::new(None));
static PBTN2: SpinLockMutex<RefCell<Option<Gpio2<Input<PullDown>>>>> =
    SpinLockMutex::new(RefCell::new(None));
static JBTN1: SpinLockMutex<RefCell<Option<Gpio5<Input<PullUp>>>>> =
    SpinLockMutex::new(RefCell::new(None));
static JBTN2: SpinLockMutex<RefCell<Option<Gpio12<Input<PullUp>>>>> =
    SpinLockMutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Disable the TIMG watchdog timer.
    let mut timer0 = Timer::new(peripherals.TIMG0);
    let serial0 = Serial::new(peripherals.UART0).unwrap();
    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    timer0.disable();
    rtc_cntl.set_wdt_global_enable(false);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio4.into_push_pull_output();
    let mut pbtn2 = io.pins.gpio2.into_pull_down_input();
    pbtn2.listen(Event::AnyEdge);
    let mut jbtn1 = io.pins.gpio5.into_pull_up_input();
    jbtn1.listen(Event::AnyEdge);
    let mut jbtn2 = io.pins.gpio12.into_pull_up_input();
    jbtn2.listen(Event::AnyEdge);

    unsafe {
        (&SERIAL).lock(|data| (*data).replace(Some(serial0)));
        (&PBTN2).lock(|data| (*data).replace(Some(pbtn2)));
        (&JBTN1).lock(|data| (*data).replace(Some(jbtn1)));
        (&JBTN2).lock(|data| (*data).replace(Some(jbtn2)));
    }

    interrupt::enable(
        Cpu::ProCpu,
        pac::Interrupt::GPIO,
        interrupt::CpuInterrupt::Interrupt1LevelPriority1,
    );

    led.set_high().unwrap();

    unsafe {
        (&SERIAL).lock(|data| {
            let mut serial = data.borrow_mut();
            let serial = serial.as_mut().unwrap();
            writeln!(serial, "Go go go").ok();
        });
    }

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new();

    unsafe {
        xtensa_lx::interrupt::enable_mask(1 << 1);
    }

    loop {
        led.toggle().unwrap();
        delay.delay_ms(500u32);
    }
}

#[no_mangle]
pub fn level1_interrupt() {
    unsafe {
        (&SERIAL).lock(|data| {
            let mut serial = data.borrow_mut();
            let serial = serial.as_mut().unwrap();
            writeln!(serial, "Interrupt").ok();
        });
    }

    interrupt::clear(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt1LevelPriority1,
    );

    unsafe {
        (&PBTN2).lock(|data| {
            let mut button = data.borrow_mut();
            let button = button.as_mut().unwrap();
            if button.is_pcore_interrupt_set() {
                (&SERIAL).lock(|data| {
                    let mut serial = data.borrow_mut();
                    let serial = serial.as_mut().unwrap();
                    writeln!(serial, "PBTN2").ok();
                });
                button.clear_interrupt();
            }
        });
        (&JBTN1).lock(|data| {
            let mut button = data.borrow_mut();
            let button = button.as_mut().unwrap();
            if button.is_pcore_interrupt_set() {
                (&SERIAL).lock(|data| {
                    let mut serial = data.borrow_mut();
                    let serial = serial.as_mut().unwrap();
                    writeln!(serial, "JBTN1").ok();
                });
                button.clear_interrupt();
                button.enable_input(true);
                button.listen(Event::AnyEdge);
            }
        });
        (&JBTN2).lock(|data| {
            let mut button = data.borrow_mut();
            let button = button.as_mut().unwrap();
            if button.is_pcore_interrupt_set() {
                (&SERIAL).lock(|data| {
                    let mut serial = data.borrow_mut();
                    let serial = serial.as_mut().unwrap();
                    writeln!(serial, "JBTN2").ok();
                });
                button.clear_interrupt();
            }
        });
    }
}
