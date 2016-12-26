#![feature(plugin)]
#![plugin(afl_plugin)]

extern crate afl;
extern crate jpeg_decoder;

use jpeg_decoder::Decoder;

fn main() {
    afl::handle_read(|r| {
        Decoder::new(r).decode().unwrap();
    })
}
