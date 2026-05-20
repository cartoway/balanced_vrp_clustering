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

require 'json'

module Ai4r
  module Clusterers
    # Runs clustering via the Rust native extension and populates Ruby cluster state.
    module BalancedVRPClusteringRustEngine
        module_function

        def available?
          return @available if defined?(@available)

          require 'balanced_vrp_clustering_native'
          @available = BalancedVRPClusteringNative.available?
        rescue LoadError
          @available = false
        end

        def build(clusterer, data_set, cut_symbol, related_item_indices, cut_ratio, options)
          payload = InputSerializer.new(
            clusterer: clusterer,
            data_set: data_set,
            cut_symbol: cut_symbol,
            related_item_indices: clusterer.instance_variable_get(:@related_item_indices_for_rust) || related_item_indices,
            cut_ratio: cut_ratio,
            options: options
          ).to_h

          json_out = BalancedVRPClusteringNative.build_from_json(JSON.generate(payload))
          result = JSON.parse(json_out)

          apply_result(clusterer, data_set, result)
          clusterer
        end

        def apply_result(clusterer, data_set, result)
          clusterer.instance_variable_set(:@iteration, result['iterations'])
          items_by_id = data_set.data_items.index_by { |item| item[2].to_s }

          clusters = result['clusters'].map do |item_ids|
            ds = Ai4r::Data::DataSet.new(data_labels: data_set.data_labels)
            item_ids.each do |item_id|
              item = items_by_id[item_id.to_s]
              raise KeyError, "unknown item id #{item_id.inspect} in Rust output" unless item

              ds << item
            end
            ds
          end
          clusterer.instance_variable_set(:@clusters, clusters)
          centroids = centroids_from_vehicles(clusterer)
          sync_centroid_loads!(centroids, clusters)
          clusterer.instance_variable_set(:@centroids, centroids)
          clusterer.instance_variable_set(:@number_of_clusters, clusters.size)
          clusterer.instance_variable_set(:@balance_coeff, Array.new(clusters.size, 1.0))
        end

        def sync_centroid_loads!(centroids, clusters)
          centroids.each_with_index do |centroid, index|
            load = Hash.new(0.0)
            clusters[index].data_items.each do |item|
              item[3].each do |unit, qty|
                load[unit] += qty.to_f if qty
              end
            end
            centroid[3] = load
          end
        end

        def centroids_from_vehicles(clusterer)
          clusterer.vehicles.map do |vehicle|
            depot = vehicle[:depot] || {}
            coords = depot[:coordinates] || [0.0, 0.0]
            skills = vehicle.merge(
              matrix_index: depot[:matrix_index],
              duration_from_and_to_depot: depot[:duration_from_and_to_depot],
              route_time: 0,
              capacity_offence_coeff: 0,
              visit_count: 0
            )
            [
              coords[0],
              coords[1],
              Array(vehicle[:id]).flatten.first,
              Hash.new(0),
              skills
            ]
          end
        end

        # JSON payload shared with benchmark fixtures and the Rust CLI.
        class InputSerializer
          def initialize(clusterer:, data_set:, cut_symbol:, related_item_indices:, cut_ratio:, options:)
            @clusterer = clusterer
            @data_set = data_set
            @cut_symbol = cut_symbol
            @related_item_indices = related_item_indices
            @cut_ratio = cut_ratio
            @options = options
          end

          def to_h
            {
              'seed' => self.class.seed_to_u64(@options[:seed]),
              'cut_symbol' => @cut_symbol&.to_s,
              'cut_ratio' => @cut_ratio.to_f,
              'max_iterations' => @clusterer.max_iterations.to_i,
              'vehicles' => @clusterer.vehicles.map { |v| self.class.vehicle_to_json(v) },
              'items' => @data_set.data_items.map { |i| self.class.item_to_json(i) },
              'distance_matrix' => @clusterer.distance_matrix,
              'centroid_indices' => @clusterer.centroid_indices || [],
              'related_item_indices' => serialize_related_indices(@related_item_indices)
            }
          end

          def self.seed_to_u64(seed)
            seed.to_i & ((1 << 64) - 1)
          end

          def self.item_to_json(item)
            {
              'id' => item[2].to_s,
              'lat' => item[0],
              'lon' => item[1],
              'quantities' => item[3].transform_keys(&:to_s).transform_values { |v| v.to_f },
              'v_id' => item[4][:v_id] || [],
              'skills' => item[4][:skills] || [],
              'day_skills' => item[4][:day_skills] || [],
              'matrix_index' => item[4][:matrix_index],
              'duration_from_and_to_depot' => item[4][:duration_from_and_to_depot] || []
            }
          end

          def self.vehicle_to_json(vehicle)
            depot = vehicle[:depot] || {}
            {
              'id' => vehicle[:id] || [],
              'depot' => {
                'coordinates' => depot[:coordinates],
                'matrix_index' => depot[:matrix_index]
              }.compact,
              'capacities' => (vehicle[:capacities] || {}).transform_keys(&:to_s).transform_values { |v| v.to_f },
              'skills' => vehicle[:skills] || [],
              'day_skills' => vehicle[:day_skills] || [],
              'duration' => vehicle[:duration].to_f,
              'total_work_days' => vehicle[:total_work_days] || 1,
              'vehicle_count' => vehicle[:vehicle_count] || 1
            }
          end

          private

          def serialize_related_indices(related)
            related.transform_keys(&:to_s).transform_values do |groups|
              groups.map { |g| g.map(&:to_i) }
            end
          end
        end
    end
  end
end
