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

  def test_build_ruby_engine_explicit
    clusterer, data_set = Instance.two_clusters_4_items
    srand 42_424
    clusterer.build(data_set, :visits, {}, 1.0, { seed: 42_424, engine: :ruby })

    assert_equal 2, clusterer.clusters.size
    assert_equal 4, clusterer.clusters.sum { |c| c.data_items.size }
  end
end
