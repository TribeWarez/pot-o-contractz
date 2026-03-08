use anchor_lang::prelude::*;

/// Vertex in the tensor network representing a miner or mining pool.
#[derive(Clone, Copy)]
pub struct PoolVertex {
    pub id: u32,
    pub miner: Pubkey,
    pub device_type: u8,
    pub entropy: u64,   // S_A in 1e6 scale
    pub coherence: u64, // Device coherence in 1e6 scale
}

/// Edge representing entanglement between two pool vertices.
#[derive(Clone, Copy)]
pub struct PoolEdge {
    pub from_id: u32,
    pub to_id: u32,
    pub mutual_info: u64, // I(A:B) in 1e6 scale
    pub cut_size: u32,    // Number of bonds crossing this edge
}

/// Service for managing tensor network state and entropy calculations.
///
/// Based on REALMS Part IV formulas:
/// - Entropy: S = |γ| log(d) where γ = edges crossing cut, d = bond dimension
/// - Mutual Information: I(A:B) = S(A) + S(B) - S(A∪B)
/// - Effective Distance: d_eff = 1 - I(A:B) / S_max
/// - Staking Probability: P(unlock) = tanh(S_A / S_max)
pub trait TensorPoolService {
    /// Add a new vertex (miner) to the tensor network.
    fn add_vertex(&mut self, vertex: PoolVertex) -> Result<u32>;

    /// Add an edge (entanglement) between two vertices.
    fn add_edge(&mut self, edge: PoolEdge) -> Result<()>;

    /// Calculate entropy for a vertex given its degree (number of connections).
    /// Formula: S = |γ| log(d) where γ = degree, d = bond_dimension
    fn calculate_vertex_entropy(&self, degree: u32, bond_dimension: u32) -> u64;

    /// Calculate mutual information between two vertices.
    /// Formula: I(A:B) = S(A) + S(B) - S(A∪B)
    fn calculate_mutual_information(
        &self,
        entropy_a: u64,
        entropy_b: u64,
        entropy_union: u64,
    ) -> u64;

    /// Calculate effective distance between two vertices.
    /// Formula: d_eff = 1 - I(A:B) / S_max
    fn calculate_effective_distance(&self, mutual_info: u64, s_max: u64) -> u64;

    /// Calculate coherence probability (unlock probability for staking).
    /// Formula: P(unlock) = tanh(S_A / S_max)
    /// Approximation: P ≈ 0.5 * tanh_approx(S_A / S_max)
    fn calculate_coherence_probability(&self, entropy: u64, s_max: u64) -> u64;

    /// Get total network entropy (sum of all vertex entropies).
    fn total_entropy(&self) -> u64;

    /// Get current number of vertices in the network.
    fn vertex_count(&self) -> u32;

    /// Get current number of edges in the network.
    fn edge_count(&self) -> u32;

    /// Check if network is at maximum capacity.
    fn is_full(&self, max_capacity: u32) -> bool;
}

/// Standard implementation of TensorPoolService.
///
/// Maintains in-memory graph structure with vertices and edges.
/// Suitable for computation within instruction context.
#[allow(dead_code)]
pub struct StandardTensorPool {
    vertices: Vec<PoolVertex>,
    edges: Vec<PoolEdge>,
    s_max: u64,
    bond_dimension: u32,
}

impl StandardTensorPool {
    pub fn new(s_max: u64, bond_dimension: u32) -> Self {
        StandardTensorPool {
            vertices: Vec::new(),
            edges: Vec::new(),
            s_max,
            bond_dimension,
        }
    }

    /// Log2 approximation using integer arithmetic.
    /// Returns log2(x) * 1_000_000 (1e6 scale)
    fn log2_fixed(x: u32) -> u64 {
        if x == 0 {
            return 0;
        }
        if x == 1 {
            return 0;
        }

        let mut power = 1u32;
        let mut log_val = 0u64;

        while power < x {
            power = power.saturating_mul(2);
            log_val += 1_000_000u64;
        }

        // Refine with binary search (simple approximation)
        let mut result = log_val;

        // Adjust for fractional part
        if power > x {
            result = result.saturating_sub(500_000); // Half a bit
        }

        result
    }

    /// Hyperbolic tangent approximation for tanh(x).
    /// Input and output in 1e6 scale.
    /// Uses polynomial approximation for better accuracy.
    fn tanh_approx(x: u64) -> u64 {
        // tanh(x) ≈ (e^(2x) - 1) / (e^(2x) + 1)
        // For fixed-point: tanh(x) ≈ x / (1 + |x|/2) for small x
        // For large x: tanh(x) → ±1

        const ONE: u64 = 1_000_000; // 1.0 in 1e6 scale

        if x >= 3 * ONE {
            return ONE; // tanh(3) ≈ 0.995, clamp to 1.0
        }
        if x == 0 {
            return 0;
        }

        // Polynomial approximation: tanh(x) ≈ x - x³/3 + 2x⁵/15
        let x_cubed = (x / ONE) * (x / ONE) * (x / ONE) / ONE;
        let x_fifth = x_cubed * (x / ONE) * (x / ONE) / ONE;

        x.saturating_sub(x_cubed / 3)
            .saturating_add(2 * x_fifth / 15)
    }
}

impl TensorPoolService for StandardTensorPool {
    fn add_vertex(&mut self, vertex: PoolVertex) -> Result<u32> {
        let id = self.vertices.len() as u32;
        self.vertices.push(vertex);
        Ok(id)
    }

    fn add_edge(&mut self, edge: PoolEdge) -> Result<()> {
        self.edges.push(edge);
        Ok(())
    }

    fn calculate_vertex_entropy(&self, degree: u32, bond_dimension: u32) -> u64 {
        // S = |γ| log(d)
        // degree = |γ| (number of edges crossing the cut)
        // bond_dimension = d
        let bond_dim = bond_dimension.max(2); // At least 2
        let log_d = Self::log2_fixed(bond_dim);
        ((degree as u64) * log_d) / 1_000_000
    }

    fn calculate_mutual_information(
        &self,
        entropy_a: u64,
        entropy_b: u64,
        entropy_union: u64,
    ) -> u64 {
        // I(A:B) = S(A) + S(B) - S(A∪B)
        entropy_a
            .saturating_add(entropy_b)
            .saturating_sub(entropy_union)
    }

    fn calculate_effective_distance(&self, mutual_info: u64, s_max: u64) -> u64 {
        // d_eff = 1 - I(A:B) / S_max
        if mutual_info >= s_max {
            return 0; // d_eff = 0 (perfect correlation)
        }
        ((s_max - mutual_info) * 1_000_000) / s_max
    }

    fn calculate_coherence_probability(&self, entropy: u64, s_max: u64) -> u64 {
        // P(unlock) = tanh(S_A / S_max)
        // Scale: S_A / S_max is in [0, 1] in 1e6 scale
        let normalized = (entropy * 1_000_000) / s_max.max(1);
        Self::tanh_approx(normalized)
    }

    fn total_entropy(&self) -> u64 {
        self.vertices.iter().map(|v| v.entropy).sum()
    }

    fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn edge_count(&self) -> u32 {
        self.edges.len() as u32
    }

    fn is_full(&self, max_capacity: u32) -> bool {
        self.vertices.len() as u32 >= max_capacity
    }
}

/// Mock tensor pool service for testing.
#[cfg(test)]
pub struct MockTensorPool {
    vertices: u32,
    edges: u32,
    total_entropy: u64,
}

#[cfg(test)]
impl MockTensorPool {
    pub fn new() -> Self {
        MockTensorPool {
            vertices: 0,
            edges: 0,
            total_entropy: 0,
        }
    }

    pub fn with_vertices(mut self, count: u32) -> Self {
        self.vertices = count;
        self
    }

    pub fn with_entropy(mut self, entropy: u64) -> Self {
        self.total_entropy = entropy;
        self
    }
}

#[cfg(test)]
impl TensorPoolService for MockTensorPool {
    fn add_vertex(&mut self, _vertex: PoolVertex) -> Result<u32> {
        self.vertices += 1;
        Ok(self.vertices - 1)
    }

    fn add_edge(&mut self, _edge: PoolEdge) -> Result<()> {
        self.edges += 1;
        Ok(())
    }

    fn calculate_vertex_entropy(&self, degree: u32, bond_dimension: u32) -> u64 {
        (degree as u64) * (bond_dimension as u64)
    }

    fn calculate_mutual_information(
        &self,
        entropy_a: u64,
        entropy_b: u64,
        entropy_union: u64,
    ) -> u64 {
        entropy_a
            .saturating_add(entropy_b)
            .saturating_sub(entropy_union)
    }

    fn calculate_effective_distance(&self, _mutual_info: u64, _s_max: u64) -> u64 {
        500_000 // Mock: 0.5
    }

    fn calculate_coherence_probability(&self, entropy: u64, s_max: u64) -> u64 {
        (entropy * 1_000_000) / s_max.max(1)
    }

    fn total_entropy(&self) -> u64 {
        self.total_entropy
    }

    fn vertex_count(&self) -> u32 {
        self.vertices
    }

    fn edge_count(&self) -> u32 {
        self.edges
    }

    fn is_full(&self, max_capacity: u32) -> bool {
        self.vertices >= max_capacity
    }
}
