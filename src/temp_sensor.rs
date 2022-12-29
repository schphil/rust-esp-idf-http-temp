use esp_idf_hal::{
    adc,
    adc::{config::Config, AdcChannelDriver, AdcDriver, Atten11dB},
    gpio::Gpio0,
    peripherals::Peripherals,
};
use libm::log;
use log::*;

use super::error::Error;

static VCC: f64 = 3.3;
static R2: f64 = 10000.0;
static ADCMAX: f64 = 4095.0;

static A: f64 = 0.001129148;
static B: f64 = 0.000234125;
static C: f64 = 0.0000000876741;

pub fn setup_adc() -> Result<AdcDriver<'static, adc::ADC1>, Error> {
    let peripherals = Peripherals::take().unwrap();
    let adc1 = peripherals.adc1;
    Ok(adc::AdcDriver::new(adc1, &Config::new().calibration(true))?)
}

pub fn setup_adc_pin() -> Result<AdcChannelDriver<'static, Gpio0, Atten11dB<adc::ADC1>>, Error> {
    let peripherals = Peripherals::take().unwrap();
    let analog_pin = peripherals.pins.gpio0;
    let adc_pin: esp_idf_hal::adc::AdcChannelDriver<'_, Gpio0, Atten11dB<_>> = adc::AdcChannelDriver::new(analog_pin)?;
    Ok(adc_pin)
}

pub fn read_temp_send(
    adc: &mut AdcDriver<'static, adc::ADC1>,
    adc_pin: &mut AdcChannelDriver<'static, Gpio0, Atten11dB<adc::ADC1>>,
) -> Result<f64, Error> {
    let adc = adc.read(adc_pin)?;
    let v_out = adc as f64 * VCC / ADCMAX;
    let r = v_out * R2 / (VCC - v_out);

    let t_k = 1.0 / (A + (B * log(r)) + (C * log(r) * log(r) * log(r)));
    let t_c = t_k - 273.15;

    info!("Analog value: {}", adc);
    info!("Temperature: {}", t_c);

    Ok(t_c)
}
