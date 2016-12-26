# afl-rs-logo

An attempt at creating a logo for [afl.rs][] similar to the [(unofficial?) afl
logo][afl-logo].

## Running

### Setup

```sh
# Pull the afl.rs docker image, image id e92ac6651d11 currently
docker pull corey/afl.rs

# Start a semi-permanent container to run all the build commands in
docker run -dv $(pwd):/source --name afl corey/afl.rs sh -c 'while true; do sleep 1; done'

# Build the executables
docker exec -it afl cargo build
```

### Generating potential images with AFL

Leave this running for a while until `out/queue` has enough source images for
the next step to work with.

```sh
# Run afl
docker exec -it afl afl-fuzz -i in -o out target/debug/check_jpeg
```

### Choosing images and putting them into a gif

WIP: Need to implement the choosing and ordering stage still, for now you can
manually filter the `out/queue` folder for interesting looking images and get
something that sorta works.

```sh
# Generate the gif from the test cases
docker exec -it afl target/debug/test_cases_to_gif in/rust-logo-blk.jpg out/queue temp.gif
```

[afl.rs]: https://github.com/frewsxcv/afl.rs
[afl-logo]: http://lcamtuf.coredump.cx/afl/rabbit.gif
