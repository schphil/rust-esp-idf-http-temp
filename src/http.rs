use std::{
    sync::{ Arc, Condvar, Mutex},
};

#[allow(deprecated)]
use embedded_svc::httpd::{registry::*, *};
use esp_idf_svc::{
    httpd as idf,
};

#[allow(unused_variables)]
#[cfg(not(feature = "experimental"))]
pub fn httpd(mutex: Arc<(Mutex<Option<u32>>, Condvar)>) -> Result<idf::Server> {
    let server = idf::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello from Rust!".into()))?;

    server.start(&Default::default())
}
