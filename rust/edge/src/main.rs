use axum::{extract::State, routing::post, Json, Router};
use common::{EdgeReport, Heartbeat, SensorReading};
use reqwest::{Certificate, Identity};
use std::env;
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

struct AppState {
    coord_url: String,
    edge_id: String,
    client: reqwest::Client,
    readings: Mutex<Vec<f64>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let coord_url =
        env::var("COORD_URL").unwrap_or_else(|_| "https://coordinador-service:3001".to_string());
    let edge_id = env::var("EDGE_ID").unwrap_or_else(|_| "edge-1".to_string());
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let cert_path = env::var("TLS_CERT").unwrap_or_else(|_| "/certs/edge.crt".to_string());
    let key_path = env::var("TLS_KEY").unwrap_or_else(|_| "/certs/edge.key".to_string());
    let ca_path = env::var("TLS_CA").unwrap_or_else(|_| "/certs/ca.crt".to_string());

    tracing::info!("Cargando cert={} key={} ca={}", cert_path, key_path, ca_path);

    // Identidad del cliente: cert + key concatenados en un solo PEM (lo que pide reqwest)
    let cert_pem = fs::read(&cert_path)?;
    let key_pem = fs::read(&key_path)?;
    let mut identity_pem = cert_pem.clone();
    identity_pem.push(b'\n');
    identity_pem.extend_from_slice(&key_pem);
    let identity = Identity::from_pem(&identity_pem)?;

    // CA con la que se verifica el cert del coordinador
    let ca_pem = fs::read(&ca_path)?;
    let ca_cert = Certificate::from_pem(&ca_pem)?;

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .identity(identity)
        .add_root_certificate(ca_cert)
        // No aceptes certs inválidos — eso es la mitad del valor del mTLS
        .danger_accept_invalid_certs(false)
        .build()?;

    let state = Arc::new(AppState {
        coord_url: coord_url.clone(),
        edge_id: edge_id.clone(),
        client,
        readings: Mutex::new(Vec::new()),
    });

    tracing::info!("Iniciando Edge {} (cliente mTLS) en {}", edge_id, bind_addr);
    tracing::info!("Coordinador objetivo: {}", coord_url);

    // Heartbeat task
    let hb_state = state.clone();
    tokio::spawn(async move {
        loop {
            let timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let hb = Heartbeat {
                node_id: hb_state.edge_id.clone(),
                role: "edge".to_string(),
                timestamp_ms,
            };

            let url = format!("{}/api/heartbeat", hb_state.coord_url);
            match hb_state.client.post(&url).json(&hb).send().await {
                Ok(res) => {
                    if !res.status().is_success() {
                        tracing::warn!("Coordinator rechazó heartbeat: {}", res.status());
                    }
                }
                Err(e) => {
                    tracing::error!("Error enviando heartbeat: {}", e);
                }
            }
            sleep(Duration::from_secs(5)).await;
        }
    });

    let app = Router::new()
        .route("/api/sensor", post(handle_sensor_reading))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_sensor_reading(
    State(state): State<Arc<AppState>>,
    Json(reading): Json<SensorReading>,
) {
    let mut readings = state.readings.lock().await;
    readings.push(reading.value);

    let current_count = readings.len();

    if current_count < 10 {
        tracing::info!(
            "\n---------- [ EDGE: Buffer ] ----------\n\
             | Sensor: {}\n\
             | Valor: {:.2} °C\n\
             | Guardado en buffer ({}/10 muestras)\n\
             --------------------------------------",
            reading.sensor_id,
            reading.value,
            current_count
        );
        return;
    }

    let sum: f64 = readings.iter().sum();
    let window_avg = sum / 10.0;
    readings.clear();
    drop(readings);

    tracing::info!(
        "\n========== [ EDGE: REPORTE LISTO ] ==========\n\
         | Promedio de 10 muestras calculado: {:.2} °C\n\
         | Enviando reporte consolidado al Coordinador...\n\
         =============================================",
        window_avg
    );

    let report = EdgeReport {
        edge_id: state.edge_id.clone(),
        window_avg,
        anomaly_detected: window_avg > 38.0,
        sample_count: 10,
        latency_ms: 0,
    };

    let url = format!("{}/api/edge", state.coord_url);
    match state.client.post(&url).json(&report).send().await {
        Ok(res) => {
            if !res.status().is_success() {
                tracing::warn!("Coordinator rechazó reporte: {}", res.status());
            }
        }
        Err(e) => {
            tracing::error!("Error enviando reporte al coordinator: {}", e);
        }
    }
}
