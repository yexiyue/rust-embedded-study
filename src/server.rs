use anyhow::Result;
use esp_idf_svc::{
    http::{
        server::{Configuration, EspHttpServer},
        Method,
    },
    io::{EspIOError, Write},
};
use serde_json::json;

pub fn create_server() -> Result<EspHttpServer<'static>> {
    let mut server = EspHttpServer::new(&Configuration {
        // stack_size: 1024,
        // http_port: 3000,
        ..Default::default()
    })?;

    server.fn_handler("/", Method::Get, |req| {
        let mut response = req.into_ok_response()?;
        let value = json!({
            "hello": "world",
            "temperature": 42.0,
        });
        response.write_all(serde_json::to_string_pretty(&value)?.as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(server)
}
