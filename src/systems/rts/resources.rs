//! Rise of Nations-style economy.
//!
//! Six resources gathered as continuous *rates* (not depleting piles):
//! Food, Timber, Metal, Wealth, Knowledge, Oil. Income per resource is
//! clamped by a commerce cap that rises with each age.

use serde::{Deserialize, Serialize};

/// The six national resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    Food,
    Timber,
    Metal,
    Wealth,
    Knowledge,
    Oil,
}

impl Resource {
    pub const ALL: [Resource; 6] = [
        Resource::Food,
        Resource::Timber,
        Resource::Metal,
        Resource::Wealth,
        Resource::Knowledge,
        Resource::Oil,
    ];

    pub fn index(&self) -> usize {
        match self {
            Resource::Food => 0,
            Resource::Timber => 1,
            Resource::Metal => 2,
            Resource::Wealth => 3,
            Resource::Knowledge => 4,
            Resource::Oil => 5,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Resource::Food => "Food",
            Resource::Timber => "Timber",
            Resource::Metal => "Metal",
            Resource::Wealth => "Wealth",
            Resource::Knowledge => "Knowledge",
            Resource::Oil => "Oil",
        }
    }

    /// One-letter tag used in compact cost strings ("120T 40W").
    pub fn tag(&self) -> &'static str {
        match self {
            Resource::Food => "F",
            Resource::Timber => "T",
            Resource::Metal => "M",
            Resource::Wealth => "W",
            Resource::Knowledge => "K",
            Resource::Oil => "O",
        }
    }
}

/// A price expressed across the six resources.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Cost(pub [f32; 6]);

impl Cost {
    pub fn food(mut self, v: f32) -> Self {
        self.0[0] = v;
        self
    }
    pub fn timber(mut self, v: f32) -> Self {
        self.0[1] = v;
        self
    }
    pub fn metal(mut self, v: f32) -> Self {
        self.0[2] = v;
        self
    }
    pub fn wealth(mut self, v: f32) -> Self {
        self.0[3] = v;
        self
    }
    pub fn knowledge(mut self, v: f32) -> Self {
        self.0[4] = v;
        self
    }
    pub fn oil(mut self, v: f32) -> Self {
        self.0[5] = v;
        self
    }

    /// Scale every component (used for Rise of Nations-style ramping costs).
    pub fn scaled(&self, mult: f32) -> Self {
        let mut out = *self;
        for v in &mut out.0 {
            *v = (*v * mult).round();
        }
        out
    }

    /// Compact human string like "60F 20T".
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        for r in Resource::ALL {
            let v = self.0[r.index()];
            if v > 0.0 {
                parts.push(format!("{}{}", v as i64, r.tag()));
            }
        }
        if parts.is_empty() {
            "free".to_string()
        } else {
            parts.join(" ")
        }
    }
}

/// A nation's stockpile of the six resources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stockpile {
    amounts: [f32; 6],
}

impl Stockpile {
    pub fn starting() -> Self {
        let mut s = Self::default();
        s.amounts[Resource::Food.index()] = 250.0;
        s.amounts[Resource::Timber.index()] = 250.0;
        s.amounts[Resource::Wealth.index()] = 100.0;
        s
    }

    pub fn get(&self, r: Resource) -> f32 {
        self.amounts[r.index()]
    }

    pub fn add(&mut self, r: Resource, amount: f32) {
        self.amounts[r.index()] += amount;
    }

    pub fn can_afford(&self, cost: &Cost) -> bool {
        self.amounts
            .iter()
            .zip(cost.0.iter())
            .all(|(have, need)| have >= need)
    }

    /// Deduct the cost if affordable; returns whether payment happened.
    pub fn pay(&mut self, cost: &Cost) -> bool {
        if !self.can_afford(cost) {
            return false;
        }
        for (have, need) in self.amounts.iter_mut().zip(cost.0.iter()) {
            *have -= need;
        }
        true
    }

    /// Refund a cost (cancelled production).
    pub fn refund(&mut self, cost: &Cost) {
        for (have, give) in self.amounts.iter_mut().zip(cost.0.iter()) {
            *have += give;
        }
    }
}

/// Maximum income per resource per second at a given age (0-based index).
/// The Rise of Nations commerce cap: economies scale with the ages, not
/// with how many workers you can pile onto one resource.
pub fn commerce_cap(age: usize) -> f32 {
    8.0 + 4.0 * age as f32
}
