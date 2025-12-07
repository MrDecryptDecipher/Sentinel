use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::error::Error;
use log::{info, error, debug};
use std::time::Duration;
use tokio::time::sleep;

const IBM_QUANTUM_API_URL: &str = "https://api.quantum-computing.ibm.com/runtime";

#[derive(Serialize)]
struct JobParams {
    program_id: String,
    params: Value,
}

pub mod neutral_atom;

pub struct QiskitRuntimeService {
    client: Client,
    api_token: String,
    active_session: Option<String>,
}

impl QiskitRuntimeService {
    pub fn new() -> Self {
        let api_token = env::var("IBM_QUANTUM_API_TOKEN").unwrap_or_else(|_| {
            info!("QPU: 'IBM_QUANTUM_API_TOKEN' not set. Switching to DIGITAL TWIN mode.");
            "DIGITAL_TWIN_MOCK_TOKEN".to_string()
        });
        
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            client,
            api_token,
            active_session: None,
        }
    }

    /// Opens a Session (Context Context) on the IBM Quantum Backend
    pub async fn open_session(&mut self, backend_name: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/sessions", IBM_QUANTUM_API_URL);
        let body = json!({
            "backend": backend_name,
            "instance": "ibm-q/open/main"
        });

        debug!("QiskitRuntime: Opening Session on {}", backend_name);
        let resp = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            let json: Value = resp.json().await?;
            if let Some(id) = json["id"].as_str() {
                self.active_session = Some(id.to_string());
                info!("QiskitRuntime: Session Established [{}]", id);
                return Ok(());
            } else {
                error!("QiskitRuntime: Session created but ID missing");
                return Err("Missing Session ID".into());
            }
        }
        
        let err_text = resp.text().await?;
        error!("QiskitRuntime: Handshake Failed: {:?}", err_text);
        Err(format!("Session creation failed: {}", err_text).into())
    }

    /// Dispatches a 'Sampler' or 'Estimator' primitive job
    pub async fn run_job(&self, program_id: &str, theta: f64) -> Result<String, Box<dyn Error>> {
        let session_id = self.active_session.as_ref().ok_or("No active Qiskit Runtime Session")?;
        let url = format!("{}/jobs", IBM_QUANTUM_API_URL);
        
        // JIT Parameter Binding
        let params = json!({
            "market_theta": theta
        });

        // Advanced runtime options for TREX and Optimization
        let options = json!({
            "optimization_level": 3,
            "resilience_level": 1,   // TREX Enabled
            "transpilation": {
                "skip_transpilation": false
            }
        });

        let body = json!({
            "program_id": program_id,
            "session_id": session_id,
            "params": params,
            "options": options
        });

        debug!("QiskitRuntime: Dispatching Job to {}", session_id);
        let resp = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            let json: Value = resp.json().await?;
            let job_id = json["id"].as_str().unwrap_or("unknown");
            info!("QiskitRuntime: Job Submitted -> ID {}", job_id);
            Ok(job_id.to_string())
        } else {
            let err = resp.text().await?;
            error!("QiskitRuntime: Job Dispatch Error: {}", err);
            Err(format!("Job dispatch failed: {}", err).into())
        }
    }

    pub async fn close_session(&self) {
        if let Some(id) = &self.active_session {
            let url = format!("{}/sessions/{}", IBM_QUANTUM_API_URL, id);
            let _ = self.client.delete(&url)
                .header("Authorization", format!("Bearer {}", self.api_token))
                .send()
                .await;
            info!("QiskitRuntime: Session Closed [{}]", id);
        }
    }
}
