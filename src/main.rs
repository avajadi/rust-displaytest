use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Rectangle, Triangle},
    text::{Baseline, Text},
    Drawable,
};
use embedded_hal::digital::v2::OutputPin as EHOutputPin;
use rppal::gpio::Gpio;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use ssd1306::{prelude::*, size::DisplaySize128x64, Ssd1306};
use std::error::Error;

// Custom error wrapper to make DisplayError compatible with std::error::Error
#[derive(Debug)]
#[allow(dead_code)]
struct DisplayErrorWrapper(display_interface::DisplayError);

impl std::fmt::Display for DisplayErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Display error")
    }
}

impl Error for DisplayErrorWrapper {}

impl From<display_interface::DisplayError> for DisplayErrorWrapper {
    fn from(error: display_interface::DisplayError) -> Self {
        DisplayErrorWrapper(error)
    }
}

// Adapter to make rppal::gpio::OutputPin compatible with embedded-hal
struct PinAdapter(rppal::gpio::OutputPin);

impl EHOutputPin for PinAdapter {
    type Error = rppal::gpio::Error;

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high();
        Ok(())
    }
}

// Adapter to make rppal::Spi compatible with embedded-hal
struct SpiAdapter(Spi);

impl embedded_hal::blocking::spi::Write<u8> for SpiAdapter {
    type Error = rppal::spi::Error;

    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.0.write(data)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing OLED display...");

    // Configure SPI
    let spi = SpiAdapter(Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?);

    // Configure GPIO pins
    let gpio = Gpio::new()?;
    let dc_pin = PinAdapter(gpio.get(25)?.into_output());  // Data/Command pin
    let mut reset_pin = PinAdapter(gpio.get(24)?.into_output());  // Reset pin
    let cs_pin = PinAdapter(gpio.get(8)?.into_output());  // Chip Select pin

    // Create display interface
    let interface = display_interface_spi::SPIInterface::new(spi, dc_pin, cs_pin);

    // Create display
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    // Reset the display
    reset_pin.set_high()?;
    std::thread::sleep(std::time::Duration::from_millis(1));
    reset_pin.set_low()?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    reset_pin.set_high()?;

    // Initialize the display (handle error conversion manually)
    match display.init() {
        Ok(_) => {},
        Err(e) => return Err(Box::new(DisplayErrorWrapper(e))),
    }

    // Clear the display (explicitly provide the BinaryColor parameter)
    match display.clear(BinaryColor::Off) {
        Ok(_) => {},
        Err(e) => return Err(Box::new(DisplayErrorWrapper(e))),
    }

    // Create styles
    let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);
    let thin_stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    println!("Drawing shapes and text...");

    // Draw shapes and handle errors manually
    let result = Triangle::new(
        Point::new(16, 16),
        Point::new(16 + 16, 16),
        Point::new(16 + 8, 16 - 8),
    )
        .into_styled(thin_stroke)
        .draw(&mut display);

    if let Err(e) = result {
        return Err(Box::new(DisplayErrorWrapper(e)));
    }

    let result = Circle::new(Point::new(64, 32), 8)
        .into_styled(thin_stroke)
        .draw(&mut display);

    if let Err(e) = result {
        return Err(Box::new(DisplayErrorWrapper(e)));
    }

    let result = Rectangle::new(Point::new(80, 16), Size::new(32, 32))
        .into_styled(thin_stroke)
        .draw(&mut display);

    if let Err(e) = result {
        return Err(Box::new(DisplayErrorWrapper(e)));
    }

    // Write text and handle errors manually
    let result = Text::with_baseline("Raspberry Pi Zero W", Point::new(5, 5), text_style, Baseline::Top)
        .draw(&mut display);

    if let Err(e) = result {
        return Err(Box::new(DisplayErrorWrapper(e)));
    }

    let result = Text::with_baseline("SSD1306 OLED", Point::new(5, 50), text_style, Baseline::Top)
        .draw(&mut display);

    if let Err(e) = result {
        return Err(Box::new(DisplayErrorWrapper(e)));
    }

    // Update the display
    match display.flush() {
        Ok(_) => {},
        Err(e) => return Err(Box::new(DisplayErrorWrapper(e))),
    }

    println!("Display initialized and pattern drawn successfully!");
    println!("Press Ctrl+C to exit...");

    // Keep the program running to maintain the display
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
