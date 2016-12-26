#![feature(plugin)]
#![plugin(afl_plugin)]

#![feature(question_mark)]

extern crate afl;
extern crate jpeg_decoder;

use jpeg_decoder::Decoder;

fn main() {
    afl::handle_read(|r| {
        Decoder::new(r).decode().unwrap();
    })
}
