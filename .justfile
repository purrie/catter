home_path := env_var('HOME')
bin_path := home_path + "/bin/"
hidden_bin_path := home_path + "/.bin/"

user_install_dir := if path_exists(hidden_bin_path) == "true" { hidden_bin_path } else  { bin_path }

install_path := user_install_dir + "catter"
build_path := "./target/release/catter"


build:
    cargo build --release

install:
    #!/usr/bin/env sh
    if [ ! -f "{{build_path}}" ]; then
        cargo build --release
    fi
    cp "{{build_path}}" "{{install_path}}"

remove:
    #!/usr/bin/env sh
    if [ -f "{{install_path}}" ]; then
        rm "{{install_path}}"
    fi
