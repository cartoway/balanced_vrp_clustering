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

require 'benchmark/ips'
require_relative '../benchmark_helper'

def setup_clusterer(n_items: 200, k: 20)
  clusterer, data_set, cut = BenchmarkHarness.synthetic_scenario(n_items: n_items, k_vehicles: k)
  BenchmarkHarness.with_fixed_seed do
    clusterer.build(data_set, cut, {}, 1.0, { seed: BenchmarkHarness::DEFAULT_SEED })
  end
  [clusterer, data_set]
end

clusterer, data_set = setup_clusterer(n_items: 200, k: 20)
sample_item = data_set.data_items.first

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report('evaluate_one_item') do
    clusterer.send(:evaluate, sample_item)
  end

  x.report('distance_one_pair') do
    clusterer.send(:distance, sample_item, clusterer.instance_variable_get(:@centroids).first, 0)
  end

  x.compare!
end
