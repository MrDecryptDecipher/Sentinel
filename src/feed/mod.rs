use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;
use serde::Deserialize;
use simd_json;
use std::env;
use log::{info, error, warn, debug};
use rand_distr::{Distribution, Normal};
use rand::thread_rng;

// HESTON MODEL IMPLEMENTATION
pub struct SentinelFeed {
    // Heston Model Parameters
    s0: f64,      // Initial Price
    v0: f64,      // Initial Volatility
    kappa: f64,   // Mean reversion speed
    theta: f64,   // Long run variance
    xi: f64,      // Vol of Vol
    rho: f64,     // Correlation
    dt: f64,      // Time step
    
    current_price: f64,
    current_vol: f64,
}

impl SentinelFeed {
    pub fn new() -> Self {
        Self {
            s0: 100.0,
            v0: 0.04,
            kappa: 2.0,
            theta: 0.04,
            xi: 0.1,
            rho: -0.7, // Leverage effect
            dt: 1.0/252.0, // Daily step
            current_price: 100.0,
            current_vol: 0.04,
        }
    }
    
    /// Simulates one step of Heston Stochastic Volatility Model
    /// dS_t = mu*S_t*dt + sqrt(v_t)*S_t*dW_t^S
    /// dv_t = kappa*(theta - v_t)*dt + xi*sqrt(v_t)*dW_t^v
    pub fn next_tick(&mut self) -> f64 {
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 1.0).unwrap();
        
        // Correlated Brownian Motions
        let z1 = normal.sample(&mut rng);
        let z2 = self.rho * z1 + (1.0 - self.rho.powi(2)).sqrt() * normal.sample(&mut rng);
        
        // Volatility Process (CIR) - Full Interaction
        let dv = self.kappa * (self.theta - self.current_vol) * self.dt 
                 + self.xi * self.current_vol.sqrt() * z2 * self.dt.sqrt();
        
        self.current_vol = (self.current_vol + dv).max(0.001); // Ensure positivity
        
        // Price Process
        let drift = 0.05; // 5% risk-free assumption
        let ds = drift * self.current_price * self.dt 
                 + self.current_vol.sqrt() * self.current_price * z1 * self.dt.sqrt();
                 
        self.current_price += ds;
        
        debug!("HESTON: Price={:.2}, Vol={:.4}", self.current_price, self.current_vol);
        self.current_price
    }
}
