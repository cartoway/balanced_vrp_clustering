# Copyright © Cartoway
#
# frozen_string_literal: true

require './test/test_helper'

# Golden values from Helper; keep in sync with rust/balanced_vrp_clustering_core/tests/integration.rs
class DistanceParityTest < Tests
  EPSILON_SCALE = 1e-9

  CASES = [
    { name: 'equator_lon', a: [0.0, 10.0], b: [0.0, 10.1], euclidean: 11_132.09999999996, flying: 11_132.09999999996 },
    { name: 'paris', a: [48.8566, 2.3522], b: [48.8606, 2.3376], euclidean: 1_158.316517608597, flying: 1_158.316517608597 },
    { name: 'near_equator', a: [0.5, 20.0], b: [0.5, 20.05], euclidean: 5_565.83806187141, flying: 5_565.83806187141 },
    { name: 'identity', a: [48.8566, 2.3522], b: [48.8566, 2.3522], euclidean: 0.0, flying: 0.0 },
    { name: 'mid_lat', a: [45.0, 4.0], b: [46.0, 5.0], euclidean: 135_942.66904027096, flying: 135_942.66904027096 }
  ].freeze

  def test_helper_euclidean_and_flying_match_golden_values
    CASES.each do |c|
      e = Helper.euclidean_distance(c[:a], c[:b])
      f = Helper.flying_distance(c[:a], c[:b])
      assert_near(e, c[:euclidean], "#{c[:name]} euclidean (Helper)")
      assert_near(f, c[:flying], "#{c[:name]} flying (Helper)")
    end
  end

  def test_rust_native_distances_match_helper
    skip 'native extension not built' unless native_distance_api?

    CASES.each do |c|
      rust_e = BalancedVRPClusteringNative.euclidean_distance(*c[:a], *c[:b])
      rust_f = BalancedVRPClusteringNative.flying_distance(*c[:a], *c[:b])
      ruby_e = Helper.euclidean_distance(c[:a], c[:b])
      ruby_f = Helper.flying_distance(c[:a], c[:b])
      assert_near(rust_e, ruby_e, "#{c[:name]} euclidean Ruby↔Rust")
      assert_near(rust_f, ruby_f, "#{c[:name]} flying Ruby↔Rust")
    end
  end

  def test_equator_longitude_distance_nonzero
    d = Helper.euclidean_distance([0.0, 10.0], [0.0, 10.1])
    assert_operator d, :>, 0.0
    assert_operator Helper.flying_distance([0.0, 10.0], [0.0, 10.1]), :>, 0.0
  end

  private

  def native_distance_api?
    require 'balanced_vrp_clustering_native'
    BalancedVRPClusteringNative.respond_to?(:euclidean_distance)
  rescue LoadError
    false
  end

  def assert_near(actual, expected, label)
    tol = [expected.abs, 1.0].max * EPSILON_SCALE
    assert((actual - expected).abs <= tol, "#{label}: got #{actual}, expected #{expected}")
  end
end