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

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

fn deserialize_usize_from_number<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Num {
        Int(usize),
        Float(f64),
    }
    match Num::deserialize(deserializer)? {
        Num::Int(v) => Ok(v),
        Num::Float(v) => Ok(v as usize),
    }
}

pub const INCOMPATIBILITY_DISTANCE_PENALTY: f64 = 4_294_967_296.0; // 2^32

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InputConfig {
    pub name: Option<String>,
    pub seed: u64,
    pub cut_symbol: Option<String>,
    pub cut_ratio: f64,
    #[serde(deserialize_with = "deserialize_usize_from_number")]
    pub max_iterations: usize,
    pub vehicles: Vec<VehicleInput>,
    pub items: Vec<ItemInput>,
    pub distance_matrix: Option<Vec<Vec<f64>>>,
    pub centroid_indices: Vec<usize>,
    #[serde(default)]
    pub related_item_indices: HashMap<String, Vec<Vec<usize>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VehicleInput {
    pub id: Vec<String>,
    pub depot: DepotInput,
    #[serde(default)]
    pub capacities: HashMap<String, f64>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub day_skills: Vec<String>,
    #[serde(default)]
    pub duration: f64,
    #[serde(default = "default_one")]
    pub total_work_days: f64,
    #[serde(default = "default_one")]
    pub vehicle_count: f64,
}

fn default_one() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DepotInput {
    pub coordinates: Option<Vec<f64>>,
    pub matrix_index: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemInput {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub quantities: HashMap<String, f64>,
    #[serde(default)]
    pub v_id: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub day_skills: Vec<String>,
    pub matrix_index: Option<usize>,
    #[serde(default)]
    pub duration_from_and_to_depot: Vec<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildOutput {
    pub iterations: usize,
    pub clusters: Vec<Vec<String>>,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CentroidWeights {
    pub limit: Vec<f64>,
    pub compatibility: f64,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub lat: f64,
    pub lon: f64,
    pub quantities: HashMap<String, f64>,
    pub v_id: Vec<String>,
    pub skills: Vec<String>,
    pub day_skills: Vec<String>,
    pub matrix_index: Option<usize>,
    pub duration_from_and_to_depot: Vec<f64>,
    pub linked_next: Option<usize>,
    pub centroid_weights: CentroidWeights,
    pub needs_to_stay_at_the_top: bool,
    pub moved_up: bool,
    pub moved_down: bool,
    pub weighted_visit_distance: f64,
    pub local_speed: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct Centroid {
    pub lat: f64,
    pub lon: f64,
    pub id: String,
    pub load: HashMap<String, f64>,
    pub v_id: Vec<String>,
    pub skills: Vec<String>,
    pub day_skills: Vec<String>,
    pub matrix_index: Option<usize>,
    pub duration_from_and_to_depot: f64,
    pub capacity_offence_coeff: f64,
    pub route_time: f64,
    pub area: f64,
    pub visit_count: f64,
    pub visit_density: f64,
    pub min_speed: f64,
    pub max_speed: f64,
    pub speed: f64,
    pub total_work_days: f64,
    pub vehicle_count: f64,
    pub capacities: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: Vec<String>,
    pub depot_lat: f64,
    pub depot_lon: f64,
    pub depot_matrix_index: Option<usize>,
    pub capacities: HashMap<String, f64>,
    pub skills: Vec<String>,
    pub day_skills: Vec<String>,
    pub duration: f64,
    pub total_work_days: f64,
    pub vehicle_count: f64,
}

#[derive(Debug, Clone)]
pub struct CutLimit {
    pub limit: f64,
}
