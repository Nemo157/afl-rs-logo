#![feature(question_mark)]

extern crate jpeg_decoder;
extern crate gif;
extern crate clap;

use std::io;
use std::fs::{ self, File };
use std::iter::FromIterator;

use jpeg_decoder as jpeg;
use gif::SetParameter;

trait LogError {
    type T;
    fn just_log_error(self) -> Option<Self::T>;
}

impl<V, E> LogError for Result<V, E> where E: std::error::Error {
    type T = V;
    fn just_log_error(self) -> Option<V> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                println!("whoops: {}", err);
                None
            }
        }
    }
}

fn write_frame<R: io::Read, W: io::Write>(mut image: jpeg::Decoder<R>, mut encoder: &mut gif::Encoder<W>) {
    let info = image.info().expect("Already read it");
    let data = image.decode().expect("Better be good");

    let frame = match info.pixel_format {
        jpeg::PixelFormat::RGB24 => gif::Frame::from_rgb(info.width, info.height, &data),
        jpeg::PixelFormat::L8 => {
            let mut new_data = Vec::with_capacity(data.len() * 4);
            for c in data {
                new_data.extend([c, c, c].iter());
            }
            gif::Frame::from_rgb(info.width, info.height, &mut new_data)
        }
        jpeg::PixelFormat::CMYK32 => unimplemented!()
    };

    encoder.write_frame(&frame).expect("What could go wrong?");
}

fn main() {
    let matches = clap::App::new("test_cases_to_gif")
        .arg(clap::Arg::with_name("original")
             .value_name("FILE")
             .help("The original file to get the size from")
             .required(true)
             .takes_value(true))
        .arg(clap::Arg::with_name("input")
             .value_name("DIRECTORY")
             .help("The directory containing jpg images to convert to a gif")
             .required(true)
             .takes_value(true))
        .arg(clap::Arg::with_name("output")
             .value_name("FILE")
             .help("The file to write the gif to")
             .required(true)
             .takes_value(true))
        .get_matches();

    let mut original = jpeg::Decoder::new(
        File::open(matches.value_of("original").expect("required"))
            .expect("Need this input file"));

    original.read_info().expect("Original must be valid");

    let info = original.info().expect("Already read it");

    let inputs = Vec::from_iter(
        fs::read_dir(matches.value_of("input").expect("required"))
            .expect("Need this directory")
            .filter_map(|entry| entry
                .and_then(|entry| File::open(entry.path()))
                .just_log_error()
                .map(|file| jpeg::Decoder::new(file))));

    let mut image = File::create(matches.value_of("output").expect("required"))
        .expect("Need this output file");
    let mut encoder = gif::Encoder::new(&mut image, info.width, info.height, &[])
        .expect("What could go wrong?");
    encoder.set(gif::Repeat::Infinite).expect("What could go wrong?");
    write_frame(original, &mut encoder);
    for input in inputs.into_iter().filter_map(|mut input| {
        input.read_info().just_log_error().map(|_| input)
    }).take(10) {
        let new_info = input.info().expect("Already read it");
        if (new_info.width, new_info.height) == (info.width, info.height) {
            write_frame(input, &mut encoder);
        }
    }
}
