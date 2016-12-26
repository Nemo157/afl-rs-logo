#![feature(question_mark)]

extern crate jpeg_decoder;
extern crate gif;
extern crate clap;
extern crate rand;

use std::fs::{ self, File };
use std::path::PathBuf;

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
}

struct Image {
    path: PathBuf,
    info: jpeg::ImageInfo,
    data: Vec<u8>,
    score: i32,
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
            score: rand::random::<u16>() as i32, // -1,
        })
    }
}

impl Into<gif::Frame<'static>> for Image {
    fn into(self) -> gif::Frame<'static> {
        let data = match self.info.pixel_format {
            jpeg::PixelFormat::RGB24 => self.data,
            jpeg::PixelFormat::CMYK32 => unimplemented!(),
            jpeg::PixelFormat::L8 => {
                let mut data = Vec::with_capacity(self.data.len() * 3);
                for c in self.data {
                    data.extend([c, c, c].iter());
                }
                data
            }
        };
        gif::Frame::from_rgb(self.info.width, self.info.height, &data)
    }
}

fn main() {
    let matches = clap::App::new("test_cases_to_gif")
        .args(&[
            clap::Arg::with_name("initial")
                 .value_name("FILE")
                 .help("The initial file to get the size from and base image distance off")
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
        .collect::<Vec<_>>();

    println!("Loaded {} of {} files", images.len(), file_count);

    println!("Choosing frames to use");

    // TODO: actually choose
    images.sort_by(|a, b| b.score.cmp(&a.score));
    let chosen = images.into_iter()
        .take(10)
        .collect::<Vec<_>>();

    let chosen_count = chosen.len();
    println!("Chosen {} frames to use", chosen.len());

    println!("Writing to {:?}", config.output);

    let mut image = File::create(config.output).expect("Need this output file");
    let mut encoder = gif::Encoder::new(&mut image, width, height, &[])
        .expect("What could go wrong?");
    encoder.set(gif::Repeat::Infinite).expect("What could go wrong?");

    println!("Writing initial frame from {:?}", initial.path);

    encoder.write_frame(&initial.into()).expect("What could go wrong?");

    for (i, image) in chosen.into_iter().enumerate() {
        if (image.info.width, image.info.height) == (width, height) {
            println!("Writing frame {}/{} from {:?} (score {})", i, chosen_count, image.path, image.score);
            encoder.write_frame(&image.into()).expect("What could go wrong?");
        }
    }
}
