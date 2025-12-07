use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomCoordinates {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RydbergPulse {
    pub duration: f64,
    pub omega: f64, // Rabi frequency
    pub delta: f64, // Detuning
    pub phase: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogHamiltonianProgram {
    pub register_name: String,
    pub atoms: Vec<AtomCoordinates>,
    pub pulses: Vec<RydbergPulse>,
}

/// Adapter for Neutral Atom Architectures (Pasqal / QuEra)
pub struct NeutralAtomAdapter {
    provider_url: String,
    api_key: String,
}

impl NeutralAtomAdapter {
    pub fn new(provider: &str, api_key: &str) -> Self {
        let url = match provider {
            "pasqal" => "https://api.pasqal.com",
            "quera" => "https://api.amazon.com/braket", // Simplified
            _ => "https://localhost:8080",
        };
        
        Self {
            provider_url: url.to_string(),
            api_key: api_key.to_string(),
        }
    }

    pub fn submit_analog_program(&self, program: &AnalogHamiltonianProgram) -> Result<String, String> {
        println!("[NeutralAtom] Submitting Analog Hamiltonian Program to {}", self.provider_url);
        println!("[NeutralAtom] Register Configuration: {} atoms", program.atoms.len());
        println!("[NeutralAtom] Pulse Sequence Length: {} steps", program.pulses.len());

        // In a real implementation, this would use reqwest to POST to the endpoint.
        // For now, we simulate the submission.
        
        if program.atoms.is_empty() {
            return Err("Atom register cannot be empty".to_string());
        }

        // Simulate Job ID return
        Ok(format!("job_{}_{}", self.provider_url, 12345))
    }

    pub fn estimate_blockade_radius(&self, rabi_freq: f64) -> f64 {
        // C6 coefficient for Rubidium-87 ~ 5420 GHz * um^6
        let c6 = 5420.0; 
        // Rb = (C6 / Omega)^(1/6)
        (c6 / rabi_freq).powf(1.0 / 6.0)
    }
}
