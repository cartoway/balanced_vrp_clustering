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

[100, 500].each do |n_items|
  clusterer, data_set, = BenchmarkHarness.synthetic_scenario(n_items: n_items)
  clusterer.instance_variable_set(:@data_set, data_set)
  clusterer.instance_variable_set(:@vehicles, clusterer.vehicles)

  puts "\n=== calculate_local_speeds n=#{n_items} ==="

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 1)

    x.report("local_speeds_n#{n_items}") do
      clusterer.send(:calculate_local_speeds)
    end
  end
end
