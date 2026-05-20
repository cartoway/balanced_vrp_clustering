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

use crate::compatibility::{compatible_item_centroid, vehicle_as_centroid_props};
use crate::distance::{
    approximate_polygon_area, approximate_quadrilateral_polygon, check_if_projection_inside_the_line_segment,
    compute_approximate_route_time, euclidean_distance, flying_distance, matrix_distance,
};
use crate::limits::compute_limits;
use crate::local_speed::calculate_local_speeds;
use crate::relations::{connect_linked_items, do_forall_linked};
use crate::rng::RubyRng;
use crate::types::{
    BuildOutput, Centroid, CentroidWeights, CutLimit, InputConfig, Item, Vehicle,
    INCOMPATIBILITY_DISTANCE_PENALTY,
};
use std::collections::HashMap;
use std::time::Instant;

pub struct Clusterer {
    pub rng: RubyRng,
    pub vehicles: Vec<Vehicle>,
    pub items: Vec<Item>,
    pub item_order: Vec<usize>,
    pub distance_matrix: Option<Vec<Vec<f64>>>,
    pub cut_symbol: Option<String>,
    pub cut_ratio: f64,
    pub max_iterations: usize,
    pub centroid_indices: Vec<usize>,
    pub related_item_indices: HashMap<String, Vec<Vec<usize>>>,

    pub number_of_clusters: usize,
    pub unit_symbols: Vec<String>,
    pub iteration: usize,
    pub centroids: Vec<Centroid>,
    pub clusters: Vec<Vec<usize>>,
    pub strict_limitations: Vec<HashMap<String, f64>>,
    pub cut_limit: Vec<CutLimit>,
    pub balance_coeff: Vec<f64>,
    pub cluster_load: Vec<HashMap<String, f64>>,
    pub evaluate_distance_buffer: Vec<f64>,
    pub expected_n_visits: f64,
    pub total_cut_load: f64,
    pub approximate_total_route_time: f64,
    pub density_cap: f64,
    pub max_balance_violation: f64,
    pub limit_violation_count: usize,
    pub needs_to_stay_at_the_top: usize,
    pub old_centroids_lat_lon: Vec<(f64, f64)>,
    pub last_n_average_diffs: Vec<f64>,
    items_with_limit_violation: Vec<LimitViolationRecord>,
    pub clusters_with_limit_violation: Vec<Vec<usize>>,
    pub already_assigned: Vec<bool>,
}

#[derive(Clone)]
struct LimitViolationRecord {
    item_index: usize,
    diff: f64,
    ratio: f64,
    closest_cluster: usize,
    closest_wo_violation: usize,
    min_with_violation: f64,
}

impl Clusterer {
    pub fn from_input(input: &InputConfig) -> Self {
        let vehicles: Vec<Vehicle> = input
            .vehicles
            .iter()
            .map(|v| {
                let coords = v.depot.coordinates.as_ref();
                Vehicle {
                    id: v.id.clone(),
                    depot_lat: coords.map(|c| c[0]).unwrap_or(0.0),
                    depot_lon: coords.map(|c| c[1]).unwrap_or(0.0),
                    depot_matrix_index: v.depot.matrix_index,
                    capacities: v.capacities.clone(),
                    skills: v.skills.clone(),
                    day_skills: if v.day_skills.is_empty() {
                        default_day_skills()
                    } else {
                        v.day_skills.clone()
                    },
                    duration: v.duration,
                    total_work_days: v.total_work_days,
                    vehicle_count: v.vehicle_count,
                }
            })
            .collect();

        let k = vehicles.len();
        let items: Vec<Item> = input
            .items
            .iter()
            .map(|it| {
                let mut durations = it.duration_from_and_to_depot.clone();
                if durations.len() < k {
                    durations.resize(k, 0.0);
                }
                Item {
                    id: it.id.clone(),
                    lat: it.lat,
                    lon: it.lon,
                    quantities: it.quantities.clone(),
                    v_id: it.v_id.clone(),
                    skills: it.skills.clone(),
                    day_skills: if it.day_skills.is_empty() {
                        default_day_skills()
                    } else {
                        it.day_skills.clone()
                    },
                    matrix_index: it.matrix_index,
                    duration_from_and_to_depot: durations,
                    linked_next: None,
                    centroid_weights: CentroidWeights {
                        limit: vec![1.0; k],
                        compatibility: 1.0,
                    },
                    needs_to_stay_at_the_top: false,
                    moved_up: false,
                    moved_down: false,
                    weighted_visit_distance: 0.0,
                    local_speed: None,
                }
            })
            .collect();

        let item_order: Vec<usize> = (0..items.len()).collect();
        let max_iterations = if input.max_iterations > 0 {
            input.max_iterations
        } else {
            ((0.5 * items.len() as f64) as usize).max(100)
        };

        Self {
            rng: RubyRng::new(input.seed),
            vehicles,
            items,
            item_order,
            distance_matrix: input.distance_matrix.clone(),
            cut_symbol: input.cut_symbol.clone(),
            cut_ratio: input.cut_ratio,
            max_iterations,
            centroid_indices: input.centroid_indices.clone(),
            related_item_indices: input.related_item_indices.clone(),
            number_of_clusters: k,
            unit_symbols: Vec::new(),
            iteration: 0,
            centroids: Vec::new(),
            clusters: vec![Vec::new(); k],
            strict_limitations: Vec::new(),
            cut_limit: Vec::new(),
            balance_coeff: vec![1.0; k],
            cluster_load: vec![HashMap::new(); k],
            evaluate_distance_buffer: vec![0.0; k],
            expected_n_visits: 0.0,
            total_cut_load: 0.0,
            approximate_total_route_time: 0.0,
            density_cap: f64::EPSILON,
            max_balance_violation: 0.0,
            limit_violation_count: 0,
            needs_to_stay_at_the_top: 0,
            old_centroids_lat_lon: Vec::new(),
            last_n_average_diffs: Vec::new(),
            items_with_limit_violation: Vec::new(),
            clusters_with_limit_violation: vec![Vec::new(); k],
            already_assigned: Vec::new(),
        }
    }

    pub fn build(mut self) -> BuildOutput {
        let started = Instant::now();
        connect_linked_items(&mut self.items, &self.related_item_indices);

        self.init_unit_symbols();
        self.init_item_defaults();
        self.init_compatibility_weights();
        let (strict, cut_lim) = compute_limits(
            self.cut_symbol.as_deref(),
            self.cut_ratio,
            &self.vehicles,
            &self.items,
        );
        self.strict_limitations = strict;
        self.cut_limit = cut_lim;

        self.calc_initial_centroids();
        self.prepare_balancing();

        self.mark_items_at_top();

        while !self.stop_criteria_met() && self.iteration < self.max_iterations {
            self.calculate_membership_clusters();
            self.update_balance_coefficients();
            self.recompute_centroids();
        }

        let clusters: Vec<Vec<String>> = self
            .clusters
            .iter()
            .map(|c| c.iter().map(|&i| self.items[i].id.clone()).collect())
            .collect();

        BuildOutput {
            iterations: self.iteration,
            clusters,
            elapsed_ms: started.elapsed().as_millis() as u64,
        }
    }

    fn init_unit_symbols(&mut self) {
        let mut symbols = self.cut_symbol.clone().into_iter().collect::<Vec<_>>();
        for v in &self.vehicles {
            for unit in v.capacities.keys() {
                if !symbols.contains(unit) {
                    symbols.push(unit.clone());
                }
            }
        }
        self.unit_symbols = symbols;
    }

    fn init_item_defaults(&mut self) {
        for item in &mut self.items {
            if item.quantities.get("visits").is_none() {
                item.quantities.insert("visits".to_string(), 1.0);
            }
        }
        for vehicle in &mut self.vehicles {
            if vehicle.capacities.is_empty() {
                vehicle
                    .capacities
                    .insert("duration".to_string(), vehicle.duration);
            }
        }
    }

    fn init_compatibility_weights(&mut self) {
        let k = self.number_of_clusters;
        let total_visits: f64 = self.items.iter().map(|i| i.quantities.get("visits").copied().unwrap_or(1.0)).sum();
        self.expected_n_visits = total_visits / k as f64;

        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, item) in self.items.iter().enumerate() {
            let key = format!(
                "{:?}|{:?}|{:?}",
                item.v_id, item.skills, item.day_skills
            );
            groups.entry(key).or_default().push(idx);
        }

        for indices in groups.values() {
            let item = &self.items[indices[0]];
            let mut compatibility: Vec<u8> = Vec::new();
            for vehicle in &self.vehicles {
                let c = vehicle_as_centroid_props(vehicle);
                compatibility.push(if compatible_item_centroid(item, &c) { 1 } else { 0 });
            }
            let compatible_vehicle_count = compatibility.iter().map(|&x| x as usize).sum::<usize>().max(1);
            let incompatible_vehicle_count = k - compatible_vehicle_count;
            let compat_weight = (self.expected_n_visits
                .powf(
                    (1.0
                        + ((incompatible_vehicle_count as f64 / k as f64) + 0.1).ln())
                        / (1.0 + 1.1_f64.ln()),
                ))
                / (indices.len() as f64 / compatible_vehicle_count as f64).max(1.0);
            let weight = compat_weight.ceil();
            for &idx in indices {
                self.items[idx].centroid_weights.compatibility = weight;
            }
        }
    }

    fn prepare_balancing(&mut self) {
        let Some(ref cut) = self.cut_symbol.clone() else {
            return;
        };
        let cut = cut.clone();
        self.total_cut_load = self
            .items
            .iter()
            .map(|i| i.quantities.get(&cut).copied().unwrap_or(0.0))
            .sum();

        if self.total_cut_load == 0.0 {
            self.cut_symbol = None;
            return;
        }

        let centroid_len = self.centroids.len();
        let data_length = self.item_order.len();
        let tail_start = centroid_len.min(data_length);
        let tail_end = data_length;
        if tail_start < tail_end {
            let mut tail: Vec<usize> = self.item_order[tail_start..tail_end].to_vec();
            tail.sort_by(|&a, &b| {
                let qa = self.items[b].quantities.get(&cut).copied().unwrap_or(0.0);
                let qb = self.items[a].quantities.get(&cut).copied().unwrap_or(0.0);
                qa.partial_cmp(&qb).unwrap()
            });
            let range_end = (data_length as f64 * 0.9) as usize;
            let range_begin = (centroid_len + (data_length as f64 * 0.1) as usize).min(range_end);
            if range_begin < range_end {
                let slice = &mut tail[range_begin - tail_start..range_end - tail_start];
                self.shuffle_slice(slice);
            }
            self.item_order[tail_start..tail_end].copy_from_slice(&tail);
        }

        calculate_local_speeds(&mut self.items);

        let coords: Vec<(f64, f64)> = self
            .items
            .iter()
            .map(|i| (i.lat, i.lon))
            .collect();
        let quad = approximate_quadrilateral_polygon(&coords, |max| self.rng.rand_usize(max));
        let area_per_cluster =
            approximate_polygon_area(&quad) / self.vehicles.len() as f64;
        let speed: f64 = self
            .items
            .iter()
            .filter_map(|i| i.local_speed)
            .sum::<f64>()
            / self.items.len() as f64;
        let visit_count_per_cluster: f64 = self
            .items
            .iter()
            .map(|i| i.quantities.get("visits").copied().unwrap_or(1.0))
            .sum::<f64>()
            / self.vehicles.len() as f64;
        let total_work_days_per_cluster: f64 = self
            .vehicles
            .iter()
            .map(|v| v.total_work_days)
            .sum::<f64>()
            / self.vehicles.len() as f64;

        self.approximate_total_route_time = compute_approximate_route_time(
            area_per_cluster,
            visit_count_per_cluster,
            speed,
            total_work_days_per_cluster,
        ) * self.vehicles.len() as f64;
    }

    fn shuffle_slice(&mut self, slice: &mut [usize]) {
        for i in (1..slice.len()).rev() {
            let j = self.rng.rand_usize(i + 1);
            slice.swap(i, j);
        }
    }

    fn mark_items_at_top(&mut self) {
        for item in &mut self.items {
            item.needs_to_stay_at_the_top = false;
        }
        let units: Vec<String> = self
            .vehicles
            .iter()
            .flat_map(|v| v.capacities.keys().cloned())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let top_count = (self.items.len() as f64 * 0.005 + 1.0) as usize;
        for unit in units {
            let mut ranked: Vec<usize> = (0..self.items.len()).collect();
            ranked.sort_by(|&a, &b| {
                let va = self.items[b].quantities.get(&unit).copied().unwrap_or(0.0);
                let vb = self.items[a].quantities.get(&unit).copied().unwrap_or(0.0);
                va.partial_cmp(&vb).unwrap()
            });
            for &idx in ranked.iter().take(top_count) {
                if self.items[idx].quantities.get(&unit).copied().unwrap_or(0.0) > 0.0 {
                    self.items[idx].needs_to_stay_at_the_top = true;
                }
            }
        }
        self.needs_to_stay_at_the_top = self.items.iter().filter(|i| i.needs_to_stay_at_the_top).count();
        let mut top_items: Vec<usize> = self
            .item_order
            .iter()
            .copied()
            .filter(|&i| self.items[i].needs_to_stay_at_the_top)
            .collect();
        let rest: Vec<usize> = self
            .item_order
            .iter()
            .copied()
            .filter(|&i| !self.items[i].needs_to_stay_at_the_top)
            .collect();
        top_items.extend(rest);
        self.item_order = top_items;
    }

    fn distance_between(&self, item_idx: usize, centroid: &Centroid) -> f64 {
        let item = &self.items[item_idx];
        if let Some(ref matrix) = self.distance_matrix {
            let a_idx = item.matrix_index.unwrap();
            let b_idx = centroid.matrix_index.unwrap();
            matrix_distance(matrix, a_idx, b_idx)
        } else {
            flying_distance((item.lat, item.lon), (centroid.lat, centroid.lon))
        }
    }

    fn distance(&self, item_idx: usize, cluster_index: usize) -> f64 {
        let centroid = &self.centroids[cluster_index];
        let mut total_dist = 0.0;
        do_forall_linked(&self.items, item_idx, |linked_idx| {
            total_dist += self.distance_between(linked_idx, centroid);
        });
        total_dist * self.balance_coeff[cluster_index]
    }

    fn get_min_index(distances: &[f64]) -> usize {
        distances
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn capacity_violation(&self, item_idx: usize, cluster_index: usize) -> bool {
        for (unit, &limit) in &self.strict_limitations[cluster_index] {
            // Match Ruby: `next unless @strict_limitations[cluster_index][unit]` (0 is truthy in Ruby)
            let added = self.linked_quantity(item_idx, unit);
            let load = self.cluster_load[cluster_index].get(unit).copied().unwrap_or(0.0);
            if load + added > limit {
                return true;
            }
        }
        false
    }

    fn linked_quantity(&self, item_idx: usize, unit: &str) -> f64 {
        let mut total = 0.0;
        do_forall_linked(&self.items, item_idx, |idx| {
            total += self.items[idx].quantities.get(unit).copied().unwrap_or(0.0);
        });
        total
    }

    fn evaluate(&mut self, item_idx: usize) -> usize {
        let k = self.number_of_clusters;
        let mut distances = vec![0.0; k];
        for i in 0..k {
            let mut dist = self.distance(item_idx, i);
            if !compatible_item_centroid(&self.items[item_idx], &self.centroids[i]) {
                dist += INCOMPATIBILITY_DISTANCE_PENALTY;
            }
            distances[i] = dist;
        }
        self.evaluate_distance_buffer.copy_from_slice(&distances);
        let closest_cluster_index = Self::get_min_index(&distances);

        if self.capacity_violation(item_idx, closest_cluster_index) {
            let mut min_wo = INCOMPATIBILITY_DISTANCE_PENALTY;
            let mut closest_wo = None;
            for k in 0..self.number_of_clusters {
                if distances[k] < min_wo && !self.capacity_violation(item_idx, k) {
                    closest_wo = Some(k);
                    min_wo = distances[k];
                }
            }
            if let Some(wo_idx) = closest_wo {
                let min_with = distances.iter().copied().fold(f64::INFINITY, f64::min);
                let mut ratio = min_wo / min_with;
                if ratio.is_nan() {
                    ratio = 1.0;
                }
                self.items_with_limit_violation.push(LimitViolationRecord {
                    item_index: item_idx,
                    diff: min_wo - min_with,
                    ratio,
                    closest_cluster: closest_cluster_index,
                    closest_wo_violation: wo_idx,
                    min_with_violation: min_with,
                });
                self.clusters_with_limit_violation[closest_cluster_index].push(wo_idx);
                return wo_idx;
            }
        }
        closest_cluster_index
    }

    fn calculate_membership_clusters(&mut self) {
        self.update_strict_duration_limitation();
        self.items_with_limit_violation.clear();
        for c in &mut self.centroids {
            c.load.clear();
        }
        for load in &mut self.cluster_load {
            load.clear();
        }
        self.clusters = vec![Vec::new(); self.number_of_clusters];
        self.already_assigned = vec![false; self.items.len()];

        let order = self.item_order.clone();
        for item_idx in order {
            if self.already_assigned[item_idx] {
                continue;
            }
            let cluster_index = self.evaluate(item_idx);
            let mut linked = Vec::new();
            do_forall_linked(&self.items, item_idx, |linked_idx| linked.push(linked_idx));
            for linked_idx in linked {
                self.assign_item(linked_idx, cluster_index);
                self.update_metrics(linked_idx, cluster_index);
            }
        }
        self.manage_empty_clusters();
    }

    fn assign_item(&mut self, item_idx: usize, cluster_index: usize) {
        self.already_assigned[item_idx] = true;
        if !self.clusters[cluster_index].contains(&item_idx) {
            self.clusters[cluster_index].push(item_idx);
        }
    }

    fn update_metrics(&mut self, item_idx: usize, cluster_index: usize) {
        for (unit, &value) in &self.items[item_idx].quantities {
            let q = value;
            *self.centroids[cluster_index].load.entry(unit.clone()).or_insert(0.0) += q;
            *self.cluster_load[cluster_index].entry(unit.clone()).or_insert(0.0) += q;
        }
    }

    fn update_strict_duration_limitation(&mut self) {
        if self.cut_symbol.is_none() {
            return;
        }
        for (index, centroid) in self.centroids.iter_mut().enumerate() {
            if let Some(limit) = self.strict_limitations[index].get("duration").copied() {
                self.strict_limitations[index].insert(
                    "duration".to_string(),
                    limit - centroid.duration_from_and_to_depot * self.vehicles[index].total_work_days,
                );
            }
        }
    }

    fn calc_initial_centroids(&mut self) {
        self.centroids.clear();
        self.old_centroids_lat_lon.clear();
        let mut remaining_skills: Vec<Vehicle> = self.vehicles.clone();

        if self.centroid_indices.is_empty() {
            while self.centroids.len() < self.number_of_clusters {
                let skills = remaining_skills.remove(0);
                let mut available: Vec<usize> = self.item_order.clone();
                let vehicle_centroid = vehicle_as_centroid_props(&skills);
                let mut compatible = self.compatible_items_multi_depot(
                    available
                        .iter()
                        .copied()
                        .filter(|&idx| {
                            let item = &self.items[idx];
                            !item.v_id.is_empty() || !item.skills.is_empty()
                        })
                        .filter(|&idx| compatible_item_centroid(&self.items[idx], &vehicle_centroid))
                        .collect(),
                );
                if compatible.is_empty() {
                    compatible = self.compatible_items_multi_depot(
                        available
                            .iter()
                            .copied()
                            .filter(|&idx| compatible_item_centroid(&self.items[idx], &vehicle_centroid))
                            .collect(),
                    );
                }
                if compatible.is_empty() {
                    compatible = self.compatible_items_multi_depot(available.iter().copied().collect());
                }
                if compatible.is_empty() {
                    compatible = self.compatible_items_multi_depot((0..self.items.len()).collect());
                }
                let pick = compatible[self.rng.rand_usize(compatible.len())];
                let item = &self.items[pick];
                let mut centroid = vehicle_as_centroid_props(&skills);
                centroid.lat = item.lat;
                centroid.lon = item.lon;
                centroid.id = item.id.clone();
                centroid.matrix_index = item.matrix_index;
                centroid.duration_from_and_to_depot =
                    item.duration_from_and_to_depot[self.centroids.len()];
                self.centroids.push(centroid);
                let mut to_remove = Vec::new();
                do_forall_linked(&self.items, pick, |li| to_remove.push(li));
                available.retain(|i| !to_remove.contains(i));
                self.move_item_to_front(pick);
            }
        } else {
            let mut insert_at_beginning = Vec::new();
            for (ind, &index) in self.centroid_indices.iter().enumerate() {
                let skills = remaining_skills.remove(0);
                let item = &self.items[index];
                do_forall_linked(&self.items, index, |li| {
                    if insert_at_beginning.contains(&li) {
                        panic!("Centroid linked conflict");
                    }
                });
                let vehicle_centroid = vehicle_as_centroid_props(&skills);
                if !compatible_item_centroid(item, &vehicle_centroid) {
                    panic!("Incompatible centroid init");
                }
                let mut centroid = vehicle_centroid;
                centroid.lat = item.lat;
                centroid.lon = item.lon;
                centroid.id = item.id.clone();
                centroid.matrix_index = item.matrix_index;
                centroid.duration_from_and_to_depot = item.duration_from_and_to_depot[ind];
                self.centroids.push(centroid);
                insert_at_beginning.push(index);
            }
            for idx in insert_at_beginning {
                self.move_item_to_front(idx);
            }
        }
        self.number_of_clusters = self.centroids.len();
        for c in &mut self.centroids {
            c.capacity_offence_coeff = 0.0;
            c.route_time = 0.0;
        }
    }

    fn move_item_to_front(&mut self, item_idx: usize) {
        if let Some(pos) = self.item_order.iter().position(|&i| i == item_idx) {
            self.item_order.remove(pos);
            self.item_order.insert(0, item_idx);
        }
    }

    fn compatible_items_multi_depot(&self, items: Vec<usize>) -> Vec<usize> {
        let mut closest_items = Vec::new();
        for margin in 0..self.vehicles.len() {
            let c_len = self.centroids.len();
            closest_items = items
                .iter()
                .copied()
                .filter(|&idx| {
                    let d = &self.items[idx].duration_from_and_to_depot;
                    let mut sorted: Vec<f64> = d.to_vec();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    d[c_len] <= sorted[margin.min(sorted.len() - 1)]
                })
                .collect();
            if !closest_items.is_empty() {
                break;
            }
        }
        closest_items
    }

    fn move_limit_violating_dataitems(&mut self) {
        self.limit_violation_count = self.items_with_limit_violation.len();
        if self.items_with_limit_violation.is_empty() {
            return;
        }
        let mean_distance_diff: f64 = self.items_with_limit_violation.iter().map(|d| d.diff).sum::<f64>()
            / self.limit_violation_count as f64;
        let mean_ratio: f64 = self.items_with_limit_violation.iter().map(|d| d.ratio).sum::<f64>()
            / self.limit_violation_count as f64;

        self.items_with_limit_violation
            .sort_by(|a, b| b.min_with_violation.partial_cmp(&a.min_with_violation).unwrap());

        while let Some(data) = self.items_with_limit_violation.pop() {
            let item_idx = data.item_index;
            if !self.items[item_idx].needs_to_stay_at_the_top
                && (data.ratio > (2.0 * mean_ratio).max(5.0)
                    || self.rng.rand_f64() < (data.diff / (3.0 * mean_distance_diff + 1e-10)))
            {
                let point = (self.items[item_idx].lat, self.items[item_idx].lon);
                let centroid_violation = (
                    self.centroids[data.closest_cluster].lat,
                    self.centroids[data.closest_cluster].lon,
                );
                let centroid_ok = (
                    self.centroids[data.closest_wo_violation].lat,
                    self.centroids[data.closest_wo_violation].lon,
                );
                if check_if_projection_inside_the_line_segment(point, centroid_violation, centroid_ok, 0.1)
                {
                    let max_w = (self.expected_n_visits / 10.0).max(5.0);
                    for w in &mut self.items[item_idx].centroid_weights.limit {
                        *w = (*w * 2.0).min(max_w);
                    }
                    self.items[item_idx].centroid_weights.limit[data.closest_cluster] = 1.0;
                    self.items[item_idx].moved_down = true;
                    if let Some(pos) = self.item_order.iter().position(|&i| i == item_idx) {
                        self.item_order.remove(pos);
                        self.item_order.push(item_idx);
                    }
                } else {
                    self.items[item_idx].moved_up = true;
                    if let Some(pos) = self.item_order.iter().position(|&i| i == item_idx) {
                        self.item_order.remove(pos);
                        self.item_order.insert(self.needs_to_stay_at_the_top, item_idx);
                    }
                }
            }
        }
    }

    fn recompute_centroids(&mut self) {
        self.move_limit_violating_dataitems();
        self.old_centroids_lat_lon = self
            .centroids
            .iter()
            .map(|c| (c.lat, c.lon))
            .collect();
        let smoothing = 0.1 + 0.9 * (self.iteration as f64 / self.max_iterations as f64).powf(0.5);

        for index in 0..self.number_of_clusters {
            if self.clusters[index].is_empty() {
                continue;
            }
            let centroid = &self.centroids[index];
            let c_lat = centroid.lat;
            let c_lon = centroid.lon;
            let mut total_weighted = 0.0;
            let cluster_items: Vec<usize> = self.clusters[index].clone();

            for &item_idx in &cluster_items {
                let item = &mut self.items[item_idx];
                let distance_weight = if item.moved_up {
                    0.1
                } else if item.moved_down {
                    10.0
                } else {
                    1.0
                };
                let w = distance_weight
                    * (flying_distance((c_lat, c_lon), (item.lat, item.lon)) + 1.0).powf(0.2)
                    * item.quantities.get("visits").copied().unwrap_or(1.0).powf(0.2)
                    * item.centroid_weights.compatibility
                    * item.centroid_weights.limit[index];
                item.weighted_visit_distance = w;
                item.moved_up = false;
                item.moved_down = false;
                total_weighted += w;
            }

            {
                let centroid = &mut self.centroids[index];
                let sum_lat: f64 = cluster_items
                    .iter()
                    .map(|&i| self.items[i].lat * self.items[i].weighted_visit_distance)
                    .sum();
                let sum_lon: f64 = cluster_items
                    .iter()
                    .map(|&i| self.items[i].lon * self.items[i].weighted_visit_distance)
                    .sum();
                centroid.lat = smoothing * centroid.lat + (1.0 - smoothing) * sum_lat / total_weighted;
                centroid.lon = smoothing * centroid.lon + (1.0 - smoothing) * sum_lon / total_weighted;
            }

            let c_lat = self.centroids[index].lat;
            let c_lon = self.centroids[index].lon;
            let sample_size = ((cluster_items.len() as f64 / 10.0).ceil() as usize)
                .max(5)
                .min(cluster_items.len())
                .max(2);
            let mut nearest: Vec<usize> = cluster_items.clone();
            nearest.sort_by(|&a, &b| {
                flying_distance((c_lat, c_lon), (self.items[a].lat, self.items[a].lon))
                    .partial_cmp(&flying_distance(
                        (c_lat, c_lon),
                        (self.items[b].lat, self.items[b].lon),
                    ))
                    .unwrap()
            });
            nearest.truncate(sample_size);
            let centroid_ref = self.centroids[index].clone();
            let representative = nearest
                .iter()
                .min_by(|&&a, &&b| {
                    let sum_a: f64 = cluster_items
                        .iter()
                        .map(|&i| {
                            self.distance_between(a, &centroid_ref)
                                * self.items[i].quantities.get("visits").copied().unwrap_or(1.0)
                        })
                        .sum();
                    let sum_b: f64 = cluster_items
                        .iter()
                        .map(|&i| {
                            self.distance_between(b, &centroid_ref)
                                * self.items[i].quantities.get("visits").copied().unwrap_or(1.0)
                        })
                        .sum();
                    sum_a.partial_cmp(&sum_b).unwrap()
                })
                .copied()
                .unwrap_or(cluster_items[0]);

            let rep = self.items[representative].clone();
            let centroid = &mut self.centroids[index];
            centroid.id = rep.id.clone();
            if centroid.matrix_index.is_some() {
                centroid.matrix_index = rep.matrix_index;
            }
            if centroid.duration_from_and_to_depot > 0.0 || rep.duration_from_and_to_depot[index] > 0.0 {
                centroid.duration_from_and_to_depot = cluster_items
                    .iter()
                    .map(|&i| self.items[i].duration_from_and_to_depot[index])
                    .sum::<f64>()
                    / cluster_items.len() as f64;
            }
        }

        self.swap_centroid_with_limit_violation();
        self.iteration += 1;
    }

    fn swap_centroid_with_limit_violation(&mut self) {
        let mut already_swapped = false;
        let mut order: Vec<(usize, usize)> = self
            .clusters_with_limit_violation
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v.len()))
            .collect();
        order.sort_by(|a, b| b.1.cmp(&a.1));

        for (violated_cluster, _) in order {
            let preferred = &self.clusters_with_limit_violation[violated_cluster];
            if preferred.is_empty() {
                if self.centroids[violated_cluster].capacity_offence_coeff > 0.3 {
                    self.centroids[violated_cluster].capacity_offence_coeff -= 0.3;
                }
                continue;
            }

            let units_that_matter: Vec<String> = self.centroids[violated_cluster]
                .load
                .iter()
                .filter(|(_, v)| **v > 0.0)
                .map(|(k, _)| k.clone())
                .filter(|u| {
                    self.strict_limitations[violated_cluster]
                        .get(u)
                        .copied()
                        .unwrap_or(0.0) > 0.0
                })
                .collect();

            let mut favorite_clusters: Vec<usize> = Vec::new();
            for i in 0..self.number_of_clusters {
                let c = &self.centroids[i];
                let cap_ok = units_that_matter.iter().any(|unit| {
                    let limit = c.capacities.get(unit).copied();
                    let v_cap = self.centroids[violated_cluster].capacities.get(unit).copied();
                    v_cap.is_some() && (limit.is_none() || limit.unwrap() > v_cap.unwrap())
                });
                if !cap_ok {
                    continue;
                }
                let swap_ok = units_that_matter.iter().all(|unit| {
                    let v_lim = self.strict_limitations[violated_cluster].get(unit).copied();
                    let i_lim = self.strict_limitations[i].get(unit).copied();
                    let c_load = c.load.get(unit).copied().unwrap_or(0.0);
                    let v_cap_val = self.centroids[violated_cluster].load.get(unit).copied().unwrap_or(0.0);
                    (v_lim.is_none() || c_load < 0.98 * v_lim.unwrap())
                        && (i_lim.is_none() || i_lim.unwrap() >= 1.02 * v_cap_val)
                        && self.clusters[violated_cluster].iter().all(|&idx| {
                            compatible_item_centroid(&self.items[idx], &self.centroids[i])
                        })
                        && self.clusters[i].iter().all(|&idx| {
                            compatible_item_centroid(&self.items[idx], &self.centroids[violated_cluster])
                        })
                });
                if !swap_ok {
                    continue;
                }
                let v_dur: f64 = self.clusters[violated_cluster]
                    .iter()
                    .map(|&idx| self.items[idx].duration_from_and_to_depot[i])
                    .sum::<f64>()
                    / self.clusters[violated_cluster].len().max(1) as f64;
                let f_dur: f64 = self.clusters[i]
                    .iter()
                    .map(|&idx| self.items[idx].duration_from_and_to_depot[violated_cluster])
                    .sum::<f64>()
                    / self.clusters[i].len().max(1) as f64;
                let v_cent = self.centroids[violated_cluster].duration_from_and_to_depot;
                let f_cent = self.centroids[i].duration_from_and_to_depot;
                if (v_dur < v_cent || f_dur < f_cent)
                    || (v_dur < 2.0 * v_cent && f_dur < 2.0 * f_cent)
                {
                    favorite_clusters.push(i);
                }
            }

            if favorite_clusters.is_empty() {
                self.balance_coeff[violated_cluster] /= 0.95;
                self.centroids[violated_cluster].capacity_offence_coeff += 1.0;
                continue;
            }
            if already_swapped {
                continue;
            }

            let favorite = favorite_clusters
                .iter()
                .min_by(|&&a, &&b| {
                    let score_a: f64 = units_that_matter
                        .iter()
                        .map(|unit| {
                            let lim = self.strict_limitations[a].get(unit).copied();
                            if lim.is_none() || lim.unwrap() == 0.0 {
                                0.0
                            } else {
                                self.centroids[a].load.get(unit).copied().unwrap_or(0.0)
                                    / lim.unwrap()
                                    * (self.rng.rand_range_f64(0.0, 0.90) + 0.1)
                            }
                        })
                        .sum();
                    let score_b: f64 = units_that_matter
                        .iter()
                        .map(|unit| {
                            let lim = self.strict_limitations[b].get(unit).copied();
                            if lim.is_none() || lim.unwrap() == 0.0 {
                                0.0
                            } else {
                                self.centroids[b].load.get(unit).copied().unwrap_or(0.0)
                                    / lim.unwrap()
                                    * (self.rng.rand_range_f64(0.0, 0.90) + 0.1)
                            }
                        })
                        .sum();
                    score_a.partial_cmp(&score_b).unwrap()
                })
                .copied()
                .unwrap();

            if favorite == violated_cluster {
                continue;
            }

            let v = violated_cluster;
            let f = favorite;
            let swap_lat = self.centroids[v].lat;
            let swap_lon = self.centroids[v].lon;
            let swap_id = self.centroids[v].id.clone();
            let swap_matrix = self.centroids[v].matrix_index;
            let swap_depot = self.centroids[v].duration_from_and_to_depot;
            let swap_balance = self.balance_coeff[v];

            self.centroids[v].lat = self.centroids[f].lat;
            self.centroids[v].lon = self.centroids[f].lon;
            self.centroids[v].id = self.centroids[f].id.clone();
            self.centroids[v].matrix_index = self.centroids[f].matrix_index;
            self.centroids[v].duration_from_and_to_depot = self.centroids[f].duration_from_and_to_depot;
            self.balance_coeff[v] = self.balance_coeff[f];

            self.centroids[f].lat = swap_lat;
            self.centroids[f].lon = swap_lon;
            self.centroids[f].id = swap_id;
            self.centroids[f].matrix_index = swap_matrix;
            self.centroids[f].duration_from_and_to_depot = swap_depot;
            self.balance_coeff[f] = swap_balance;

            already_swapped = true;
        }

        for v in &mut self.clusters_with_limit_violation {
            v.clear();
        }
    }

    fn manage_empty_clusters(&mut self) {
        if !self.clusters.iter().any(|c| c.is_empty()) {
            return;
        }
        for ind in 0..self.number_of_clusters {
            if !self.clusters[ind].is_empty() {
                continue;
            }
            let empty_centroid = self.centroids[ind].clone();
            let mut best: Option<(f64, usize, usize)> = None;
            for (cluster_idx, cluster) in self.clusters.iter().enumerate() {
                if cluster.len() <= 1 {
                    continue;
                }
                let mut min_dist = f64::INFINITY;
                let mut closest_item = None;
                for &item_idx in cluster {
                    if !compatible_item_centroid(&self.items[item_idx], &empty_centroid) {
                        continue;
                    }
                    let mut total_dist = 0.0;
                    do_forall_linked(&self.items, item_idx, |li| {
                        total_dist += self.distance_between(li, &empty_centroid);
                    });
                    if total_dist < min_dist {
                        min_dist = total_dist;
                        closest_item = Some(item_idx);
                    }
                }
                if let Some(item_idx) = closest_item {
                    if best.is_none() || min_dist < best.unwrap().0 {
                        best = Some((min_dist, item_idx, cluster_idx));
                    }
                }
            }
            if let Some((_, item_idx, from_cluster)) = best {
                let mut linked = Vec::new();
                do_forall_linked(&self.items, item_idx, |li| linked.push(li));
                for li in linked {
                    self.clusters[from_cluster].retain(|&i| i != li);
                    self.clusters[ind].push(li);
                }
            }
        }
    }

    fn update_balance_coefficients(&mut self) {
        let Some(ref cut) = self.cut_symbol.clone() else {
            return;
        };
        let cut = cut.clone();
        self.update_cut_limit_wrt_depot_and_route_time();

        let mut balance_violations = Vec::new();
        for index in 0..self.number_of_clusters {
            let route_time = if cut == "duration" {
                self.centroids[index].route_time
            } else {
                0.0
            };
            let load = self.centroids[index].load.get(&cut).copied().unwrap_or(0.0);
            let violation = (load + route_time) / self.cut_limit[index].limit - 1.0;
            balance_violations.push(violation);
        }

        let stepsize = 0.2 - 0.1 * self.iteration as f64 / self.max_iterations as f64;
        let max_correction = 1.05 + 0.95 * (self.max_iterations - self.iteration) as f64 / self.max_iterations as f64;
        let min_correction = 1.0 / max_correction;
        self.max_balance_violation = 0.0;

        for index in 0..self.number_of_clusters {
            if !self.clusters_with_limit_violation[index].is_empty() && balance_violations[index] < 0.0 {
                continue;
            }
            self.max_balance_violation = self
                .max_balance_violation
                .max(balance_violations[index].abs());
            let balance_correction = ((1.0 + balance_violations[index]).powf(stepsize))
                .max(min_correction)
                .min(max_correction);
            self.balance_coeff[index] *= balance_correction;
            for bc in &mut self.balance_coeff {
                *bc /= balance_correction;
            }
        }
        let mean: f64 = self.balance_coeff.iter().sum::<f64>() / self.balance_coeff.len() as f64;
        for bc in &mut self.balance_coeff {
            *bc /= mean;
        }
    }

    fn update_cut_limit_wrt_depot_and_route_time(&mut self) {
        self.update_approximate_route_times();
        if self.cut_symbol.as_deref() != Some("duration") {
            return;
        }
        let vehicle_work_times = self.compute_vehicle_work_time_with_depot_and_capacity();
        let total: f64 = vehicle_work_times.iter().sum();
        for index in 0..self.number_of_clusters {
            self.cut_limit[index].limit = (self.total_cut_load + self.approximate_total_route_time)
                * vehicle_work_times[index]
                / total;
        }
    }

    fn compute_vehicle_work_time_with_depot_and_capacity(&self) -> Vec<f64> {
        let coef = self
            .centroids
            .iter()
            .enumerate()
            .map(|(index, c)| {
                self.vehicles[index].duration
                    / (c.duration_from_and_to_depot.max(1.0) * self.vehicles[index].total_work_days)
            })
            .fold(f64::INFINITY, f64::min);
        let coef = (coef * 0.9).min(1.0);
        let min_offence = self
            .centroids
            .iter()
            .map(|c| c.capacity_offence_coeff)
            .fold(f64::INFINITY, f64::min);
        self.centroids
            .iter()
            .enumerate()
            .map(|(index, c)| {
                let offence = c.capacity_offence_coeff - min_offence;
                (self.vehicles[index].duration
                    - coef * c.duration_from_and_to_depot * self.vehicles[index].total_work_days)
                    * 0.98_f64.powf(offence)
            })
            .collect()
    }

    fn update_approximate_route_times(&mut self) {
        if self.cut_symbol.is_none() {
            return;
        }
        self.update_approximate_area_and_speeds();
        let mut approximate_total = 0.0;
        for centroid in &mut self.centroids {
            if centroid.speed == 0.0 {
                continue;
            }
            centroid.route_time = compute_approximate_route_time(
                centroid.area,
                centroid.visit_count,
                centroid.speed,
                centroid.total_work_days / centroid.vehicle_count,
            ) * centroid.vehicle_count;
            approximate_total += centroid.route_time;
        }
        if approximate_total < self.approximate_total_route_time {
            self.approximate_total_route_time =
                (9.0 * self.approximate_total_route_time + approximate_total) / 10.0;
        }
    }

    fn update_approximate_area_and_speeds(&mut self) {
        let densities: Vec<f64> = self
            .centroids
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                if self.clusters[index].is_empty() {
                    None
                } else {
                    let coords: Vec<(f64, f64)> = self
                        .clusters[index]
                        .iter()
                        .map(|&i| (self.items[i].lat, self.items[i].lon))
                        .collect();
                    let quad = approximate_quadrilateral_polygon(&coords, |m| self.rng.rand_usize(m));
                    let area = approximate_polygon_area(&quad).max(1.0);
                    let visit_count: f64 = self
                        .clusters[index]
                        .iter()
                        .map(|&i| self.items[i].quantities.get("visits").copied().unwrap_or(1.0))
                        .sum();
                    Some(visit_count / area)
                }
            })
            .collect();
        if let Some(median) = median(&densities) {
            self.density_cap = self.density_cap.max(median * 6.0);
        }

        for index in 0..self.number_of_clusters {
            if self.clusters[index].is_empty() {
                continue;
            }
            let cluster = &self.clusters[index];
            let min_size = (0.95 * cluster.len() as f64).ceil() as usize;
            let max_size = (0.15 * cluster.len() as f64).ceil() as usize;
            let mut speeds: Vec<f64> = cluster
                .iter()
                .map(|&i| self.items[i].local_speed.unwrap_or(0.0))
                .collect();
            speeds.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let min_speed: f64 = speeds.iter().take(min_size).sum::<f64>() / min_size as f64;
            speeds.sort_by(|a, b| b.partial_cmp(a).unwrap());
            let max_speed: f64 = speeds.iter().take(max_size).sum::<f64>() / max_size.max(1) as f64;
            let cp = &mut self.centroids[index];
            let coords: Vec<(f64, f64)> = cluster
                .iter()
                .map(|&i| (self.items[i].lat, self.items[i].lon))
                .collect();
            let quad = approximate_quadrilateral_polygon(&coords, |m| self.rng.rand_usize(m));
            cp.area = approximate_polygon_area(&quad).max(1.0);
            cp.visit_count = cluster
                .iter()
                .map(|&i| self.items[i].quantities.get("visits").copied().unwrap_or(1.0))
                .sum();
            cp.visit_density = cp.visit_count / cp.area;
            cp.min_speed = min_speed;
            cp.max_speed = max_speed;
            let ratio = ((self.density_cap - cp.visit_density).max(0.0) / self.density_cap).powf(6.0);
            cp.speed = cp.min_speed + (cp.max_speed - cp.min_speed).max(0.0) * ratio;
        }
    }

    fn centroids_converged_or_in_loop(&mut self, last_n_iterations: usize) -> bool {
        if self.iteration == 0 {
            self.last_n_average_diffs = vec![0.0; 2 * last_n_iterations + 1];
            return false;
        }
        let mut total_movement = 0.0;
        for i in 0..self.number_of_clusters {
            total_movement +=
                euclidean_distance(self.old_centroids_lat_lon[i], (self.centroids[i].lat, self.centroids[i].lon));
        }
        self.last_n_average_diffs.push(total_movement);
        if self.last_n_average_diffs.last().copied().unwrap_or(0.0)
            < self.number_of_clusters as f64
                * (20.0 + 80.0 * self.iteration as f64 / self.max_iterations as f64)
        {
            return true;
        }
        for n in 1..=last_n_iterations {
            let len = self.last_n_average_diffs.len();
            if len < 2 * n {
                break;
            }
            let curr: f64 = self.last_n_average_diffs[len - n..].iter().sum();
            let prev: f64 = self.last_n_average_diffs[len - n - n..len - n].iter().sum();
            if (curr - prev).abs() < 1e-5 {
                return true;
            }
        }
        if self.last_n_average_diffs.len() > 2 * last_n_iterations + 1 {
            self.last_n_average_diffs.remove(0);
        }
        false
    }

    fn stop_criteria_met(&mut self) -> bool {
        let last_n = (self.iteration as f64).sqrt() as usize;
        self.centroids_converged_or_in_loop(last_n)
            && self.limit_violation_count == 0
            && self.max_balance_violation
                <= 0.05 + (self.iteration as f64 / self.max_iterations as f64).powi(8)
    }
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let m = v.len() / 2;
    if v.len() % 2 == 1 {
        Some(v[m])
    } else {
        Some((v[m - 1] + v[m]) / 2.0)
    }
}

fn default_day_skills() -> Vec<String> {
    (0..7)
        .map(|i| format!("{}_day_skill", i))
        .collect()
}

pub fn build_from_input(input: &InputConfig) -> BuildOutput {
    Clusterer::from_input(input).build()
}
