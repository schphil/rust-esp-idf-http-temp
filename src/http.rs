use std::sync::{Arc, Condvar, Mutex};

#[allow(deprecated)]
use embedded_svc::httpd::{registry::*, *};
use esp_idf_hal::{
    adc,
    adc::{AdcChannelDriver, AdcDriver, Atten11dB},
    gpio::Gpio0,
};
use esp_idf_svc::httpd as idf;

use serde_json::json;

use super::temp_sensor;

#[allow(unused_variables)]
pub fn httpd(
    mutex: Arc<(Mutex<Option<u32>>, Condvar)>,
    adc_mutex: Arc<Mutex<AdcDriver<'static, adc::ADC1>>>,
    adc_pin_mutex: Arc<Mutex<AdcChannelDriver<'static, Gpio0, Atten11dB<adc::ADC1>>>>,
) -> Result<idf::Server> {
    let adc_mutex_ = adc_mutex.clone();
    let server = idf::ServerRegistry::new().at("/").get(move |_| {
        let adc_mutex_ = adc_mutex.clone();
        let adc_pin_mutex = adc_pin_mutex.clone();
        let temp = temp_sensor::read_temp_send(adc_mutex_, adc_pin_mutex)?;

        let body = json!({
            "temperature": temp,
        });

        Response::new(200).body(body.to_string().into()).into()
    })?;

    server.start(&Default::default())
}
