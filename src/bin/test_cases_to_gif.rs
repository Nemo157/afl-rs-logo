#![feature(question_mark)]
#![feature(iter_arith)]
#![feature(inclusive_range_syntax)]

extern crate jpeg_decoder;
extern crate gif;
extern crate clap;

use std::iter;
use std::fs::{ self, File };
use std::path::PathBuf;
use std::borrow::Cow;

use jpeg_decoder as jpeg;
use gif::SetParameter;

trait LogError {
    type T;
    fn ok_or_log(self) -> Option<Self::T>;
}

struct Config {
    initial: PathBuf,
    input: PathBuf,
    output: PathBuf,
    frames: usize,
}

#[derive(Clone)]
struct Image {
    path: PathBuf,
    info: jpeg::ImageInfo,
    data: Vec<u8>,
}

impl<V, E> LogError for Result<V, E> where E: std::error::Error {
    type T = V;
    fn ok_or_log(self) -> Option<V> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                println!("whoops: {}", err);
                None
            }
        }
    }
}

macro_rules! maybe {
    ($e:expr) => {
        match $e { Some(e) => e, None => return None }
    }
}

impl Image {
    fn load(path: PathBuf) -> Option<Image> {
        let file = maybe!(File::open(&path).ok_or_log());
        let mut decoder = jpeg::Decoder::new(file);
        maybe!(decoder.read_info().ok_or_log());
        Some(Image {
            path: path,
            info: decoder.info().unwrap(),
            data: maybe!(decoder.decode().ok_or_log()),
        })
    }

    fn distance_from(&self, other: &Image) -> u64 {
        self.data.iter()
            .zip(other.data.iter())
            .map(|(&a, &b)| (a as i64, b as i64))
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<i64>() as u64
    }

    fn frame(self) -> gif::Frame<'static> {
        assert!(self.info.pixel_format == jpeg::PixelFormat::L8);
        gif::Frame {
            width: self.info.width,
            height: self.info.height,
            buffer: Cow::Owned(self.data),
            ..gif::Frame::default()
        }
    }
}

fn rescale(image: Image, info: jpeg::ImageInfo) -> Option<Image> {
    if image.info == info {
        return Some(image);
    }

    if image.info.pixel_format != jpeg::PixelFormat::L8 {
        println!("image {:?} is not grayscale", image.path);
        return None;
    }

    // println!("Converting from {}x{} to {}x{}", image.info.width, image.info.height, info.width, info.height);

    let data = {
        let image = &image;
        (0..info.width as usize).flat_map(move |y| {
            let mut y = ((y as f64 / info.height as f64) * image.info.height as f64).round() as usize;
            if y == image.info.height as usize { y -= 1; }
            (0..info.height as usize).map(move |x| {
                let mut x = ((x as f64 / info.width as f64) * image.info.width as f64).round() as usize;
                if x == image.info.width as usize { x -= 1; }
                image.data[x + y * image.info.width as usize]
            })
        }).collect()
    };

    Some(Image { path: image.path, info: info, data: data })
}

fn main() {
    let matches = clap::App::new("test_cases_to_gif")
        .args(&[
            clap::Arg::with_name("initial")
                 .value_name("FILE")
                 .help("The initial file to start the gif from")
                 .required(true)
                 .takes_value(true),
            clap::Arg::with_name("input")
                 .value_name("DIRECTORY")
                 .help("The directory containing jpg images to convert to a gif")
                 .required(true)
                 .takes_value(true),
            clap::Arg::with_name("output")
                 .value_name("FILE")
                 .help("The file to write the gif to")
                 .required(true)
                 .takes_value(true),
        ])
        .get_matches();

    let config = Config {
        initial: matches.value_of("initial").expect("required").into(),
        input: matches.value_of("input").expect("required").into(),
        output: matches.value_of("output").expect("required").into(),
        frames: 30,
    };

    let initial = Image::load(config.initial).expect("Need this input file");
    let (width, height) = (initial.info.width, initial.info.height);

    println!("Loaded initial file {:?}, {}x{} pixels", initial.path, width, height);

    let files = fs::read_dir(&config.input)
        .expect("Need this directory")
        .filter_map(|entry| entry.ok_or_log())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();

    let file_count = files.len();
    println!("Found {} files in input directory {:?}", files.len(), config.input);

    let mut images = files.into_iter()
        .filter_map(Image::load)
        .filter_map(|image| rescale(image, initial.info))
        .collect::<Vec<_>>();

    println!("Loaded {} of {} files", images.len(), file_count);

    print!("Choosing frames to use");
    // Do a greedy walk through the images from the initial one based on
    // distance from the current image, skipping any identical images
    let mut chosen = Vec::with_capacity(config.frames);
    chosen.extend(iter::repeat(initial.clone()).take(10));
    while chosen.len() < config.frames && !images.is_empty() {
        let local = |image: &Image, chosen: &[Image]| {
            image.distance_from(&chosen[chosen.len() - 1])
        };
        let global = |image: &Image| {
            image.distance_from(&initial)
        };
        let distance = |image: &Image, chosen: &[Image]| {
            local(image, chosen) / 2 + global(image)
        };
        let index = images.iter()
            .enumerate()
            .min_by_key(|&(_, image)| distance(image, &chosen))
            .map(|(index, _)| index)
            .unwrap();
        let image = images.swap_remove(index);
        if local(&image, &chosen) > 0 {
            if chosen.len() % 10 == 0 { println!(""); print!("   "); }
            chosen.push(image);
            print!(" {:3}..", chosen.len());
        }
    }
    println!("");
    println!("Chosen {} frames to use", chosen.len());

    let mut rgb_palette = Vec::with_capacity(256 * 3);
    for i in 0...255 {
        rgb_palette.extend([i, i, i].iter());
    }

    println!("Writing to {:?}", config.output);

    let mut image = File::create(config.output).expect("Need this output file");
    let mut encoder = gif::Encoder::new(&mut image, width, height, &rgb_palette)
        .expect("What could go wrong?");
    encoder.set(gif::Repeat::Infinite).expect("What could go wrong?");

    print!("Writing {} frames", chosen.len());
    for (i, image) in chosen.into_iter().enumerate() {
        if i % 10 == 0 { println!(""); print!("   "); }
        print!(" {:3}..", i + 1);
        encoder.write_frame(&image.frame()).expect("What could go wrong?");
    }
    println!("");
}
