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

require 'rbconfig'

ROOT = File.expand_path('..', __dir__)
NATIVE_CRATE_DIR = File.join(ROOT, 'rust', 'balanced_vrp_clustering_native')
LIB_DIR = File.join(ROOT, 'lib')
DLEXT = RbConfig::CONFIG['DLEXT']
SO_NAME = "balanced_vrp_clustering_native.#{DLEXT}"

# After gem install, rb_sys places the shared lib next to extconf (rust/.../crate).
SEARCH_PATHS = [
  File.join(LIB_DIR, SO_NAME),
  File.join(NATIVE_CRATE_DIR, SO_NAME),
  File.join(NATIVE_CRATE_DIR, 'target', 'release', SO_NAME)
].freeze

path = SEARCH_PATHS.find { |p| File.exist?(p) }
raise LoadError, "balanced_vrp_clustering_native not built. Run: bundle exec rake native:compile" unless path

require path
