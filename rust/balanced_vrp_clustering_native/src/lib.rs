// Copyright © Cartoway
//
// This file is part of balanced_vrp_clustering.
//
// Cartoway balanced_vrp_clustering is free software. You can redistribute it and/or
// modify since you respect the terms of the GNU Affero General
// Public License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any later version.
//
// Cartoway Planner is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
// or FITNESS FOR A PARTICULAR PURPOSE.  See the Licenses for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Cartoway Planner. If not, see:
// <http://www.gnu.org/licenses/agpl.html>
//

use balanced_vrp_clustering_core::{build_from_input, InputConfig};
use magnus::{function, prelude::*, Error, Ruby, RString};

fn build_from_json(input: RString) -> Result<RString, Error> {
    let ruby = Ruby::get().expect("Ruby not initialized");
    let json_str = input
        .to_string()
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.to_string()))?;

    let config: InputConfig = serde_json::from_str(&json_str).map_err(|e| {
        Error::new(
            ruby.exception_arg_error(),
            format!("invalid JSON input: {e}"),
        )
    })?;

    let output = build_from_input(&config);
    let out_json = serde_json::to_string(&output).map_err(|e| {
        Error::new(
            ruby.exception_runtime_error(),
            format!("failed to serialize output: {e}"),
        )
    })?;

    Ok(RString::new(&out_json))
}

/// Returns true when the native extension loaded successfully.
fn available() -> bool {
    true
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("BalancedVRPClusteringNative")?;
    module.define_singleton_method("build_from_json", function!(build_from_json, 1))?;
    module.define_singleton_method("available?", function!(available, 0))?;
    Ok(())
}
