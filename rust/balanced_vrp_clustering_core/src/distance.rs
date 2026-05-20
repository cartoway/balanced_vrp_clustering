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

const R: f64 = 6_378_137.0;
const L: f64 = 111_321.0;
const RADIANS_IN_A_DEGREE: f64 = std::f64::consts::PI / 180.0;

pub fn deg2rad(degree: f64) -> f64 {
    degree * RADIANS_IN_A_DEGREE
}

pub fn euclidean_distance(loc_a: (f64, f64), loc_b: (f64, f64)) -> f64 {
    if loc_a.0 == 0.0 && loc_b.0 == 0.0 {
        return 0.0;
    }
    let delta_lat = loc_a.0 - loc_b.0;
    let delta_lon = (loc_a.1 - loc_b.1) * deg2rad((loc_a.0 + loc_b.0) / 2.0);
    L * (delta_lat * delta_lat + delta_lon * delta_lon).sqrt()
}

pub fn flying_distance(loc_a: (f64, f64), loc_b: (f64, f64)) -> f64 {
    if loc_a.0 == 0.0 || loc_b.0 == 0.0 {
        return 0.0;
    }
    if (loc_a.0 - loc_b.0).abs() < 30.0
        && loc_a.0.abs().max(loc_b.0.abs()) + (loc_a.1 - loc_b.1).abs() < 100.0
    {
        return euclidean_distance(loc_a, loc_b);
    }
    let deg2rad_loc_a_lat = deg2rad(loc_a.0);
    let deg2rad_loc_b_lat = deg2rad(loc_b.0);
    let intermediate = ((deg2rad_loc_b_lat - deg2rad_loc_a_lat) / 2.0).sin().powi(2)
        + ((deg2rad(loc_b.1) - deg2rad(loc_a.1)) / 2.0).sin().powi(2)
            * deg2rad_loc_b_lat.cos()
            * deg2rad_loc_a_lat.cos();
    R * 2.0 * (intermediate.sqrt()).atan2((1.0 - intermediate).sqrt())
}

pub fn matrix_distance(matrix: &[Vec<f64>], a_idx: usize, b_idx: usize) -> f64 {
    let d = matrix[a_idx][b_idx];
    let b_dist = matrix[b_idx][a_idx];
    if d <= b_dist { d } else { b_dist }
}

pub fn compute_approximate_route_time(
    area: f64,
    visit_count: f64,
    speed: f64,
    total_work_days: f64,
) -> f64 {
    let k0 = 0.765 * 1.45;
    total_work_days * k0 * (area * visit_count / total_work_days.powf(1.5)).sqrt() / speed
}

pub fn approximate_polygon_area(coordinates: &[(f64, f64)]) -> f64 {
    if coordinates.len() <= 2 {
        return 0.0;
    }
    let mut area = 0.0;
    let mut coor_p = coordinates[0];
    for coor in &coordinates[1..] {
        area += deg2rad(coor.1 - coor_p.1)
            * (2.0 + deg2rad(coor_p.0).sin() + deg2rad(coor.0).sin());
        coor_p = *coor;
    }
    (area * R * R / 2.0).abs()
}

pub fn position_and_distance_to_line(
    l_begin: (f64, f64),
    l_end: (f64, f64),
    point: (f64, f64),
) -> (f64, f64) {
    let lat_diff = l_end.0 - l_begin.0;
    let lon_diff = l_end.1 - l_begin.1;
    let position = lat_diff * (point.1 - l_begin.1) - lon_diff * (point.0 - l_begin.0);
    let dist = position.abs() / (lat_diff * lat_diff + lon_diff * lon_diff).sqrt();
    (position, dist)
}

pub fn approximate_quadrilateral_polygon<F>(
    coordinates: &[(f64, f64)],
    mut rand_index: F,
) -> Vec<(f64, f64)>
where
    F: FnMut(usize) -> usize,
{
    if coordinates.is_empty() {
        return vec![];
    }
    let crand_idx = rand_index(coordinates.len());
    let crand = coordinates[crand_idx];
    let c0 = coordinates
        .iter()
        .max_by(|a, b| {
            flying_distance(crand, **a)
                .partial_cmp(&flying_distance(crand, **b))
                .unwrap()
        })
        .copied()
        .unwrap();
    let c1 = coordinates
        .iter()
        .max_by(|a, b| {
            flying_distance(c0, **a)
                .partial_cmp(&flying_distance(c0, **b))
                .unwrap()
        })
        .copied()
        .unwrap();
    let c2 = coordinates
        .iter()
        .max_by(|a, b| {
            flying_distance(c1, **a)
                .partial_cmp(&flying_distance(c1, **b))
                .unwrap()
        })
        .copied()
        .unwrap();
    let mut c3 = None;
    let mut c4 = None;
    let mut max_up = 0.0;
    let mut max_down = 0.0;
    for c_i in coordinates {
        let (position, distance_to_line) =
            position_and_distance_to_line(c1, c2, *c_i);
        if position > 0.0 && distance_to_line > max_up {
            max_up = distance_to_line;
            c3 = Some(*c_i);
        } else if position < 0.0 && distance_to_line > max_down {
            max_down = distance_to_line;
            c4 = Some(*c_i);
        }
    }
    let mut result = vec![c1];
    if let Some(p) = c3 {
        result.push(p);
    }
    result.push(c2);
    if let Some(p) = c4 {
        result.push(p);
    }
    result.push(c1);
    result
}

pub fn check_if_projection_inside_the_line_segment(
    point: (f64, f64),
    line_beg: (f64, f64),
    line_end: (f64, f64),
    margin: f64,
) -> bool {
    let line_direction = [line_end.0 - line_beg.0, line_end.1 - line_beg.1];
    let point_direction = [point.0 - line_beg.0, point.1 - line_beg.1];
    let denom: f64 = line_direction.iter().map(|x| x * x).sum();
    if denom == 0.0 {
        return false;
    }
    let projection_scaler: f64 = (0..2)
        .map(|i| line_direction[i] * point_direction[i])
        .sum::<f64>()
        / denom;
    projection_scaler >= 0.0 + 0.5 * margin && projection_scaler <= 1.0 - 0.5 * margin
}
