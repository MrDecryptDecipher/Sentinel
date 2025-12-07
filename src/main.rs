mod interop;
mod sre;
mod knowledge;
mod feed;
mod qpu;
mod ltl;
mod crypto;
mod manager;

use feed::SentinelFeed;
use qpu::QiskitRuntimeService;
use ltl::{SafetyMonitor, SentinelEvent};
use crypto::Ledger;
use interop::InteropNexus;
use sre::SentinelSRE;
use manager::QuantumManager; // Architecture Upgrade
use knowledge::QuantumKnowledge;
use dotenv::dotenv;
use tracing::{info, warn, error};
use tokio::sync::mpsc;
use serde_json::Value;

// Real-world program ID would be dynamic or loaded from config
const PROGRAM_ID: &str = "pricing_iqpe_v1";


// ... (other imports)

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    
    // ARCHITECTURE UPGRADE: Quantum Manager (Orchestrator)
    let manager = QuantumManager::new("./knowledge_data/quantum_kg.json");
    let sre = SentinelSRE::new();
    
    info!("Sentinel Hypervisor [ENTERPRISE EDITION] Active.");
    
    // ... (Heston/Feed Logic) ...
    let mut feed = SentinelFeed::new(); 
    let mut ledger = Ledger::new("sentinel_ledger.log");
    let mut monitor = SafetyMonitor::new(10); 
    let (tx, mut rx) = mpsc::channel(32);
    
    tokio::spawn(async move {
        let mut sim_feed = SentinelFeed::new();
        loop {
            let p = sim_feed.next_tick();
            if tx.send(p).await.is_err() { break; }
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    // ... (Qiskit Service) ...
    let mut qiskit_service = QiskitRuntimeService::new(); 

    let mut step = 1;
    while let Some(price) = rx.recv().await {
        
        if !sre.check_health() {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            continue;
        }

        let event = SentinelEvent::PriceUpdate(price);
        if !monitor.check(&event) {
            warn!("LTL Violation: Price {:.2}", price);
            continue; 
        }

        // Advanced Workflow
        if step % 50 == 0 {
            // 1. Quant Pricing (IQAE) - Direct Interop Call
            let vol = 0.2; 
            if let Ok(_) = InteropNexus::generate_pricing_circuit(price, 105.0, vol) {
                 info!("Quant: IQAE Pricing Complete.");
            }

            // 2. Optimization (QAOA) - Delegated to Manager (Actor)
            manager.run_optimization_cycle(step, price, &mut ledger);
        }
        
        if step % 10 == 0 {
             info!("Market Price: {:.2}", price);
        }
        step += 1;
    }
}
