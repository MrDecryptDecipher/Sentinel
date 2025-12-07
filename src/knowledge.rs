
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tracing::{info, error};

#[derive(Debug, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub relationship: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub struct QuantumKnowledge {
    pub nodes: HashMap<String, Node>,
    pub edges_by_source: HashMap<String, Vec<Edge>>,
}

impl QuantumKnowledge {
    pub fn new(path: &str) -> Option<Self> {
        info!("Loading Quantum Knowledge Graph from: {}", path);
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read KG file: {}", e);
                return None;
            }
        };

        let kg: KnowledgeGraph = match serde_json::from_str(&content) {
            Ok(k) => k,
            Err(e) => {
                error!("Failed to parse KG JSON: {}", e);
                return None;
            }
        };

        let mut nodes_map = HashMap::new();
        for node in kg.nodes {
            nodes_map.insert(node.id.clone(), node);
        }

        let mut edges_map: HashMap<String, Vec<Edge>> = HashMap::new();
        for edge in kg.edges {
            edges_map.entry(edge.source.clone()).or_default().push(edge);
        }

        info!("Knowledge Graph Loaded: {} Nodes, {} Edges", nodes_map.len(), edges_map.len());
        Some(Self {
            nodes: nodes_map,
            edges_by_source: edges_map,
        })
    }

    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn get_related(&self, id: &str) -> &[Edge] {
        self.edges_by_source.get(id).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    pub fn get_device_specs(&self, id: &str) -> Option<HashMap<String, serde_json::Value>> {
        self.nodes.get(id).map(|n| n.properties.clone())
    }

    /// INFERENCE ENGINE: Determines optimal Algorithm parameters based on Hardware Constraints
    /// Uses Knowledge Graph (EPLG) to set QAOA Depth (p)
    pub fn infer_optimal_strategy(&self, target_hw: &str) -> (String, usize) {
        let mut depth = 1; // Conservative default
        let mut strategy = "Standard-QAOA".to_string();

        if let Some(specs) = self.get_device_specs(target_hw) {
            if let Some(eplg_val) = specs.get("eplg") {
                // Parse "3.7E-3" or 0.0037
                let eplg = if let Some(s) = eplg_val.as_str() {
                     s.parse::<f64>().unwrap_or(0.01)
                } else {
                     eplg_val.as_f64().unwrap_or(0.01)
                };

                // Semantic Rule: "High Fidelity Hardware allows Deeper Circuits"
                // Thresholds derived from literature (kb)
                if eplg < 1e-3 {
                    depth = 4; // High Precision
                    strategy = "Deep-QAOA (High-Fi)".to_string();
                } else if eplg < 5e-3 {
                    depth = 2; // Balanced (IBM Heron range: 3.7e-3)
                    strategy = "Balanced-QAOA".to_string();
                } else {
                    depth = 1; // NISQ Safe
                    strategy = "Shallow-QAOA (NISQ)".to_string();
                }
            }
        }
        (strategy, depth)
    }

    pub fn describe_algorithm(&self, algo_id: &str) -> String {
        if let Some(node) = self.nodes.get(algo_id) {
            // ... (rest of function)
            let mut desc = format!("Algorithm: {} ({})\n", node.label, node.node_type);
            if let Some(d) = node.properties.get("description") {
                desc.push_str(&format!("  Description: {}\n", d));
            }
            if let Some(s) = node.properties.get("speedup") {
                desc.push_str(&format!("  Speedup: {}\n", s));
            }
            
            // Find related
            if let Some(edges) = self.edges_by_source.get(algo_id) {
                desc.push_str("  Context:\n");
                for edge in edges {
                     if let Some(target_node) = self.nodes.get(&edge.target) {
                         desc.push_str(&format!("    --[{}]--> {} ({})\n", edge.relationship, target_node.label, target_node.node_type));
                     }
                }
            }
            return desc;
        }
        format!("Algorithm {} not found in Knowledge Graph.", algo_id)
    }
}
