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

require 'benchmark'
require 'json'
require 'open3'
require 'fileutils'
require_relative 'benchmark_helper'
require_relative '../test/test_helper'

include Ai4r::Data

JSON_DIR = File.join(PROJECT_ROOT, 'benchmark', 'fixtures', 'json')
COMPARE_PATH = File.join(BenchmarkHarness::RESULTS_DIR, 'compare_latest.json')
RUST_BIN = File.join(
  PROJECT_ROOT,
  'rust',
  'target',
  'release',
  'bvrp-cluster'
)

def rust_available?
  File.executable?(RUST_BIN)
end

def normalize_clusters(clusters)
  clusters.map { |c| c.sort }.sort
end

def clusters_match?(ruby_clusters, rust_clusters)
  normalize_clusters(ruby_clusters) == normalize_clusters(rust_clusters)
end

def load_json_fixture(name)
  JSON.parse(File.read(File.join(JSON_DIR, "#{name}.json")))
end

def build_ruby_from_json(json)
  clusterer = Ai4r::Clusterers::BalancedVRPClustering.new
  clusterer.max_iterations = json['max_iterations']
  clusterer.distance_matrix = json['distance_matrix']
  clusterer.vehicles = json['vehicles'].map do |v|
    depot = v['depot']
    {
      id: v['id'],
      depot: {
        coordinates: depot['coordinates'],
        matrix_index: depot['matrix_index']
      }.compact,
      capacities: v['capacities'].transform_keys(&:to_sym),
      skills: v['skills'],
      day_skills: v['day_skills'],
      duration: v['duration'],
      total_work_days: v['total_work_days'],
      vehicle_count: v['vehicle_count'] || 1
    }
  end
  items = json['items'].map do |it|
    [
      it['lat'],
      it['lon'],
      it['id'],
      it['quantities'].transform_keys(&:to_sym),
      {
        v_id: it['v_id'],
        skills: it['skills'],
        day_skills: it['day_skills'],
        matrix_index: it['matrix_index'],
        duration_from_and_to_depot: it['duration_from_and_to_depot']
      }
    ]
  end
  related = (json['related_item_indices'] || {}).transform_keys(&:to_sym).transform_values do |groups|
    groups.map { |g| g }
  end
  data_set = DataSet.new(data_items: items)
  cut = json['cut_symbol']&.to_sym || :visits
  BenchmarkHarness.with_fixed_seed(json['seed']) do
    clusterer.build(data_set, cut, related, json['cut_ratio'] || 1.0, { seed: json['seed'] })
  end
  clusterer
end

def run_rust(json_path)
  out = File.join(Dir.tmpdir, "bvrp_rust_#{Process.pid}.json")
  stdout, stderr, status = Open3.capture3(RUST_BIN, '--input', json_path, '--output', out)
  raise "Rust failed: #{stderr}\n#{stdout}" unless status.success?

  result = JSON.parse(File.read(out))
  File.delete(out)
  result
end

def scenario_names
  names = %w[tiny length_centroid avoid_capacities_overlap]
  BenchmarkHarness.synthetic_sizes.each { |n| names << "scale_#{n}" }
  names.select { |n| File.exist?(File.join(JSON_DIR, "#{n}.json")) }
end

def measure_ruby(json)
  clusterer = nil
  time = Benchmark.realtime do
    clusterer = build_ruby_from_json(json)
  end
  clusters = clusterer.clusters.map { |c| c.data_items.map { |i| i[2] } }
  { time: time, iterations: clusterer.iteration, clusters: clusters }
end

def measure_rust(json_path, json)
  time = Benchmark.realtime do
    @rust_result = run_rust(json_path)
  end
  {
    time: time,
    iterations: @rust_result['iterations'],
    clusters: @rust_result['clusters'],
    elapsed_ms: @rust_result['elapsed_ms']
  }
end

BenchmarkHarness.ensure_results_dir!

unless rust_available?
  warn "Rust binary not found at #{RUST_BIN}. Run: bundle exec rake rust:build"
  exit 1
end

runs = BenchmarkHarness.runs_from_env
results = []

puts "\n=== Ruby vs Rust comparison (#{runs} runs, seed=#{BenchmarkHarness::DEFAULT_SEED}) ===\n"
puts format('%-28s %10s %10s %8s %10s', 'Scenario', 'Ruby(s)', 'Rust(s)', 'Speedup', 'Parity')
puts '-' * 72

scenario_names.each do |name|
  json_path = File.join(JSON_DIR, "#{name}.json")
  json = load_json_fixture(name)

  ruby_times = runs.times.map { measure_ruby(json)[:time] }
  rust_times = runs.times.map { measure_rust(json_path, json)[:time] }

  ruby_mean = ruby_times.sum / ruby_times.size
  rust_mean = rust_times.sum / rust_times.size
  speedup = ruby_mean / rust_mean

  ruby_sample = measure_ruby(json)
  rust_sample = measure_rust(json_path, json)
  parity = clusters_match?(ruby_sample[:clusters], rust_sample[:clusters]) ? 'match' : 'DIFF'

  entry = {
    name: name,
    n: json['items'].size,
    k: json['vehicles'].size,
    ruby_mean_s: ruby_mean.round(6),
    rust_mean_s: rust_mean.round(6),
    speedup: speedup.round(2),
    ruby_iterations: ruby_sample[:iterations],
    rust_iterations: rust_sample[:iterations],
    parity: parity,
    ruby_clusters: ruby_sample[:clusters],
    rust_clusters: rust_sample[:clusters]
  }
  results << entry

  puts format(
    '%-28s %10.4f %10.4f %8.2fx %10s',
    name,
    ruby_mean,
    rust_mean,
    speedup,
    parity
  )
end

payload = {
  compared_at: Time.now.utc.iso8601,
  ruby_version: RUBY_VERSION,
  runs: runs,
  scenarios: results
}
File.write(COMPARE_PATH, JSON.pretty_generate(payload))
puts "\nWritten to #{COMPARE_PATH}"

required_parity = %w[tiny length_centroid]
failed = required_parity.select do |name|
  entry = results.find { |r| r[:name] == name }
  entry.nil? || entry[:parity] != 'match'
end
if results.any? { |r| r[:parity] == 'DIFF' }
  puts "\nNote: assignment differences on some scenarios (RNG / floating-point; see JSON)."
end
if failed.any?
  warn "Required parity failed for: #{failed.join(', ')}"
  exit 1
end
