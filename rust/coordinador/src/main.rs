use axum::{routing::post, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use common::{EdgeReport, Heartbeat};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Provider criptográfico para rustls 0.22 (necesario antes de cualquier ServerConfig)
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("No se pudo instalar el CryptoProvider de rustls");

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".to_string());
    let cert_path = env::var("TLS_CERT").unwrap_or_else(|_| "/certs/coordinador.crt".to_string());
    let key_path = env::var("TLS_KEY").unwrap_or_else(|_| "/certs/coordinador.key".to_string());
    let ca_path = env::var("TLS_CA").unwrap_or_else(|_| "/certs/ca.crt".to_string());

    tracing::info!("Cargando cert={} key={} ca={}", cert_path, key_path, ca_path);
    let server_config = build_tls_config(&cert_path, &key_path, &ca_path)?;
    let rustls_config = RustlsConfig::from_config(Arc::new(server_config));

    tracing::info!("Iniciando Coordinador (mTLS) en https://{}", bind_addr);

    let app = Router::new()
        .route("/api/edge", post(handle_edge_report))
        .route("/api/heartbeat", post(handle_heartbeat));

    let addr: SocketAddr = bind_addr.parse()?;
    axum_server::bind_rustls(addr, rustls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn build_tls_config(
    cert_path: &str,
    key_path: &str,
    ca_path: &str,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let cert_chain = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;

    // CA con la que se verifican los certs de los clientes (edges)
    let mut ca_store = RootCertStore::empty();
    for cert in load_certs(ca_path)? {
        ca_store.add(cert)?;
    }

    // EXIGIR cert de cliente válido firmado por la CA
    let client_verifier = WebPkiClientVerifier::builder(Arc::new(ca_store)).build()?;

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(cert_chain, key)?;

    Ok(config)
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>> {
    let file = File::open(path).map_err(|e| format!("Abriendo {}: {}", path, e))?;
    let mut reader = BufReader::new(file);
    let certs: Vec<_> = certs(&mut reader).collect::<Result<_, _>>()?;
    if certs.is_empty() {
        return Err(format!("No se encontraron certificados en {}", path).into());
    }
    Ok(certs)
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>, Box<dyn std::error::Error>> {
    // Intenta PKCS8 primero
    {
        let file = File::open(path).map_err(|e| format!("Abriendo {}: {}", path, e))?;
        let mut reader = BufReader::new(file);
        if let Some(key) = pkcs8_private_keys(&mut reader).next() {
            return Ok(PrivateKeyDer::Pkcs8(key?));
        }
    }
    // Fallback a PKCS1 (RSA)
    {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        if let Some(key) = rsa_private_keys(&mut reader).next() {
            return Ok(PrivateKeyDer::Pkcs1(key?));
        }
    }
    Err(format!("No se encontró clave privada PKCS8/PKCS1 en {}", path).into())
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
    tracing::debug!(
        "Heartbeat recibido de {} ({}) en {}",
        hb.node_id,
        hb.role,
        hb.timestamp_ms
    );
}
