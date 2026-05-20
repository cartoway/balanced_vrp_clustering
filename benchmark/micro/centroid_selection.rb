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

def cluster_items(n_items: 200, k: 20)
  clusterer, data_set, cut = BenchmarkHarness.synthetic_scenario(n_items: n_items, k_vehicles: k)
  BenchmarkHarness.with_fixed_seed do
    clusterer.build(data_set, cut, {}, 1.0, { seed: BenchmarkHarness::DEFAULT_SEED })
  end
  cluster = clusterer.clusters.max_by { |c| c.data_items.size }
  centroid = clusterer.centroids[clusterer.clusters.index(cluster)]
  [cluster, centroid]
end

[50, 200].each do |n_items|
  cluster, centroid = cluster_items(n_items: n_items, k: 20)
  n_c = cluster.data_items.size
  puts "\n=== centroid representative selection n_c=#{n_c} ==="

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 1)

    x.report('current_two_pass_min_by') do
      sample_size = [[(n_c / 10.0).ceil, 5].min, 2].max
      point_closest = cluster.data_items.min_by(sample_size) { |data_point|
        Helper.flying_distance(centroid, data_point)
      }.min_by { |data_point|
        cluster.data_items.sum { |d_i|
          Helper.flying_distance(data_point, d_i) * d_i[3][:visits]
        }
      }
      point_closest[2]
    end

    x.report('single_pass_nearest') do
      cluster.data_items.min_by { |data_point|
        Helper.flying_distance(centroid, data_point)
      }[2]
    end

    x.compare!
  end
end
