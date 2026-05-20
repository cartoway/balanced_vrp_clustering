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
require 'fileutils'

PROJECT_ROOT = File.expand_path('..', __dir__)
$LOAD_PATH.unshift(File.join(PROJECT_ROOT, 'lib'))
require 'balanced_vrp_clustering'
require 'helpers/helper'

include Ai4r::Data

module BenchmarkHarness
  DEFAULT_SEED = 42_424
  DEFAULT_RUNS = 3
  RESULTS_DIR = File.join(PROJECT_ROOT, 'benchmark', 'results')
  LATEST_PATH = File.join(RESULTS_DIR, 'latest.json')
  BASELINES_DIR = File.join(PROJECT_ROOT, 'benchmark', 'regression', 'baselines')
  FIXTURES_DIR = File.join(PROJECT_ROOT, 'test', 'fixtures')

  module_function

  def ensure_results_dir!
    FileUtils.mkdir_p(RESULTS_DIR)
    FileUtils.mkdir_p(BASELINES_DIR)
  end

  def runs_from_env
    (ENV['BENCHMARK_RUNS'] || DEFAULT_RUNS).to_i
  end

  def include_heavy_scenarios?
    ENV['BENCHMARK_HEAVY'] == '1'
  end

  def synthetic_sizes
    sizes = [100, 500, 1000]
    sizes << 5000 if include_heavy_scenarios?
    sizes
  end

  def with_fixed_seed(seed = DEFAULT_SEED)
    previous = Random.rand
    srand(seed)
    yield
  ensure
    srand(previous)
  end

  def measure_wall_time(runs: runs_from_env, &block)
    times = Array.new(runs) { Benchmark.realtime(&block) }
  end

  def summarize_times(times)
    sorted = times.sort
    {
      runs: times.size,
      min_s: sorted.first.round(6),
      max_s: sorted.last.round(6),
      mean_s: (times.sum / times.size).round(6),
      median_s: sorted[times.size / 2].round(6)
    }
  end

  def record_result(results, name, times, metadata = {})
    stats = summarize_times(times)
    entry = {
      name: name,
      **stats,
      **metadata
    }
    results << entry
    puts format_result_line(entry)
    entry
  end

  def format_result_line(entry)
    meta = []
    meta << "n=#{entry[:n]}" if entry[:n]
    meta << "k=#{entry[:k]}" if entry[:k]
    meta << "iter=#{entry[:iterations]}" if entry[:iterations]
    suffix = meta.empty? ? '' : " (#{meta.join(', ')})"
    format(
      '  %-28s  mean=%8.4fs  median=%8.4fs  min=%8.4fs%s',
      entry[:name],
      entry[:mean_s],
      entry[:median_s],
      entry[:min_s],
      suffix
    )
  end

  def write_latest!(results, type:)
    ensure_results_dir!
    payload = {
      type: type,
      recorded_at: Time.now.utc.iso8601,
      ruby_version: RUBY_VERSION,
      runs: runs_from_env,
      results: results
    }
    File.write(LATEST_PATH, JSON.pretty_generate(payload))
    puts "\nResults written to #{LATEST_PATH}"
    payload
  end

  def load_fixture(name)
    path = File.join(FIXTURES_DIR, "#{name}.bindump")
    raise "Fixture not found: #{path}" unless File.exist?(path)

    data_set, options, ratio = Marshal.load(File.binread(path))
    options = options.dup
    options[:seed] = DEFAULT_SEED
    options[:vehicles] ||= options.delete(:clusters_infos) || options.delete(:vehicles_infos)

    compute_duration_from_and_to_depot(options[:vehicles], data_set, options[:distance_matrix])

    clusterer = Ai4r::Clusterers::BalancedVRPClustering.new
    clusterer.max_iterations = options[:max_iterations]
    clusterer.distance_matrix = options[:distance_matrix]
    clusterer.vehicles = options[:vehicles]
    clusterer.centroid_indices = options[:centroid_indices] || []
    clusterer.logger = nil

    [clusterer, data_set, options, ratio]
  end

  def compute_duration_from_and_to_depot(vehicles, data_set, matrix)
    return if data_set.data_items.all? { |item| item[4][:duration_from_and_to_depot]&.size.to_f == vehicles.size }
    return if matrix.nil? || matrix.empty?

    data_set.data_items.each { |point| point[4][:duration_from_and_to_depot] = [] }

    vehicles.each do |vehicle_info|
      single_index_array = [vehicle_info[:depot][:matrix_index]]
      point_indices = data_set.data_items.map { |point| point[4][:matrix_index] }
      time_matrix_from_depot = Helper.unsquared_matrix(matrix, single_index_array, point_indices)
      time_matrix_to_depot = Helper.unsquared_matrix(matrix, point_indices, single_index_array)

      data_set.data_items.each_with_index do |point, index|
        point[4][:duration_from_and_to_depot] << time_matrix_from_depot[0][index] + time_matrix_to_depot[index][0]
      end
    end
  end

  def build_tiny_scenario
    clusterer = Ai4r::Clusterers::BalancedVRPClustering.new
    clusterer.max_iterations = 300
    clusterer.logger = nil
    clusterer.vehicles = [
      {
        id: ['vehicle_0'],
        depot: { coordinates: [45.604784, 4.758965] },
        capacities: { visits: 6 },
        skills: [],
        day_skills: ['all_days'],
        duration: 0,
        total_work_days: 1
      },
      {
        id: ['vehicle_1'],
        depot: { coordinates: [45.576412, 4.805614] },
        capacities: { visits: 6 },
        skills: [],
        day_skills: ['all_days'],
        duration: 0,
        total_work_days: 1
      }
    ]
    data_set = DataSet.new(
      data_items: [
        [45.604784, 4.758965, 'point_1', { visits: 1 }, { v_id: [], skills: [], day_skills: ['all_days'], duration_from_and_to_depot: [0.0, 4814.68] }],
        [45.344334, 4.817731, 'point_2', { visits: 1 }, { v_id: [], skills: [], day_skills: ['all_days'], duration_from_and_to_depot: [29_354.21, 25_852.47] }],
        [45.576412, 4.805614, 'point_3', { visits: 1 }, { v_id: [], skills: [], day_skills: ['all_days'], duration_from_and_to_depot: [4814.68, 0.0] }],
        [45.258324, 4.687322, 'point_4', { visits: 1 }, { v_id: [], skills: [], day_skills: ['all_days'], duration_from_and_to_depot: [38_972.24, 36_596.43] }]
      ]
    )
    [clusterer, data_set]
  end

  def synthetic_scenario(n_items:, k_vehicles: nil, cut_symbol: :visits, seed: DEFAULT_SEED)
    k = k_vehicles || [n_items / 10, 2].max
    rng = Random.new(seed)

    base_lat = 45.75
    base_lon = 4.85
    visits_per_vehicle = (n_items.to_f / k).ceil + 1

    vehicles = Array.new(k) do |i|
      angle = (2 * Math::PI * i) / k
      {
        id: ["vehicle_#{i}"],
        depot: {
          coordinates: [base_lat + 0.05 * Math.cos(angle), base_lon + 0.05 * Math.sin(angle)]
        },
        capacities: { visits: visits_per_vehicle },
        skills: [],
        day_skills: ['all_days'],
        duration: 0,
        total_work_days: 1
      }
    end

    data_items = Array.new(n_items) do |i|
      lat = base_lat + rng.rand(-0.15..0.15)
      lon = base_lon + rng.rand(-0.15..0.15)
      depot_durations = vehicles.map do |vehicle|
        Helper.flying_distance([lat, lon], vehicle[:depot][:coordinates]) * 10
      end
      [
        lat,
        lon,
        "point_#{i}",
        { visits: 1 },
        {
          v_id: [],
          skills: [],
          day_skills: ['all_days'],
          duration_from_and_to_depot: depot_durations
        }
      ]
    end

    clusterer = Ai4r::Clusterers::BalancedVRPClustering.new
    clusterer.max_iterations = [0.5 * n_items, 100].max
    clusterer.vehicles = vehicles
    clusterer.logger = nil

    [clusterer, DataSet.new(data_items: data_items), cut_symbol]
  end

  def run_build(clusterer, data_set, cut_symbol, options = {}, ratio = 1.0, related = {})
    iterations = 0
    with_fixed_seed do
      clusterer.build(data_set, cut_symbol, related, ratio, { seed: DEFAULT_SEED }.merge(options))
      iterations = clusterer.iteration
    end
    iterations
  end

  def run_integration_suite
    results = []
    puts "\n=== Integration benchmarks (#{runs_from_env} runs each, seed=#{DEFAULT_SEED}) ===\n"

    clusterer, data_set = build_tiny_scenario
    n = data_set.data_items.size
    k = clusterer.vehicles.size
    iterations = nil
    times = measure_wall_time do
      c, ds = build_tiny_scenario
      iterations = run_build(c, ds, :visits)
    end
    record_result(results, 'tiny', times, n: n, k: k, iterations: iterations)

    %w[length_centroid avoid_capacities_overlap].each do |fixture|
      sample_clusterer, sample_data, options, ratio = load_fixture(fixture)
      n = sample_data.data_items.size
      k = sample_clusterer.vehicles.size
      cut = options[:cut_symbol] || :visits
      iterations = nil
      times = measure_wall_time do
        c, ds, opts, r = load_fixture(fixture)
        iterations = run_build(c, ds, cut, opts, r)
      end
      record_result(results, fixture, times, n: n, k: k, iterations: iterations)
    end

    if include_heavy_scenarios? && File.exist?(File.join(FIXTURES_DIR, 'cluster_balance.bindump'))
      sample_clusterer, sample_data, options, ratio = load_fixture('cluster_balance')
      n = sample_data.data_items.size
      k = sample_clusterer.vehicles.size
      iterations = nil
      times = measure_wall_time do
        c, ds, opts, r = load_fixture('cluster_balance')
        iterations = run_build(c, ds, opts[:cut_symbol], opts, r)
      end
      record_result(results, 'cluster_balance', times, n: n, k: k, iterations: iterations)
    elsif !include_heavy_scenarios?
      puts '  (skip cluster_balance — set BENCHMARK_HEAVY=1 to include)'
    else
      warn 'Skipping cluster_balance: fixture not found'
    end

    synthetic_sizes.each do |n_items|
      sample_clusterer, = synthetic_scenario(n_items: n_items)
      k = sample_clusterer.vehicles.size
      iterations = nil
      times = measure_wall_time do
        c, ds, cut = synthetic_scenario(n_items: n_items)
        iterations = run_build(c, ds, cut)
      end
      record_result(results, "scale_#{n_items}", times, n: n_items, k: k, iterations: iterations)
    end

    write_latest!(results, type: 'integration')
    results
  end

  def save_baseline!
    ensure_results_dir!
    unless File.exist?(LATEST_PATH)
      raise "No latest results at #{LATEST_PATH}. Run benchmark:integration first."
    end

    timestamp = Time.now.utc.strftime('%Y%m%d_%H%M%S')
    baseline_path = File.join(BASELINES_DIR, "baseline_#{timestamp}.json")
    FileUtils.cp(LATEST_PATH, baseline_path)
    File.write(File.join(BASELINES_DIR, 'current.json'), File.read(LATEST_PATH))
    puts "Baseline saved to #{baseline_path}"
    baseline_path
  end

  def compare_regression!
    ensure_results_dir!
    baseline_path = File.join(BASELINES_DIR, 'current.json')
    unless File.exist?(baseline_path)
      raise "No baseline at #{baseline_path}. Run: rake benchmark:baseline"
    end
    unless File.exist?(LATEST_PATH)
      raise "No latest results at #{LATEST_PATH}. Run: rake benchmark:integration"
    end

    baseline = JSON.parse(File.read(baseline_path))
    current = JSON.parse(File.read(LATEST_PATH))

    baseline_by_name = baseline['results'].to_h { |r| [r['name'], r] }
    puts "\n=== Regression vs baseline ===\n"
    puts format('%-28s  baseline   current    delta%%', 'Scenario')
    puts '-' * 62

    current['results'].each do |entry|
      base = baseline_by_name[entry['name']]
      unless base
        puts format('%-28s  (new scenario)', entry['name'])
        next
      end

      delta_pct = ((entry['mean_s'] - base['mean_s']) / base['mean_s']) * 100.0
      puts format(
        '%-28s  %8.4fs  %8.4fs  %+7.1f%%',
        entry['name'],
        base['mean_s'],
        entry['mean_s'],
        delta_pct
      )
    end
    puts ''
  end
end
