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
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bvrp-cluster")]
#[command(about = "Balanced VRP clustering (Rust core)")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();
    let json = fs::read_to_string(&cli.input).expect("read input");
    let input: InputConfig = serde_json::from_str(&json).expect("parse input");
    let result = build_from_input(&input);
    let out_json = serde_json::to_string_pretty(&result).expect("serialize output");
    if let Some(path) = cli.output {
        fs::write(path, out_json).expect("write output");
    } else {
        println!("{out_json}");
    }
}
