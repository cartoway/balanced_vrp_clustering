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
require_relative '../benchmark/benchmark_helper'

class BenchmarkSmokeTest < Tests
  def test_benchmark_harness_tiny_completes_quickly
    clusterer, data_set = BenchmarkHarness.build_tiny_scenario
    elapsed = Benchmark.realtime do
      BenchmarkHarness.with_fixed_seed do
        clusterer.build(data_set, :visits, {}, 1.0, { seed: BenchmarkHarness::DEFAULT_SEED })
      end
    end

    assert_operator elapsed, :<, 5.0, "tiny build took #{elapsed.round(3)}s, expected < 5s"
    assert_equal 2, clusterer.clusters.size
    assert_equal 4, clusterer.clusters.sum { |c| c.data_items.size }
  end

  def test_benchmark_helper_loads
    assert defined?(BenchmarkHarness)
    assert_equal 42_424, BenchmarkHarness::DEFAULT_SEED
  end
end
