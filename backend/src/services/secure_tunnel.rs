use actix_web::{web, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock};
use tokio_rustls::{TlsAcceptor, TlsConnector};
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;
use std::fs::File;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub listen_addr: String,
    pub cert_path: String,
    pub key_path: String,
    pub max_connections: usize,
    pub connection_timeout_seconds: u64,
    pub enable_encryption: bool,
    pub enable_compression: bool,
    pub allowed_origins: Vec<String>,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8443".to_string(),
            cert_path: "./certs/server.crt".to_string(),
            key_path: "./certs/server.key".to_string(),
            max_connections: 1000,
            connection_timeout_seconds: 300,
            enable_encryption: true,
            enable_compression: true,
            allowed_origins: vec!["https://localhost:3000".to_string()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct TunnelConnection {
    pub id: String,
    pub client_addr: SocketAddr,
    pub established_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_encrypted: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMessage {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub payload: String,
    pub checksum: String,
    pub encrypted: bool,
    pub compressed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelMetrics {
    pub active_connections: usize,
    pub total_connections: u64,
    pub bytes_transferred: u64,
    pub messages_processed: u64,
    pub encryption_overhead_ms: f64,
    pub compression_ratio: f64,
    pub uptime_seconds: u64,
}

pub struct SecureTunnelService {
    config: TunnelConfig,
    connections: Arc<RwLock<HashMap<String, TunnelConnection>>>,
    tls_acceptor: Option<TlsAcceptor>,
    aes_key: Key<Aes256Gcm>,
    metrics: Arc<Mutex<TunnelMetrics>>,
    started_at: DateTime<Utc>,
}

impl SecureTunnelService {
    pub async fn new(config: TunnelConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize TLS if certificates are available
        let tls_acceptor = if std::path::Path::new(&config.cert_path).exists() && 
                            std::path::Path::new(&config.key_path).exists() {
            Some(Self::create_tls_acceptor(&config.cert_path, &config.key_path).await?)
        } else {
            log::warn!("TLS certificates not found, running without TLS");
            None
        };

        // Generate AES key for message encryption
        let aes_key = Aes256Gcm::generate_key(&mut rand::thread_rng());

        let metrics = TunnelMetrics {
            active_connections: 0,
            total_connections: 0,
            bytes_transferred: 0,
            messages_processed: 0,
            encryption_overhead_ms: 0.0,
            compression_ratio: 1.0,
            uptime_seconds: 0,
        };

        Ok(Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            tls_acceptor,
            aes_key,
            metrics: Arc::new(Mutex::new(metrics)),
            started_at: Utc::now(),
        })
    }

    async fn create_tls_acceptor(cert_path: &str, key_path: &str) -> Result<TlsAcceptor, Box<dyn std::error::Error>> {
        // Load certificates
        let cert_file = File::open(cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = certs(&mut cert_reader)?
            .into_iter()
            .map(Certificate)
            .collect();

        // Load private key
        let key_file = File::open(key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let mut keys = pkcs8_private_keys(&mut key_reader)?;
        
        if keys.is_empty() {
            return Err("No private keys found in key file".into());
        }
        
        let key = PrivateKey(keys.remove(0));

        // Create TLS configuration
        let tls_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        Ok(TlsAcceptor::from(Arc::new(tls_config)))
    }

    /// Start the secure tunnel server
    pub async fn start_server(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.config.listen_addr).await?;
        log::info!("Secure tunnel server listening on {}", self.config.listen_addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let connection_id = Uuid::new_v4().to_string();
                    
                    // Check connection limit
                    {
                        let connections = self.connections.read().await;
                        if connections.len() >= self.config.max_connections {
                            log::warn!("Connection limit reached, rejecting connection from {}", addr);
                            continue;
                        }
                    }

                    // Create new connection
                    let connection = TunnelConnection {
                        id: connection_id.clone(),
                        client_addr: addr,
                        established_at: Utc::now(),
                        last_activity: Utc::now(),
                        bytes_sent: 0,
                        bytes_received: 0,
                        is_encrypted: self.tls_acceptor.is_some(),
                        is_active: true,
                    };

                    // Store connection
                    {
                        let mut connections = self.connections.write().await;
                        connections.insert(connection_id.clone(), connection);
                    }

                    // Update metrics
                    {
                        let mut metrics = self.metrics.lock().await;
                        metrics.total_connections += 1;
                        metrics.active_connections += 1;
                    }

                    // Handle connection
                    let tunnel_service = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = tunnel_service.handle_connection(stream, connection_id).await {
                            log::error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    log::error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_connection(&self, stream: TcpStream, connection_id: String) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Handling connection: {}", connection_id);

        // Apply TLS if available
        let mut stream: Box<dyn tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> = if let Some(ref acceptor) = self.tls_acceptor {
            match acceptor.accept(stream).await {
                Ok(tls_stream) => Box::new(tls_stream),
                Err(e) => {
                    log::error!("TLS handshake failed: {}", e);
                    self.remove_connection(&connection_id).await;
                    return Err(e.into());
                }
            }
        } else {
            Box::new(stream)
        };

        // Connection handling logic would go here
        // This is a simplified implementation
        
        // Simulate some processing
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Clean up connection
        self.remove_connection(&connection_id).await;
        
        Ok(())
    }

    async fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if connections.remove(connection_id).is_some() {
            let mut metrics = self.metrics.lock().await;
            metrics.active_connections = metrics.active_connections.saturating_sub(1);
        }
    }

    /// Encrypt message payload
    pub fn encrypt_message(&self, message: &str) -> Result<SecureMessage, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        if !self.config.enable_encryption {
            return Ok(SecureMessage {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                payload: message.to_string(),
                checksum: self.calculate_checksum(message),
                encrypted: false,
                compressed: false,
            });
        }

        let cipher = Aes256Gcm::new(&self.aes_key);
        let nonce = Aes256Gcm::generate_nonce(&mut rand::thread_rng());
        
        // Compress if enabled
        let data_to_encrypt = if self.config.enable_compression {
            self.compress_data(message.as_bytes())?
        } else {
            message.as_bytes().to_vec()
        };
        
        let ciphertext = cipher.encrypt(&nonce, data_to_encrypt.as_slice())?;
        
        // Combine nonce and ciphertext
        let mut encrypted_data = nonce.to_vec();
        encrypted_data.extend_from_slice(&ciphertext);
        
        let encrypted_payload = general_purpose::STANDARD.encode(&encrypted_data);
        
        let encryption_time = start_time.elapsed();
        
        // Update metrics
        tokio::spawn({
            let metrics = Arc::clone(&self.metrics);
            async move {
                let mut m = metrics.lock().await;
                m.encryption_overhead_ms = (m.encryption_overhead_ms + encryption_time.as_millis() as f64) / 2.0;
            }
        });

        Ok(SecureMessage {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            payload: encrypted_payload,
            checksum: self.calculate_checksum(&encrypted_payload),
            encrypted: true,
            compressed: self.config.enable_compression,
        })
    }

    /// Decrypt message payload
    pub fn decrypt_message(&self, secure_message: &SecureMessage) -> Result<String, Box<dyn std::error::Error>> {
        if !secure_message.encrypted {
            return Ok(secure_message.payload.clone());
        }

        // Verify checksum
        if self.calculate_checksum(&secure_message.payload) != secure_message.checksum {
            return Err("Message integrity check failed".into());
        }

        let encrypted_data = general_purpose::STANDARD.decode(&secure_message.payload)?;
        
        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted message format".into());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.aes_key);
        let decrypted_data = cipher.decrypt(nonce, ciphertext)?;
        
        // Decompress if needed
        let final_data = if secure_message.compressed {
            self.decompress_data(&decrypted_data)?
        } else {
            decrypted_data
        };
        
        Ok(String::from_utf8(final_data)?)
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn calculate_checksum(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get current tunnel metrics
    pub async fn get_metrics(&self) -> TunnelMetrics {
        let mut metrics = self.metrics.lock().await;
        metrics.uptime_seconds = (Utc::now() - self.started_at).num_seconds() as u64;
        metrics.clone()
    }

    /// Get active connections
    pub async fn get_connections(&self) -> Vec<TunnelConnection> {
        let connections = self.connections.read().await;
        connections.values().cloned().collect()
    }

    /// Close specific connection
    pub async fn close_connection(&self, connection_id: &str) -> bool {
        let mut connections = self.connections.write().await;
        if let Some(mut connection) = connections.get_mut(connection_id) {
            connection.is_active = false;
            true
        } else {
            false
        }
    }

    /// Clean up inactive connections
    pub async fn cleanup_inactive_connections(&self) {
        let cutoff = Utc::now() - Duration::seconds(self.config.connection_timeout_seconds as i64);
        let mut connections = self.connections.write().await;
        let initial_count = connections.len();
        
        connections.retain(|_, conn| {
            conn.is_active && conn.last_activity > cutoff
        });
        
        let cleaned_count = initial_count - connections.len();
        if cleaned_count > 0 {
            log::info!("Cleaned up {} inactive connections", cleaned_count);
            
            // Update metrics
            let metrics_arc = Arc::clone(&self.metrics);
            tokio::spawn(async move {
                let mut metrics = metrics_arc.lock().await;
                metrics.active_connections = metrics.active_connections.saturating_sub(cleaned_count);
            });
        }
    }

    /// Validate client origin
    pub fn validate_origin(&self, origin: &str) -> bool {
        if self.config.allowed_origins.is_empty() {
            return true; // Allow all if no restrictions
        }
        
        self.config.allowed_origins.iter().any(|allowed| {
            origin == allowed || (allowed.ends_with("*") && origin.starts_with(&allowed[..allowed.len()-1]))
        })
    }

    /// Create secure WebSocket upgrade response
    pub fn create_websocket_response(&self, key: &str) -> String {
        use sha1::{Digest, Sha1};
        const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        
        let mut hasher = Sha1::new();
        hasher.update(key.as_bytes());
        hasher.update(WS_GUID.as_bytes());
        general_purpose::STANDARD.encode(hasher.finalize())
    }
}

impl Clone for SecureTunnelService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            connections: Arc::clone(&self.connections),
            tls_acceptor: self.tls_acceptor.clone(),
            aes_key: self.aes_key.clone(),
            metrics: Arc::clone(&self.metrics),
            started_at: self.started_at,
        }
    }
}

/// Background maintenance task for tunnel service
pub async fn tunnel_maintenance_task(tunnel_service: Arc<SecureTunnelService>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Every minute
    
    loop {
        interval.tick().await;
        
        // Clean up inactive connections
        tunnel_service.cleanup_inactive_connections().await;
        
        // Log metrics
        let metrics = tunnel_service.get_metrics().await;
        log::info!(
            "Tunnel metrics - Active: {}, Total: {}, Bytes: {}, Messages: {}, Encryption overhead: {:.2}ms",
            metrics.active_connections,
            metrics.total_connections,
            metrics.bytes_transferred,
            metrics.messages_processed,
            metrics.encryption_overhead_ms
        );
    }
}

/// HTTP handler for tunnel status
pub async fn tunnel_status_handler(
    tunnel_service: web::Data<Arc<SecureTunnelService>>,
) -> ActixResult<HttpResponse> {
    let metrics = tunnel_service.get_metrics().await;
    Ok(HttpResponse::Ok().json(metrics))
}

/// HTTP handler for active connections
pub async fn connections_handler(
    tunnel_service: web::Data<Arc<SecureTunnelService>>,
) -> ActixResult<HttpResponse> {
    let connections = tunnel_service.get_connections().await;
    Ok(HttpResponse::Ok().json(connections))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_encryption() {
        let config = TunnelConfig::default();
        let tunnel_service = SecureTunnelService::new(config).await.unwrap();
        
        let original_message = "This is a secret message";
        let encrypted_message = tunnel_service.encrypt_message(original_message).unwrap();
        let decrypted_message = tunnel_service.decrypt_message(&encrypted_message).unwrap();
        
        assert_eq!(original_message, decrypted_message);
        assert!(encrypted_message.encrypted);
    }

    #[tokio::test]
    async fn test_tunnel_metrics() {
        let config = TunnelConfig::default();
        let tunnel_service = SecureTunnelService::new(config).await.unwrap();
        
        let metrics = tunnel_service.get_metrics().await;
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.total_connections, 0);
    }

    #[test]
    fn test_origin_validation() {
        let mut config = TunnelConfig::default();
        config.allowed_origins = vec![
            "https://example.com".to_string(),
            "https://*.test.com".to_string(),
        ];
        
        let tunnel_service = tokio_test::block_on(SecureTunnelService::new(config)).unwrap();
        
        assert!(tunnel_service.validate_origin("https://example.com"));
        assert!(tunnel_service.validate_origin("https://api.test.com"));
        assert!(!tunnel_service.validate_origin("https://malicious.com"));
    }
}