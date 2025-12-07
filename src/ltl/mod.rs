use log::{warn, error, info};

// Abstract Event enum
#[derive(Debug, Clone, PartialEq)]
pub enum SentinelEvent {
    PriceUpdate(f64),
    HedgeExecuted,
    QuantumJobFinished,
}

// Property: [](Price < 100 -> <>(Hedge))
// Transformed to Monitor:
// State 0: Normal. If Price < 100 -> State 1 (Obligation).
// State 1: Obligation. If Hedge -> State 0.
// If we persist in State 1 too long, it's a "violation" in a practical sense (liveness property).

#[derive(Debug)]
pub enum MonitorState {
    Safe,
    PendingHedge(u64), // Ticks since obligation started
}

pub struct SafetyMonitor {
    state: MonitorState,
    max_ticks_tolerance: u64,
}

impl SafetyMonitor {
    pub fn new(tolerance: u64) -> Self {
        Self {
            state: MonitorState::Safe,
            max_ticks_tolerance: tolerance,
        }
    }

    pub fn check(&mut self, event: &SentinelEvent) -> bool {
        match &mut self.state {
            MonitorState::Safe => {
                if let SentinelEvent::PriceUpdate(price) = event {
                    if *price < 100.0 {
                        warn!("LTL Monitor: Violation of Precondition (Price < 100). Entering Obligation State.");
                        self.state = MonitorState::PendingHedge(0);
                    }
                }
            }
            MonitorState::PendingHedge(ticks) => {
                if let SentinelEvent::HedgeExecuted = event {
                    info!("LTL Monitor: Obligation Met (Hedge). Returning to Safe State.");
                    self.state = MonitorState::Safe;
                } else {
                    *ticks += 1;
                    if *ticks > self.max_ticks_tolerance {
                        error!("LTL Monitor: SAFETY VIOLATION! Expected Hedge within {} ticks.", self.max_ticks_tolerance);
                        return false; // Hardware Interrupt Trigger
                    }
                }
            }
        }
        true
    }
}
