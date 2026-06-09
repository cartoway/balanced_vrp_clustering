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

class FfiEngineTest < Tests
  def test_native_extension_available
    skip 'Run: bundle exec rake native:compile' unless Ai4r::Clusterers::BalancedVRPClusteringRustEngine.available?

    assert BalancedVRPClusteringNative.available?
  end

  def test_build_uses_rust_engine_by_default
    skip 'Run: bundle exec rake native:compile' unless Ai4r::Clusterers::BalancedVRPClusteringRustEngine.available?

    clusterer, data_set = Instance.two_clusters_4_items
    srand 42_424
    clusterer.build(data_set, :visits, {}, 1.0, { seed: 42_424 })

    assert_equal 2, clusterer.clusters.size
    assert_equal 4, clusterer.clusters.sum { |c| c.data_items.size }
    refute_includes clusterer.clusters.map { |c| c.data_items.size }, 0

    expected = [%w[point_1 point_3], %w[point_2 point_4]].map(&:sort).sort
    actual = clusterer.clusters.map { |c| c.data_items.map { |i| i[2] }.sort }.sort
    assert_equal expected, actual
  end

  def test_build_accepts_integer_vehicle_ids
    skip 'Run: bundle exec rake native:compile' unless Ai4r::Clusterers::BalancedVRPClusteringRustEngine.available?

    payload = Ai4r::Clusterers::BalancedVRPClusteringRustEngine::InputSerializer.new(
      clusterer: Ai4r::Clusterers::BalancedVRPClustering.new.tap do |c|
        c.vehicles = [{
          id: [1],
          depot: { coordinates: [0.0, 0.0] },
          capacities: { visits: 10 },
          skills: [],
          day_skills: %w[0_day_skill],
          duration: 100,
          total_work_days: 1
        }]
        c.max_iterations = 10
      end,
      data_set: Ai4r::Data::DataSet.new(data_items: [[0.0, 0.0, 'p1', { visits: 1 }, { v_id: [2], skills: [], day_skills: %w[0_day_skill] }]]),
      cut_symbol: :visits,
      related_item_indices: {},
      cut_ratio: 1.0,
      options: { seed: 42 }
    ).to_h

    assert_equal ['1'], payload['vehicles'].first['id']
    assert_equal ['2'], payload['items'].first['v_id']
  end

  def test_rust_build_exposes_cut_limit
    skip 'Run: bundle exec rake native:compile' unless Ai4r::Clusterers::BalancedVRPClusteringRustEngine.available?

    clusterer, data_set = Instance.two_clusters_4_items
    clusterer.build(data_set, :visits, {}, 1.0, { seed: 42_424 })

    assert clusterer.cut_limit
    assert_equal clusterer.vehicles.size, clusterer.cut_limit.size
    assert clusterer.cut_limit.all? { |entry| entry.key?(:limit) }
  end

  def test_build_tolerates_nil_numeric_fields_in_items
    skip 'Run: bundle exec rake native:compile' unless Ai4r::Clusterers::BalancedVRPClusteringRustEngine.available?

    clusterer, data_set = Instance.two_clusters_4_items
    item = data_set.data_items.first
    item[0] = nil
    item[1] = nil
    item[4][:duration_from_and_to_depot] = [nil, 10.0]

    srand 42_424
    clusterer.build(data_set, :visits, {}, 1.0, { seed: 42_424 })

    assert_equal 4, clusterer.clusters.sum { |c| c.data_items.size }
  end

  def test_output_cluster_stats_after_rust_build
    skip 'Run: bundle exec rake native:compile' unless Ai4r::Clusterers::BalancedVRPClusteringRustEngine.available?

    clusterer, data_set = Instance.two_clusters_4_items
    clusterer.logger = Logger.new(IO::NULL)
    srand 42_424
    clusterer.build(data_set, :visits, {}, 1.0, { seed: 42_424 })

    Helper.output_cluster_stats(clusterer.instance_variable_get(:@centroids), clusterer.logger)
  end

  def test_build_ruby_engine_explicit
    clusterer, data_set = Instance.two_clusters_4_items
    srand 42_424
    clusterer.build(data_set, :visits, {}, 1.0, { seed: 42_424, engine: :ruby })

    assert_equal 2, clusterer.clusters.size
    assert_equal 4, clusterer.clusters.sum { |c| c.data_items.size }
  end
end
