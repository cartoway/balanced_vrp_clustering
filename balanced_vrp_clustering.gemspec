Gem::Specification.new do |s|
  s.name = 'balanced_vrp_clustering'
  s.version = '0.3.0'
  s.date = '2026-05-20'
  s.summary = 'Gem for clustering points of a given VRP.'
  s.authors = 'Cartoway'
  s.files = Dir[
    'lib/**/*.rb',
    'rust/Cargo.toml',
    'rust/Cargo.lock',
    'rust/balanced_vrp_clustering_core/**/*',
    'rust/balanced_vrp_clustering_native/**/*'
  ].reject do |f|
    f.include?('/target/') ||
      f.end_with?('.so', '.bundle') ||
      f.end_with?('/Makefile') ||
      f.end_with?('mkmf.log')
  end
  s.require_paths = %w[lib]
  s.extensions = ['rust/balanced_vrp_clustering_native/extconf.rb']

  s.add_dependency 'ai4r'
  s.add_dependency 'activesupport'
  s.add_dependency 'rake'
  s.add_dependency 'rb_sys', '~> 0.9.128'

  s.add_dependency 'color-generator' # for geojson debug output
  s.add_dependency 'geojson2image'   # for geojson debug output
  s.add_dependency 'awesome_print'   # for geojson debug output
end
