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

loc_a = [45.604784, 4.758965]
loc_b = [45.344334, 4.817731]
matrix = [
  [0, 2824, 1110],
  [2780, 0, 2132],
  [1174, 2212, 0]
]

Benchmark.ips do |x|
  x.config(time: 5, warmup: 2)

  x.report('flying_distance') do
    Helper.flying_distance(loc_a, loc_b)
  end

  x.report('matrix_lookup_symmetric') do
    a_idx = 1
    b_idx = 2
    [matrix[a_idx][b_idx], matrix[b_idx][a_idx]].min
  end

  x.compare!
end
