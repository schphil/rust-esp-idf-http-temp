use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Condvar;

use esp_idf_hal::{delay::Ets, peripherals::Peripherals};
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
    let peripherals = Peripherals::take().unwrap();

    #[allow(unused)]
    let sysloop = EspSystemEventLoop::take()?;

    #[allow(clippy::redundant_clone)]
    #[cfg(not(feature = "qemu"))]
    #[allow(unused_mut)]
    let mut wifi = wifi::wifi(peripherals.modem, sysloop.clone())?;

    let mutex = Arc::new((Mutex::new(None), Condvar::new()));

    let httpd = http::httpd(mutex.clone())?;

    let mut wait = mutex.0.lock().unwrap();

    let mut adc = temp_sensor::setup_adc()?;
    let mut adc_pin = temp_sensor::setup_adc_pin()?;

    #[allow(unused)]
    let cycles = loop {
        if let Some(cycles) = *wait {
            break cycles;
        } else {
            wait = mutex
                .1
                .wait_timeout(wait, std::time::Duration::from_secs(1))
                .unwrap()
                .0;

            log::info!(
                "Temperature sensor reading: {}mV",
                temp_sensor::read_temp_send(&mut adc, &mut adc_pin)?
            );
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
