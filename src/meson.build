cargo_sources = files(
  '../Cargo.toml',
  '../Cargo.lock',
)

rust_sources = files(
  'parse_config/mod.rs',
  'parse_config/yaml_config.rs',

  'app.rs',
  'egui_data.rs',
  'espanso_yaml.rs',
  'main.rs',
  'style.rs',
)

sources = [cargo_sources, rust_sources]

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
