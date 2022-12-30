use std::sync::{Arc, Condvar, Mutex};

use esp_idf_hal::{
    adc,
    adc::{config::Config, Atten11dB},
    gpio::Gpio0,
    peripherals::Peripherals,
};
use esp_idf_svc::eventloop::*;
use esp_idf_sys as _;
use log::*;

use error::Error;

pub mod error;
pub mod http;
pub mod temp_sensor;
pub mod wifi;

fn main() -> Result<(), Error> {
    esp_idf_sys::link_patches();

    #[allow(unused)]
    let mut peripherals = Peripherals::take().unwrap();
    let adc1 = peripherals.adc1;
    let adc = adc::AdcDriver::new(adc1, &Config::new().calibration(true))?;
    let analog_pin = peripherals.pins.gpio0;
    let adc_pin: esp_idf_hal::adc::AdcChannelDriver<'_, Gpio0, Atten11dB<_>> = adc::AdcChannelDriver::new(analog_pin)?;
    let adc_mutex = Arc::new(Mutex::new(adc));
    let adc_pin_mutex = Arc::new(Mutex::new(adc_pin));

    #[allow(unused)]
    let sysloop = EspSystemEventLoop::take()?;

    #[allow(clippy::redundant_clone)]
    #[cfg(not(feature = "qemu"))]
    #[allow(unused_mut)]
    let mut wifi = wifi::wifi(peripherals.modem, sysloop.clone())?;

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = http::httpd(mutex.clone(), adc_mutex, adc_pin_mutex)?;

    let mut wait = mutex.0.lock().unwrap();

    #[allow(unused)]
    let cycles: _ = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex.1.wait_timeout(wait, std::time::Duration::from_secs(1)).unwrap().0;
        }
    };

    for s in 0..3 {
        info!("Shutting down in {} secs", 3 - s);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    drop(httpd);
    info!("Httpd stopped");

    #[cfg(not(feature = "qemu"))]
    {
        drop(wifi);
        info!("Wifi stopped");
    }

    Ok(())
}
