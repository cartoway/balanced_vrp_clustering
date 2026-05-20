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

use crate::types::Item;
use std::collections::HashMap;

const LINKING_RELATIONS: &[&str] = &[
    "order",
    "same_route",
    "sequence",
    "same_vehicle",
    "shipment",
];
const BINDING_RELATIONS: &[&str] = &["order", "same_route", "sequence", "same_vehicle"];

pub fn connect_linked_items(
    items: &mut [Item],
    related_item_indices: &HashMap<String, Vec<Vec<usize>>>,
) {
    for relation in LINKING_RELATIONS {
        if let Some(groups) = related_item_indices.get(*relation) {
            for linked_indices in groups {
                assert_eq!(
                    linked_indices.len(),
                    linked_indices.iter().collect::<std::collections::HashSet<_>>().len(),
                    "Each relation group should contain only unique indices"
                );
            }
        }
    }

    let non_binding: Vec<&str> = LINKING_RELATIONS
        .iter()
        .copied()
        .filter(|r| !BINDING_RELATIONS.contains(r))
        .collect();

    for relation in non_binding {
        if let Some(groups) = related_item_indices.get(relation) {
            for mut linked_indices in groups.clone() {
                if linked_indices
                    .iter()
                    .any(|&ind| items[ind].linked_next.is_some())
                {
                    panic!("A service should not appear in multiple non-binding linking relations");
                }
                linked_indices.push(linked_indices[0]);
                for i in 0..linked_indices.len() - 1 {
                    let current = linked_indices[i];
                    let next = linked_indices[i + 1];
                    items[current].linked_next = Some(next);
                }
            }
        }
    }

    for relation in BINDING_RELATIONS {
        if let Some(groups) = related_item_indices.get(*relation) {
            for mut linked_indices in groups.clone() {
                linked_indices.push(linked_indices[0]);
                for i in 0..linked_indices.len() - 1 {
                    let mut item_idx = linked_indices[i];
                    let mut next_idx = linked_indices[i + 1];
                    let item_has = items[item_idx].linked_next.is_some();
                    let next_has = items[next_idx].linked_next.is_some();

                    if !item_has && !next_has {
                        items[item_idx].linked_next = Some(next_idx);
                    } else if item_has && next_has {
                        let first_loop_end = item_idx;
                        let mut walk = items[item_idx].linked_next.unwrap();
                        while walk != first_loop_end && walk != next_idx {
                            item_idx = walk;
                            walk = items[item_idx].linked_next.unwrap();
                        }
                        if items[item_idx].linked_next == Some(next_idx) {
                            continue;
                        }
                        let second_loop_end = next_idx;
                        let mut walk2 = items[next_idx].linked_next.unwrap();
                        while walk2 != second_loop_end {
                            next_idx = walk2;
                            walk2 = items[next_idx].linked_next.unwrap();
                        }
                        items[item_idx].linked_next = Some(second_loop_end);
                        items[next_idx].linked_next = Some(first_loop_end);
                    } else {
                        if item_has {
                            std::mem::swap(&mut item_idx, &mut next_idx);
                        }
                        items[item_idx].linked_next = Some(next_idx);
                        let loop_end = next_idx;
                        let mut walk = items[next_idx].linked_next.unwrap();
                        while walk != loop_end {
                            next_idx = walk;
                            walk = items[next_idx].linked_next.unwrap();
                        }
                        if next_idx != item_idx {
                            items[next_idx].linked_next = Some(item_idx);
                        }
                    }
                }
            }
        }
    }
}

/// Mirrors Ruby `do_forall_linked_items_of`
pub fn do_forall_linked<F>(items: &[Item], start: usize, mut f: F)
where
    F: FnMut(usize),
{
    let mut linked_item: Option<usize> = None;
    loop {
        if linked_item == Some(start) {
            break;
        }
        let current = if let Some(li) = linked_item {
            items[li].linked_next.unwrap_or(start)
        } else if let Some(next) = items[start].linked_next {
            next
        } else {
            start
        };
        f(current);
        linked_item = Some(current);
    }
}
