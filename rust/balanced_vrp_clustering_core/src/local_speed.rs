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

use crate::distance::flying_distance;
use crate::types::Item;
use std::collections::HashMap;

pub const LOCAL_SPEED_CELL_DEG: f64 = 0.01;
pub const LOCAL_SPEED_NEIGHBOR_LIMIT: usize = 100;

pub fn build_spatial_grid(items: &[Item]) -> HashMap<(i32, i32), Vec<usize>> {
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (idx, item) in items.iter().enumerate() {
        let key = (
            (item.lat / LOCAL_SPEED_CELL_DEG) as i32,
            (item.lon / LOCAL_SPEED_CELL_DEG) as i32,
        );
        grid.entry(key).or_default().push(idx);
    }
    grid
}

pub fn spatial_neighbors(item: &Item, grid: &HashMap<(i32, i32), Vec<usize>>) -> Vec<usize> {
    let cx = (item.lat / LOCAL_SPEED_CELL_DEG) as i32;
    let cy = (item.lon / LOCAL_SPEED_CELL_DEG) as i32;
    let mut neighbors = Vec::new();
    for dx in -1..=1 {
        for dy in -1..=1 {
            if let Some(bucket) = grid.get(&(cx + dx, cy + dy)) {
                neighbors.extend(bucket.iter().copied());
            }
        }
    }
    neighbors
}

pub fn calculate_local_speeds(items: &mut [Item]) {
    let local_max_speed = 60.0 / 3.6;
    let local_min_speed = 5.0 / 3.6;
    let grid = build_spatial_grid(items);

    for i in 0..items.len() {
        if items[i].local_speed.is_some() {
            continue;
        }
        let depot_duration_a = items[i].duration_from_and_to_depot[0];
        let a_lat = items[i].lat;
        let a_lon = items[i].lon;

        let mut candidates: Vec<(f64, usize)> = Vec::new();
        for j in spatial_neighbors(&items[i], &grid) {
            if i == j {
                continue;
            }
            let depot_diff = (depot_duration_a - items[j].duration_from_and_to_depot[0]).abs();
            if depot_diff <= 1.0 {
                continue;
            }
            let flying_dist = flying_distance((a_lat, a_lon), (items[j].lat, items[j].lon));
            if flying_dist <= 5.0 {
                continue;
            }
            candidates.push((flying_dist, j));
        }

        let calculated_speed = if candidates.is_empty() {
            local_min_speed
        } else {
            candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let speeds: Vec<f64> = candidates
                .iter()
                .take(LOCAL_SPEED_NEIGHBOR_LIMIT)
                .map(|(dist, j)| {
                    dist / (depot_duration_a - items[*j].duration_from_and_to_depot[0]).abs()
                })
                .collect();
            let sum: f64 = speeds.iter().take(2).sum();
            sum * 1.1
        };

        let speed = calculated_speed.max(local_min_speed).min(local_max_speed);
        items[i].local_speed = Some(speed);
    }
}
