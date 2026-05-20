require 'rubygems'
require 'bundler/setup'
require 'fileutils'

require 'rake/testtask'
Rake::TestTask.new do |t|
  ENV['APP_ENV'] ||= 'test'
  t.pattern = 'test/**/*_test.rb'
end

namespace :benchmark do
  desc 'Run all benchmarks (micro + integration)'
  task all: %i[micro integration]

  desc 'Run micro-benchmarks (benchmark-ips)'
  task :micro do
    Dir[File.join(__dir__, 'benchmark/micro/*.rb')].sort.each do |path|
      puts "\n#{'=' * 60}\n#{File.basename(path)}\n#{'=' * 60}"
      ruby path
    end
  end

  desc 'Run integration benchmarks and write benchmark/results/latest.json'
  task :integration do
    ruby File.join(__dir__, 'benchmark/integration/build_all.rb')
  end

  desc 'Save latest.json as regression baseline'
  task :baseline do
    ruby File.join(__dir__, 'benchmark/regression/save_baseline.rb')
  end

  desc 'Compare latest.json against saved baseline (run benchmark:integration first)'
  task :regression do
    ruby File.join(__dir__, 'benchmark/regression/compare.rb')
  end
end

desc 'Alias for benchmark:all'
task benchmark: 'benchmark:all'

namespace :rust do
  desc 'Build Rust bvrp-cluster binary (release)'
  task :build do
    sh 'cargo build --release --manifest-path rust/Cargo.toml -p balanced_vrp_clustering_core --features cli'
  end
end

namespace :native do
  desc 'Compile Ruby native extension (Rust FFI)'
  task :compile do
    native_dir = File.join(__dir__, 'rust', 'balanced_vrp_clustering_native')
    Dir.chdir(native_dir) do
      sh 'ruby', 'extconf.rb'
      sh 'make'
    end
    so = "balanced_vrp_clustering_native.#{RbConfig::CONFIG['DLEXT']}"
    built = File.join(native_dir, so)
    lib_dst = File.join(__dir__, 'lib', so)
    FileUtils.cp(built, lib_dst) if File.exist?(built)
    puts "Native extension: #{lib_dst}" if File.exist?(lib_dst)
  end
end

desc 'Compile native extension'
task compile: 'native:compile'

namespace :benchmark do
  desc 'Export JSON fixtures from Ruby (bindump + synthetic)'
  task :export_json do
    ruby File.join(__dir__, 'benchmark/export_fixture_json.rb')
  end

  desc 'Compare Ruby vs Rust on JSON fixtures'
  task compare_rust: 'rust:build' do
    ruby File.join(__dir__, 'benchmark/compare_ruby_rust.rb')
  end
end
