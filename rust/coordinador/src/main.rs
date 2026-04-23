use axum::{
    routing::post,
    Json, Router,
};
use common::{EdgeReport, Heartbeat};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());

    tracing::info!("Iniciando Coordinador en {}", bind_addr);

    let app = Router::new()
        .route("/api/edge", post(handle_edge_report))
        .route("/api/heartbeat", post(handle_heartbeat));

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_edge_report(Json(report): Json<EdgeReport>) {
    tracing::info!(
        "\n========== [ COORDINADOR ] ==========\n\
         | Reporte recibido de: {}\n\
         | Promedio Muestras: {:.2} °C\n\
         | Número de muestras: {}\n\
         | Anomalía Detectada: {}\n\
         =====================================",
        report.edge_id, report.window_avg, report.sample_count, report.anomaly_detected
    );
    
    if report.anomaly_detected {
        tracing::warn!(
            "\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\
             ! ALERTA ROJA: Anomalía detectada   !\n\
             ! Origen: {:<25} !\n\
             !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!", 
            report.edge_id
        );
    }
}

async fn handle_heartbeat(Json(hb): Json<Heartbeat>) {
    tracing::debug!("Heartbeat recibido de {} ({}) en {}", hb.node_id, hb.role, hb.timestamp_ms);
}
