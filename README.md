# afl-logo-rs

An attempt at creating a logo for [afl.rs][] similar to the afl logo.

## Running

```sh
# Pull the afl.rs docker image, image id e92ac6651d11 currently
docker pull corey/afl.rs

# Start a semi-permanent container to run all the build commands in
docker run -dv $(pwd):/source --name afl corey/afl.rs sh -c 'while true; do sleep 1; done'

# Build the executables
docker exec -it afl cargo build

# Run afl
docker exec -it afl afl-fuzz -i in -o out target/debug/check_jpeg
```

[afl.rs]: https://github.com/frewsxcv/afl.rs
