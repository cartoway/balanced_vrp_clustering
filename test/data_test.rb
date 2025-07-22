# Copyright © Mapotempo, 2020
#
# This file is part of Mapotempo.
#
# Mapotempo is free software. You can redistribute it and/or
# modify since you respect the terms of the GNU Affero General
# Public License as published by the Free Software Foundation,
# either version 3 of the License, or (at your option) any later version.
#
# Mapotempo is distributed in the hope that it will be useful, but WITHOUT
# ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
# or FITNESS FOR A PARTICULAR PURPOSE.  See the Licenses for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with Mapotempo. If not, see:
# <http://www.gnu.org/licenses/agpl.html>
#
require './test/test_helper'

class DataTest < Tests
  def test_data_with_matrix
    clusterer, data_set = Instance.two_clusters_4_items_with_matrix
    item_id = data_set.data_items.first[2]

    # make one duration_from_to_depot incorrect (too much values)
    data_set.data_items.first[4][:duration_from_and_to_depot] = [1, 2, 3]
    assert_raises ArgumentError do
      clusterer.build(data_set, :visits)
    end

    # make one duration_from_to_depot incorrect (too few values)
    data_set.data_items.first[4][:duration_from_and_to_depot] = [1]
    clusterer.build(data_set, :visits)

    item = data_set.data_items.find { |i| i[2] == item_id }

    item[4][:duration_from_and_to_depot] = [1]
    clusterer.build(data_set, :visits)
    # Verify that duration_from_and_to_depot is completed with 0s if too short
    assert_equal [1, 0], item[4][:duration_from_and_to_depot], "duration_from_and_to_depot should be completed with 0s if too short"

    # Verify that duration_from_and_to_depot has not been changed
    item[4][:duration_from_and_to_depot] = [1, 2]
    clusterer.build(data_set, :visits)
    assert_equal [1, 2], item[4][:duration_from_and_to_depot], "duration_from_and_to_depot should be unchanged if correct"

    data_set.data_items.first[4].delete(:matrix_index)
    assert_raises ArgumentError do
      clusterer.build(data_set, :visits)
    end
  end
end
