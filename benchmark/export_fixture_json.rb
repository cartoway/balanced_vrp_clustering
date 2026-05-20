# Copyright © Cartoway
#
# This file is part of balanced_vrp_clustering.
#
# Cartoway balanced_vrp_clustering is free software. You can redistribute it and/or
# modify since you respect the terms of the GNU Affero General
# Public License as published by the Free Software Foundation,
# either version 3 of the License, or (at your option) any later version.
#
# Cartoway Planner is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
# or FITNESS FOR A PARTICULAR PURPOSE.  See the Licenses for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with Cartoway Planner. If not, see:
# <http://www.gnu.org/licenses/agpl.html>
#

# frozen_string_literal: true

require 'json'
require 'fileutils'
require_relative 'benchmark_helper'
require_relative '../test/test_helper'

JSON_DIR = File.join(PROJECT_ROOT, 'benchmark', 'fixtures', 'json')

def item_to_json(item)
  {
    'id' => item[2].to_s,
    'lat' => item[0],
    'lon' => item[1],
    'quantities' => item[3].transform_keys(&:to_s).transform_values { |v| v.to_f },
    'v_id' => item[4][:v_id] || [],
    'skills' => item[4][:skills] || [],
    'day_skills' => item[4][:day_skills] || [],
    'matrix_index' => item[4][:matrix_index],
    'duration_from_and_to_depot' => item[4][:duration_from_and_to_depot] || []
  }
end

def vehicle_to_json(vehicle)
  depot = vehicle[:depot]
  {
    'id' => vehicle[:id] || [],
    'depot' => {
      'coordinates' => depot[:coordinates],
      'matrix_index' => depot[:matrix_index]
    }.compact,
    'capacities' => (vehicle[:capacities] || {}).transform_keys(&:to_s).transform_values { |v| v.to_f },
    'skills' => vehicle[:skills] || [],
    'day_skills' => vehicle[:day_skills] || [],
    'duration' => vehicle[:duration].to_f,
    'total_work_days' => vehicle[:total_work_days] || 1,
    'vehicle_count' => vehicle[:vehicle_count] || 1
  }
end

def export_from_bindump(name)
  clusterer, data_set, options, ratio = BenchmarkHarness.load_fixture(name)
  payload = {
    'name' => name,
    'seed' => BenchmarkHarness::DEFAULT_SEED,
    'cut_symbol' => (options[:cut_symbol] || :visits).to_s,
    'cut_ratio' => ratio,
    'max_iterations' => clusterer.max_iterations,
    'vehicles' => clusterer.vehicles.map { |v| vehicle_to_json(v) },
    'items' => data_set.data_items.map { |i| item_to_json(i) },
    'distance_matrix' => clusterer.distance_matrix,
    'centroid_indices' => options[:centroid_indices] || [],
    'related_item_indices' => (options[:related_item_indices] || {}).transform_keys(&:to_s)
  }
  write_json(name, payload)
end

def export_tiny
  clusterer, data_set = BenchmarkHarness.build_tiny_scenario
  payload = {
    'name' => 'tiny',
    'seed' => BenchmarkHarness::DEFAULT_SEED,
    'cut_symbol' => 'visits',
    'cut_ratio' => 1.0,
    'max_iterations' => clusterer.max_iterations,
    'vehicles' => clusterer.vehicles.map { |v| vehicle_to_json(v) },
    'items' => data_set.data_items.map { |i| item_to_json(i) },
    'distance_matrix' => nil,
    'centroid_indices' => [],
    'related_item_indices' => {}
  }
  write_json('tiny', payload)
end

def export_synthetic(n_items)
  clusterer, data_set, cut = BenchmarkHarness.synthetic_scenario(
    n_items: n_items,
    seed: BenchmarkHarness::DEFAULT_SEED
  )
  payload = {
    'name' => "scale_#{n_items}",
    'seed' => BenchmarkHarness::DEFAULT_SEED,
    'cut_symbol' => cut.to_s,
    'cut_ratio' => 1.0,
    'max_iterations' => clusterer.max_iterations,
    'vehicles' => clusterer.vehicles.map { |v| vehicle_to_json(v) },
    'items' => data_set.data_items.map { |i| item_to_json(i) },
    'distance_matrix' => nil,
    'centroid_indices' => [],
    'related_item_indices' => {}
  }
  write_json("scale_#{n_items}", payload)
end

def write_json(name, payload)
  path = File.join(JSON_DIR, "#{name}.json")
  File.write(path, JSON.pretty_generate(payload))
  puts "Exported #{path} (#{payload['items'].size} items, #{payload['vehicles'].size} vehicles)"
end

FileUtils.mkdir_p(JSON_DIR)

export_tiny
FIXTURES_DIR = File.join(PROJECT_ROOT, 'test', 'fixtures')
%w[length_centroid avoid_capacities_overlap].each { |f| export_from_bindump(f) if File.exist?(File.join(FIXTURES_DIR, "#{f}.bindump")) }
[100, 500, 1000].each { |n| export_synthetic(n) }

puts "\nDone. Fixtures in #{JSON_DIR}"
