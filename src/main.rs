#![no_std]
#![no_main]

use esp32c3_hal::{
    clock::ClockControl, esp_riscv_rt::entry, i2c::I2C, peripherals::Peripherals, prelude::*,
    timer::TimerGroup, Rtc, IO,
};
use esp_backtrace as _;
use esp_println::println;
use nobcd::BcdNumber;

#[entry]
fn main() -> ! {
    // Take Peripherals, Initialize Clocks, and Create a Handle for Each
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Instantiate and Create Handles for the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    // Disable the RTC and TIMG watchdog timers
    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    // Instantiate and Create Handle for IO
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut ds1307 = I2C::new(
        peripherals.I2C0,
        io.pins.gpio3,
        io.pins.gpio2,
        100u32.kHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    );

    #[repr(u8)]
    enum DS1307 {
        Seconds,
        Minutes,
        Hours,
        Day,
        Date,
        Month,
        Year,
    }

    struct DateTime {
        sec: u8,
        min: u8,
        hrs: u8,
        day: u8,
        date: u8,
        mnth: u8,
        yr: u8,
    }

    let mut current_dt = DateTime {
        sec: 0,
        min: 0,
        hrs: 0,
        day: 0,
        date: 0,
        mnth: 0,
        yr: 0,
    };

    // Application Loop
    loop {
        let mut data = [0u8; 1];

        // Activate Oscillator
        ds1307.write(0x68, &[DS1307::Seconds as u8, 0_u8]).unwrap();
        println!("Oscillator Activated");

        // Read Seconds
        ds1307.write(0x68, &[DS1307::Seconds as u8]).unwrap();
        ds1307.read(0x68, &mut data).unwrap();

        current_dt.sec = BcdNumber::from_bcd_bytes(data).unwrap().value();
        println!("Seconds Read");
    }
}
