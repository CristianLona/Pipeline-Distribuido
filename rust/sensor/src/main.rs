use common::SensorReading;
use rand::Rng;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let edge_url = env::var("EDGE_URL").unwrap_or_else(|_| "http://edge:3000/api/sensor".to_string());
    let sensor_id = env::var("SENSOR_ID").unwrap_or_else(|_| "sensor-1".to_string());
    let interval_ms = env::var("INTERVAL_MS")
        .unwrap_or_else(|_| "2000".to_string())
        .parse::<u64>()
        .unwrap_or(2000);

    tracing::info!(
        "Iniciando Sensor {} enviando a {} cada {} ms",
        sensor_id, edge_url, interval_ms
    );

    let client = reqwest::Client::new();
    let mut rng = rand::thread_rng();

    loop {
        // Simular temperatura entre 15.0 y 40.0 °C
        let base_temp: f64 = rng.gen_range(15.0..40.0);
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;

        let reading = SensorReading {
            sensor_id: sensor_id.clone(),
            timestamp_ms,
            value: base_temp,
            unit: "C".to_string(),
        };

        tracing::info!(
            "\n========== [ SENSOR: {} ] ==========\n\
             | Enviando lectura de temperatura...\n\
             | Valor: {:.2} °C\n\
             =====================================",
            reading.sensor_id, reading.value
        );

        match client.post(&edge_url).json(&reading).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    tracing::debug!("Lectura enviada con éxito");
                } else {
                    tracing::error!("Error del servidor Edge: {}", res.status());
                }
            }
            Err(e) => {
                tracing::error!("Error de conexión al Edge: {}", e);
            }
        }

        sleep(Duration::from_millis(interval_ms)).await;
    }
}
