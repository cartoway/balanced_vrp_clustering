// Copyright © Cartoway
//
// This file is part of balanced_vrp_clustering.
//
// Cartoway balanced_vrp_clustering is free software. You can redistribute it and/or
// modify since you respect the terms of the GNU Affero General
// Public License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any later version.
//
// Cartoway Planner is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
// or FITNESS FOR A PARTICULAR PURPOSE.  See the Licenses for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Cartoway Planner. If not, see:
// <http://www.gnu.org/licenses/agpl.html>
//

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Seeded RNG for clustering (deterministic per seed).
pub struct RubyRng {
    inner: StdRng,
}

impl RubyRng {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: StdRng::seed_from_u64(seed),
        }
    }

    /// Ruby `rand` without args: float in [0, 1)
    pub fn rand_f64(&mut self) -> f64 {
        self.inner.gen::<f64>()
    }

    /// Ruby `rand(max)` for positive integer max: 0..max
    pub fn rand_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            return 0;
        }
        (self.rand_f64() * max as f64) as usize % max
    }

    pub fn rand_range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + self.rand_f64() * (max - min)
    }
}
