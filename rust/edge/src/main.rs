use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use common::{EdgeReport, Heartbeat, SensorReading};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
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

    let coord_url = env::var("COORD_URL").unwrap_or_else(|_| "http://coordinator:3001".to_string());
    let edge_id = env::var("EDGE_ID").unwrap_or_else(|_| "edge-1".to_string());
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let state = Arc::new(AppState {
        coord_url: coord_url.clone(),
        edge_id: edge_id.clone(),
        client: reqwest::Client::new(),
        readings: Mutex::new(Vec::new()),
    });

    tracing::info!("Iniciando Edge {} en {}", edge_id, bind_addr);

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
            reading.sensor_id, reading.value, current_count
        );
        return; // Esperar a tener 10 muestras
    }

    // Ya tenemos 10 muestras, calculamos el promedio
    let sum: f64 = readings.iter().sum();
    let window_avg = sum / 10.0;
    readings.clear(); // Limpiamos para las siguientes 10
    drop(readings); // Liberamos el lock del Mutex

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
