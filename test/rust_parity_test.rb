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

require './test/test_helper'
require 'json'
require 'open3'

class RustParityTest < Tests
  RUST_BIN = File.expand_path('../rust/target/release/bvrp-cluster', __dir__)
  TINY_JSON = File.expand_path('../benchmark/fixtures/json/tiny.json', __dir__)

  def test_rust_binary_exists
    skip 'Run: bundle exec rake rust:build' unless File.executable?(RUST_BIN)
    assert File.executable?(RUST_BIN)
  end

  def test_tiny_clusters_match_ruby
    skip 'Run: bundle exec rake benchmark:export_json && rake rust:build' unless fixtures_ready?

    ruby_clusters = ruby_clusters_from_tiny_json
    rust_clusters = rust_clusters_from_tiny_json

    assert clusters_equal?(ruby_clusters, rust_clusters),
           "Ruby: #{ruby_clusters.inspect}\nRust: #{rust_clusters.inspect}"
  end

  private

  def fixtures_ready?
    File.executable?(RUST_BIN) && File.exist?(TINY_JSON)
  end

  def tiny_json
    JSON.parse(File.read(TINY_JSON))
  end

  def ruby_clusters_from_tiny_json
    json = tiny_json
    clusterer = Ai4r::Clusterers::BalancedVRPClustering.new
    clusterer.max_iterations = json['max_iterations']
    clusterer.vehicles = json['vehicles'].map do |v|
      {
        id: v['id'],
        depot: { coordinates: v['depot']['coordinates'] },
        capacities: v['capacities'].transform_keys(&:to_sym),
        skills: v['skills'],
        day_skills: v['day_skills'],
        duration: v['duration'],
        total_work_days: v['total_work_days']
      }
    end
    items = json['items'].map do |it|
      [
        it['lat'], it['lon'], it['id'],
        it['quantities'].transform_keys(&:to_sym),
        {
          v_id: it['v_id'], skills: it['skills'], day_skills: it['day_skills'],
          duration_from_and_to_depot: it['duration_from_and_to_depot']
        }
      ]
    end
    srand json['seed']
    clusterer.build(Ai4r::Data::DataSet.new(data_items: items), json['cut_symbol'].to_sym, {}, 1.0, { seed: json['seed'] })
    clusterer.clusters.map { |c| c.data_items.map { |i| i[2] }.sort }
  end

  def rust_clusters_from_tiny_json
    out = File.join(Dir.tmpdir, "rust_parity_#{Process.pid}.json")
    _stdout, stderr, status = Open3.capture3(RUST_BIN, '--input', TINY_JSON, '--output', out)
    assert status.success?, stderr
    result = JSON.parse(File.read(out))
    result['clusters'].map(&:sort).sort
  ensure
    File.delete(out) if out && File.exist?(out)
  end

  def clusters_equal?(ruby, rust)
    ruby.sort == rust.sort
  end
end
