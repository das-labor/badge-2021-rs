#![no_std]
#![no_main]

use arrform::{arrform, ArrForm};
use core::{borrow::BorrowMut, cell::RefCell};

use embedded_graphics::mono_font::{
    ascii::{FONT_10X20, FONT_6X10},
    MonoTextStyle,
};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

use esp32_hal::{
    clock::ClockControl,
    gpio::{Event, Gpio0, Gpio12, Gpio2, Gpio5, Input, Pin, PullUp, IO},
    i2c::I2C,
    interrupt,
    peripherals::{Interrupt, Peripherals, I2C0, UART0},
    prelude::*,
    rtc_cntl::RtcClock,
    timer::TimerGroup,
    xtensa_lx, Cpu, Delay, Timer, Uart,
};
use esp_backtrace as _;
use esp_println::println;

use shared_bus;
use ssd1306::{self, mode::DisplayConfig, I2CDisplayInterface, Ssd1306};

use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt::entry;

// Used in example code, see
// https://github.com/esp-rs/esp-hal/blob/main/esp32-hal/examples/gpio_interrupt.rs
// use critical_section::Mutex;

type LCD128x64<'a> = Ssd1306<
    ssd1306::prelude::I2CInterface<shared_bus::I2cProxy<'a, I2C0>>,
    ssd1306::size::DisplaySize128x64,
    ssd1306::mode::BufferedGraphicsMode<ssd1306::size::DisplaySize128x64>,
>;

/*
static PBTN1: Mutex<Option<Gpio0<Input<PullUp>>>> = Mutex::new(RefCell::new(None));
static PBTN2: Mutex<Option<Gpio2<Input<PullUp>>>> = Mutex::new(RefCell::new(None));
static JBTN1: Mutex<Option<Gpio5<Input<PullUp>>>> = Mutex::new(RefCell::new(None));
static JBTN2: Mutex<Option<Gpio12<Input<PullUp>>>> = Mutex::new(RefCell::new(None));
static DI1: Mutex<Option<LCD128x64>> = Mutex::new(RefCell::new(None));

static BOOP: Mutex<bool> = Mutex::new(RefCell::new(false));
*/

static PBTN1: SpinLockMutex<Option<Gpio0<Input<PullUp>>>> = SpinLockMutex::new(None);
static PBTN2: SpinLockMutex<Option<Gpio2<Input<PullUp>>>> = SpinLockMutex::new(None);
static JBTN1: SpinLockMutex<Option<Gpio5<Input<PullUp>>>> = SpinLockMutex::new(None);
static JBTN2: SpinLockMutex<Option<Gpio12<Input<PullUp>>>> = SpinLockMutex::new(None);
static DI1: SpinLockMutex<Option<LCD128x64>> = SpinLockMutex::new(None);

static BOOP: SpinLockMutex<bool> = SpinLockMutex::new(false);

// unsafe impl Sync for I2C0 {}

fn draw<'a, D>(display: &mut D, title: &'a str, msg: &'a str) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    display.clear(Rgb565::BLACK.into())?;

    Rectangle::new(display.bounding_box().top_left, display.bounding_box().size)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE.into())
                .stroke_color(Rgb565::YELLOW.into())
                .stroke_width(1)
                .build(),
        )
        .draw(display)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE.into());
    Text::with_baseline(title, Point::new(3, 3), text_style, Baseline::Top).draw(display)?;

    Text::new(
        msg,
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    Ok(())
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Set GPIO4 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio4.into_push_pull_output();

    led.set_high().unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;
    /*
        timer0.disable();
        timer0.start(30_000_000u64);
    */

    println!("Go go go");

    // FIXME: As of now, push button 2 and joystick button 2 trigger once
    // initially. This is an issue in the esp-hal crate.
    // https://github.com/esp-rs/esp-hal/issues/54#issuecomment-1115306416
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

    interrupt::enable(Interrupt::GPIO, interrupt::Priority::Priority3).unwrap();

    /* I2C OLED displays */
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;
    let i2c = I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        400u32.kHz(), // 400kHz
        &clocks,
    );

    println!("Initialize displays...");
    // Instantiate
    let i2c_bus = shared_bus::BusManagerSimple::new(i2c);
    let di1 = I2CDisplayInterface::new(i2c_bus.acquire_i2c());
    let di2 = I2CDisplayInterface::new_alternate_address(i2c_bus.acquire_i2c());

    // Initialize
    let mut d1 = Ssd1306::new(
        di1,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    d1.init().expect("display 1 init");
    // (&DI1).lock(|data| (*data).replace(d1));

    let mut d2 = Ssd1306::new(
        di2,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    d2.init().expect("display 2 init");

    println!("Test draw on displays...");

    // Draw! :)
    draw(&mut d1, ">> Das Labor <<", "Write Rust!").expect("draw");
    d1.flush().unwrap();

    draw(&mut d2, "\\o/ *woop woop* \\o/", "Party hard!").unwrap();
    d2.flush().unwrap();

    // Good to go, let the LED shine!
    led.set_high().unwrap();
    let mut x = 0;

    println!("Initialized. Enter loop...");

    /* main loop :) */
    loop {
        led.toggle().unwrap();
        delay.delay_ms(500u32);
        if (&BOOP).lock(|data| data.clone()) {
            (&BOOP).lock(|data| {
                println!("boop boop");
                x += 1;
                let y = arrform!(13, "YEEHAW {}", x);
                draw(&mut d2, "BOOP BOOP", y.as_str()).unwrap();
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

    /* push buttons */
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
}
