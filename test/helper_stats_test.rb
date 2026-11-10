# Copyright © Cartoway
#
# frozen_string_literal: true

require './test/test_helper'
require 'logger'
require 'stringio'

class HelperStatsTest < Tests
  def test_output_cluster_stats_formats_rows_without_csv_gem
    io = StringIO.new
    logger = Logger.new(io)
    logger.level = Logger::DEBUG

    centroids = [
      [0.0, 0.0, 'c0', { duration: 3600.0 }, {
        vehicle_count: 1,
        capacities: { visits: 10 },
        skills: [],
        visit_count: 2,
        route_time: 0.0,
        duration_from_and_to_depot: 0.0,
        total_work_days: 1
      }]
    ]

    Helper.output_cluster_stats(centroids, logger)

    assert_match(/cluster_stats:/, io.string)
    assert_match(/zone,nb_vehicles,capacities/, io.string)
    assert_match(/1,1/, io.string)
  end
end
