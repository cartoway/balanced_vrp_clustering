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

use crate::types::{Centroid, Item, Vehicle};

pub fn compatible_characteristics(item: &Item, vehicle: &Vehicle) -> bool {
    if !item.v_id.is_empty() && item.v_id.iter().all(|v| !vehicle.id.contains(v)) {
        return false;
    }
    if item.skills.iter().any(|s| !vehicle.skills.contains(s)) {
        return false;
    }
    if !item
        .day_skills
        .iter()
        .any(|d| vehicle.day_skills.contains(d))
    {
        return false;
    }
    true
}

pub fn compatible_item_centroid(item: &Item, centroid: &Centroid) -> bool {
    if !item.v_id.is_empty() && item.v_id.iter().all(|v| !centroid.v_id.contains(v)) {
        return false;
    }
    if item.skills.iter().any(|s| !centroid.skills.contains(s)) {
        return false;
    }
    if !item
        .day_skills
        .iter()
        .any(|d| centroid.day_skills.contains(d))
    {
        return false;
    }
    true
}

pub fn vehicle_as_centroid_props(vehicle: &Vehicle) -> Centroid {
    Centroid {
        lat: vehicle.depot_lat,
        lon: vehicle.depot_lon,
        id: vehicle.id.first().cloned().unwrap_or_default(),
        load: Default::default(),
        v_id: vehicle.id.clone(),
        skills: vehicle.skills.clone(),
        day_skills: vehicle.day_skills.clone(),
        matrix_index: vehicle.depot_matrix_index,
        duration_from_and_to_depot: 0.0,
        capacity_offence_coeff: 0.0,
        route_time: 0.0,
        area: 0.0,
        visit_count: 0.0,
        visit_density: 0.0,
        min_speed: 0.0,
        max_speed: 0.0,
        speed: 0.0,
        total_work_days: vehicle.total_work_days,
        vehicle_count: vehicle.vehicle_count,
        capacities: vehicle.capacities.clone(),
    }
}
