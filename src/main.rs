#![no_std]
#![no_main]

use arrform::{arrform, ArrForm};

mod gfx;
mod gfx_spi;
mod res;

use esp32_hal::{
    clock::ClockControl,
    gpio::{Event, Gpio0, Gpio12, Gpio2, Gpio5, Input, Pin, PullUp, IO},
    i2c::I2C,
    interrupt,
    peripherals::{Interrupt, Peripherals},
    prelude::*,
    spi::{master::Spi, SpiMode},
    timer::TimerGroup,
    xtensa_lx::mutex::{Mutex, SpinLockMutex},
    Cpu, Delay,
};
use esp_backtrace as _;
use esp_println::println;

use shared_bus::BusManagerSimple;

// Used in example code, see
// https://github.com/esp-rs/esp-hal/blob/main/esp32-hal/examples/gpio_interrupt.rs
use core::{borrow::BorrowMut, cell::RefCell};
use critical_section::Mutex as CSMutex;

type RefCMutex<G> = CSMutex<RefCell<Option<G>>>;

// type GpioMutex = RefCMutex<...>;
//static PBTN1: GpioMutex = CSMutex::new(RefCell::new(None));

static PBTN1: RefCMutex<Gpio0<Input<PullUp>>> = CSMutex::new(RefCell::new(None));
static PBTN2: RefCMutex<Gpio2<Input<PullUp>>> = CSMutex::new(RefCell::new(None));
static JBTN1: RefCMutex<Gpio5<Input<PullUp>>> = CSMutex::new(RefCell::new(None));
static JBTN2: RefCMutex<Gpio12<Input<PullUp>>> = CSMutex::new(RefCell::new(None));

// static BOOP: Mutex<bool> = Mutex::new(RefCell::new(false));

/*
static PBTN1: SpinLockMutex<Option<Gpio0<Input<PullUp>>>> = SpinLockMutex::new(None);
static PBTN2: SpinLockMutex<Option<Gpio2<Input<PullUp>>>> = SpinLockMutex::new(None);
static JBTN1: SpinLockMutex<Option<Gpio5<Input<PullUp>>>> = SpinLockMutex::new(None);
static JBTN2: SpinLockMutex<Option<Gpio12<Input<PullUp>>>> = SpinLockMutex::new(None);
*/

// TODO: https://docs.rs/shared-bus/latest/shared_bus/struct.XtensaMutex.html
static BOOP: SpinLockMutex<bool> = SpinLockMutex::new(false);

// static DI1: SpinLockMutex<Option<LCD128x64>> = SpinLockMutex::new(None);
// unsafe impl Sync for I2C0 {}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt = timer_group0.wdt;
    /*
    let mut timer0 = timer_group0.timer0;
    timer0.disable();
    timer0.start(30_000_000u64);
    */

    println!("Go go go");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Group IO pins here to keep an overview

    // NOTE: GPIO4 is also the status LED on the ESP32 board, but inverted.
    let mut led = io.pins.gpio4.into_push_pull_output();
    // I2C
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;
    // SPI
    let sclk = io.pins.gpio14;
    let miso = io.pins.gpio15; // TODO: use dummy!
    let mosi = io.pins.gpio13;
    let cs0 = io.pins.gpio10;
    let mut cs1 = io.pins.gpio23.into_push_pull_output();
    let mut cs2 = io.pins.gpio18.into_push_pull_output();
    // SPI displays
    let rst1 = io.pins.gpio17.into_push_pull_output();
    let rst2 = io.pins.gpio19.into_push_pull_output();
    let dc = io.pins.gpio16.into_push_pull_output();

    // Big kudos to Bjoern for getting the ESP23's GPIO interrupts fixed:
    // https://github.com/esp-rs/esp-hal/issues/54#issuecomment-1115306416
    /* push buttons */
    let mut pbtn1 = io.pins.gpio0.into_pull_up_input();
    pbtn1.listen(Event::RisingEdge);
    // (&PBTN1).lock(|data| (*data).replace(pbtn1));

    let mut pbtn2 = io.pins.gpio2.into_pull_up_input();
    pbtn2.listen(Event::RisingEdge);
    // (&PBTN2).lock(|data| (*data).replace(pbtn2));

    /* joystick buttons */
    let mut jbtn1 = io.pins.gpio5.into_pull_up_input();
    jbtn1.listen(Event::RisingEdge);
    // (&JBTN1).lock(|data| (*data).replace(jbtn1));

    let mut jbtn2 = io.pins.gpio12.into_pull_up_input();
    jbtn2.listen(Event::RisingEdge);
    // (&JBTN2).lock(|data| (*data).replace(jbtn2));

    critical_section::with(|cs| {
        PBTN1.borrow_ref_mut(cs).replace(pbtn1);
        PBTN2.borrow_ref_mut(cs).replace(pbtn2);
        JBTN1.borrow_ref_mut(cs).replace(jbtn1);
        JBTN2.borrow_ref_mut(cs).replace(jbtn2);
    });

    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    println!("Initialize OLED displays...");
    let i2c = I2C::new(peripherals.I2C0, sda, scl, 400u32.kHz(), &clocks);
    let i2c_bus = BusManagerSimple::new(i2c);

    // TODO: Figure out how to move the mode change into `init_displays()`.
    use ssd1306::mode::DisplayConfig;
    let (d1, d2) = gfx::init_displays(&i2c_bus);
    let d1 = &mut d1.into_buffered_graphics_mode();
    d1.init().expect("display 1 init");
    let d2 = &mut d2.into_buffered_graphics_mode();
    d2.init().expect("display 1 init");
    // TODO: Can we get a mutex on this?
    // (&DI1).lock(|data| (*data).replace(d1));

    println!("Test draw on OLED displays...");
    gfx::draw(d1, ">> Das Labor <<", "Write Rust!").expect("draw");
    d1.flush().unwrap();
    gfx::draw(d2, "\\o/ *woop woop* \\o/", "Party hard!").unwrap();
    d2.flush().unwrap();

    println!("Initialize SPI displays...");
    let spi = Spi::new(
        peripherals.SPI2,
        sclk,
        mosi,
        miso,
        cs0, // TODO: use dummy
        16u32.MHz(),
        SpiMode::Mode0,
        &clocks,
    );

    let spi_bus = BusManagerSimple::new(spi);
    let (tft1, tft2) =
        &mut gfx_spi::init_displays(&spi_bus, &mut delay, dc, rst1, rst2, &mut cs1, &mut cs2);

    use embedded_graphics::prelude::*;
    println!("Splash splash...");
    cs2.set_high().unwrap();
    cs1.set_low().unwrap();
    tft1.clear(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .unwrap();
    gfx_spi::splash(tft1, &mut delay);
    cs1.set_high().unwrap();
    cs2.set_low().unwrap();
    tft2.clear(embedded_graphics::pixelcolor::Rgb565::BLACK)
        .unwrap();
    gfx_spi::splash(tft2, &mut delay);
    delay.delay_ms(1000u32);
    gfx::splash(d1, d2, &mut delay);
    delay.delay_ms(1000u32);

    gfx::draw(d1, "GFX", "regines").expect("draw");
    d1.flush().unwrap();

    gfx::draw(d2, "code", "CyReVolt").expect("draw");
    d2.flush().unwrap();

    delay.delay_ms(1500u32);

    // Good to go, let the LED shine!
    led.set_high().unwrap();
    delay.delay_ms(50u32);

    let mut x = 0;

    // Who let the dogs out?
    wdt.start(2u64.secs());

    println!("Initialized. Enter loop...");
    loop {
        wdt.feed();
        led.set_low().unwrap();
        delay.delay_ms(200u32);
        led.set_high().unwrap();
        delay.delay_ms(2u32);
        if (&BOOP).lock(|data| data.clone()) {
            (&BOOP).lock(|data| {
                println!("boop boop");
                x += 1;
                let y = arrform!(13, "YEEHAW {x}");
                gfx::draw(d2, "BOOP BOOP", y.as_str()).unwrap();
                d2.flush().unwrap();
                *data = false;
            });
        }
    }
}

#[ram]
#[interrupt]
unsafe fn GPIO() {
    let lvl = xtensa_lx::interrupt::get_level();
    println!("GPIO Interrupt with priority {lvl}",);

    // TODO: Is this necessary?
    interrupt::clear(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt22EdgePriority3,
    );

    critical_section::with(|cs| {
        PBTN1
            .borrow_ref_mut(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt();
        PBTN2
            .borrow_ref_mut(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt();
        JBTN1
            .borrow_ref_mut(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt();
        JBTN2
            .borrow_ref_mut(cs)
            .borrow_mut()
            .as_mut()
            .unwrap()
            .clear_interrupt();
        (&BOOP).lock(|data| {
            *data = true;
        });
    });

    /* push buttons */
    /*
    (&PBTN1).lock(|data| {
        let button = data.as_mut().unwrap();
        // if button.is_interrupt_set() {
        println!("PBTN1");
        button.clear_interrupt();
        // }
    });
    (&PBTN2).lock(|data| {
        let button = data.as_mut().unwrap();
        // if button.is_interrupt_set() {
        // println!("PBTN2");
        (&BOOP).lock(|data| {
            *data = true;
        });
        button.clear_interrupt();
        //}
    });

    /* joystick buttons */
    (&JBTN1).lock(|data| {
        let button = data.as_mut().unwrap();
        // if button.is_interrupt_set() {
        // println!("JBTN1");
        button.clear_interrupt();
        // }
    });
    (&JBTN2).lock(|data| {
        let button = data.as_mut().unwrap();
        // if button.is_interrupt_set() {
        // println!("JBTN2");
        button.clear_interrupt();
        // }
    });
    */
}
