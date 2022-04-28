#![no_std]
#![no_main]

use core::fmt::Write;

use esp32_hal::{
    gpio::{Gpio0, Gpio12, Gpio2, Gpio5, IO},
    pac::{self, Peripherals, UART0},
    prelude::*,
    Delay, RtcCntl, Serial, Timer,
};
use esp_hal_common::{
    gpio::{Event, Pin},
    interrupt, Cpu, Input, InputPin, PullUp,
};
use panic_halt as _;
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::entry;

static SERIAL: SpinLockMutex<Option<Serial<UART0>>> = SpinLockMutex::new(None);
static PBTN1: SpinLockMutex<Option<Gpio0<Input<PullUp>>>> = SpinLockMutex::new(None);
static PBTN2: SpinLockMutex<Option<Gpio2<Input<PullUp>>>> = SpinLockMutex::new(None);
static JBTN1: SpinLockMutex<Option<Gpio5<Input<PullUp>>>> = SpinLockMutex::new(None);
static JBTN2: SpinLockMutex<Option<Gpio12<Input<PullUp>>>> = SpinLockMutex::new(None);

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Disable the TIMG watchdog timer.
    let mut timer0 = Timer::new(peripherals.TIMG0);
    let mut rtc_cntl = RtcCntl::new(peripherals.RTC_CNTL);

    // Disable MWDT and RWDT (Watchdog) flash boot protection
    timer0.disable();
    rtc_cntl.set_wdt_global_enable(false);

    let serial0 = Serial::new(peripherals.UART0).unwrap();
    (&SERIAL).lock(|data| (*data).replace(serial0));

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio4.into_push_pull_output();

    interrupt::enable(
        Cpu::ProCpu,
        pac::Interrupt::GPIO,
        interrupt::CpuInterrupt::Interrupt1LevelPriority1,
    );

    /* push buttons */
    let mut pbtn1 = io.pins.gpio0.into_pull_up_input();
    pbtn1.listen(Event::FallingEdge);
    (&PBTN1).lock(|data| (*data).replace(pbtn1));

    let mut pbtn2 = io.pins.gpio2.into_pull_up_input();
    pbtn2.listen(Event::FallingEdge);
    (&PBTN2).lock(|data| (*data).replace(pbtn2));

    /* joystick buttons */
    let mut jbtn1 = io.pins.gpio5.into_pull_up_input();
    jbtn1.listen(Event::FallingEdge);
    (&JBTN1).lock(|data| (*data).replace(jbtn1));

    let mut jbtn2 = io.pins.gpio12.into_pull_up_input();
    jbtn2.listen(Event::FallingEdge);
    (&JBTN2).lock(|data| (*data).replace(jbtn2));

    // ackshully there are two banks, another one for GPIO > 32
    unsafe {
        xtensa_lx::interrupt::disable();
        // xtensa_lx::interrupt::enable_mask(1 << 1);
        xtensa_lx::interrupt::enable_mask(1 << 1);
    }

    led.set_high().unwrap();

    (&SERIAL).lock(|data| {
        let serial = data.as_mut().unwrap();
        writeln!(serial, "Go go go").ok();
    });

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new();

    loop {
        led.toggle().unwrap();
        delay.delay_ms(500u32);
    }
}

#[no_mangle]
pub fn level1_interrupt() {
    (&SERIAL).lock(|data| {
        let serial = data.as_mut().unwrap();
        writeln!(serial, "Interrupt1").ok();
    });

    interrupt::clear(
        Cpu::ProCpu,
        //interrupt::CpuInterrupt::Interrupt10EdgePriority1,
        interrupt::CpuInterrupt::Interrupt1LevelPriority1,
    );
    /* push buttons */
    (&PBTN1).lock(|data| {
        let button = data.as_mut().unwrap();
        if button.is_pcore_interrupt_set() {
            (&SERIAL).lock(|data| {
                let serial = data.as_mut().unwrap();
                writeln!(serial, "PBTN1").ok();
            });
            button.clear_interrupt();
        }
    });
    (&PBTN2).lock(|data| {
        let button = data.as_mut().unwrap();
        if button.is_pcore_interrupt_set() {
            (&SERIAL).lock(|data| {
                let serial = data.as_mut().unwrap();
                writeln!(serial, "PBTN2").ok();
            });
            button.clear_interrupt();
        }
    });
    /* joystick buttons */
    (&JBTN1).lock(|data| {
        let button = data.as_mut().unwrap();
        if button.is_pcore_interrupt_set() {
            (&SERIAL).lock(|data| {
                let serial = data.as_mut().unwrap();
                writeln!(serial, "JBTN1").ok();
            });
            button.clear_interrupt();
        }
    });
    (&JBTN2).lock(|data| {
        let button = data.as_mut().unwrap();
        if button.is_pcore_interrupt_set() {
            (&SERIAL).lock(|data| {
                let serial = data.as_mut().unwrap();
                writeln!(serial, "JBTN2").ok();
            });
            button.clear_interrupt();
        }
    });
}
