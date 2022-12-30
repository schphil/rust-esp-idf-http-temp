use std::sync::{Arc, Mutex};

use esp_idf_hal::{
    adc,
    adc::{AdcChannelDriver, AdcDriver, Atten11dB},
    gpio::Gpio0,
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

pub fn read_temp_send(
    adc_mutex: Arc<Mutex<AdcDriver<'static, adc::ADC1>>>,
    adc_pin_mutex: Arc<Mutex<AdcChannelDriver<'static, Gpio0, Atten11dB<adc::ADC1>>>>,
) -> Result<f64, Error> {
    let mut adc = adc_mutex.lock().unwrap();
    let mut adc_pin = adc_pin_mutex.lock().unwrap();
    let adc = adc.read(&mut adc_pin)?;
    let v_out = adc as f64 * VCC / ADCMAX;
    let r = v_out * R2 / (VCC - v_out);

    let t_k = 1.0 / (A + (B * log(r)) + (C * log(r) * log(r) * log(r)));
    let t_c = t_k - 273.15;

    info!("Analog value: {}", adc);
    info!("Temperature: {}", t_c);

    Ok(t_c)
}
