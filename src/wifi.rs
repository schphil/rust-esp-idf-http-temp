use std::time::Duration;

#[allow(deprecated)]
use embedded_svc::ipv4;
use embedded_svc::wifi::*;
use esp_idf_hal::peripheral;
use esp_idf_svc::{eventloop::*, netif::*, ping, wifi::*};
use log::*;

use super::error::Error;

#[cfg(not(feature = "qemu"))]
#[allow(dead_code)]
pub fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>, Error> {
    use std::net::Ipv4Addr;

    // use esp_idf_svc::handle::RawHandle;

    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

    let ssid = dotenv::var("RUST_ESP32_STD_DEMO_WIFI_SSID").expect("RUST_ESP32_STD_DEMO_WIFI_SSID muste be set");
    let pass = dotenv::var("RUST_ESP32_STD_DEMO_WIFI_PASS").expect("RUST_ESP32_STD_DEMO_WIFI_PASS muste be set");

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid.as_str());

    let channel = if let Some(ours) = ours {
        info!("Found configured access point {} on channel {}", ssid, ours.channel);
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: ssid.as_str().into(),
            password: pass.as_str().into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.start()?;

    info!("Starting wifi...");

    if !WifiWait::new(&sysloop)?.wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap()) {
        error!("Wifi did not start");
    }

    info!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(Duration::from_secs(20), || {
        wifi.is_connected().unwrap() && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
    }) {
        error!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    ping(ip_info.subnet.gateway)?;

    Ok(wifi)
}

fn ping(ip: ipv4::Ipv4Addr) -> Result<(), Error> {
    info!("About to do some pings for {:?}", ip);

    let ping_summary = ping::EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        error!("Pinging IP {} resulted in timeouts", ip)
    }

    info!("Pinging done");

    Ok(())
}
