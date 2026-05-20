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
require 'helpers/hull'

def random_vector(size, rng)
  Array.new(size) { [rng.rand(4.0..5.0), rng.rand(45.0..46.0)] }
end

rng = Random.new(BenchmarkHarness::DEFAULT_SEED)

[20, 100, 500].each do |size|
  vector = random_vector(size, rng)
  puts "\n=== Hull.get_hull size=#{size} ==="

  Benchmark.ips do |x|
    x.config(time: 5, warmup: 1)

    x.report("hull_#{size}") do
      Hull.get_hull(vector)
    end
  end
end
