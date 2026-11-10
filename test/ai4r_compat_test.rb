# Copyright © Cartoway
#
# frozen_string_literal: true

require './test/test_helper'

class Ai4rCompatTest < Tests
  def test_data_set_roundtrip_without_ai4r_gem
    refute defined?(Ai4r::Clusterers::KMeans), 'ai4r gem should not be loaded'

    items = [[1.0, 2.0, 'a', { visits: 1 }, { v_id: [] }]]
    data_set = Ai4r::Data::DataSet.new(data_items: items)

    assert_equal 1, data_set.data_items.size
    assert_equal 'a', data_set.data_items.first[2]

    empty = Ai4r::Data::DataSet.new(data_labels: data_set.data_labels)
    empty << items.first
    assert_equal 1, empty.data_items.size
  end

  def test_clusterer_parameters_still_accessible
    clusterer = Ai4r::Clusterers::BalancedVRPClustering.new
    clusterer.max_iterations = 12
    clusterer.centroid_indices = [0]
    clusterer.vehicles = [{ id: 'v1' }]

    assert_equal 12, clusterer.max_iterations
    assert_equal [0], clusterer.centroid_indices
    assert_equal [{ id: 'v1' }], clusterer.vehicles
  end

  def test_marshal_fixture_loads_as_local_data_set
    path = File.expand_path('fixtures/length_centroid.bindump', __dir__)
    data_set, = Marshal.load(File.binread(path))

    assert_instance_of Ai4r::Data::DataSet, data_set
    assert data_set.data_items.size.positive?
  end
end
