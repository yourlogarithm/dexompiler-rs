#![feature(test)]
extern crate test;

mod utils;
mod dex_parsing;
mod manifest_parsing;
mod apk;
mod cli;

use clap::Parser;
use std::{fs::OpenOptions, sync::{Mutex, Arc}, collections::HashMap};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Serialize, Serializer};
use indicatif::ParallelProgressIterator;
use std::io::BufWriter;

use cli::Args;
use apk::ApkParseModel;


#[derive(Debug)]
pub struct MutexWrapper<T: ?Sized>(pub Mutex<T>);

impl<T: ?Sized + Serialize> Serialize for MutexWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .lock()
            .expect("mutex is poisoned")
            .serialize(serializer)
    }
}


fn main() {
    let args = Args::parse();

    println!("Parsing {} files up to {} opcodes, using {} threads", args.input.len(), args.sequence_cap, args.threads);

    rayon::ThreadPoolBuilder::new().num_threads(args.threads).build_global().unwrap();
    let accumulator = Arc::new(MutexWrapper(Mutex::new(HashMap::new())));
    args.input.par_iter().progress_count(args.input.len() as u64).for_each(|path| {
        if let Ok(apk) = ApkParseModel::try_from_path(path, args.sequence_cap) {
            let mut accumulator = accumulator.0.lock().unwrap();
            accumulator.insert(path, apk);
        } else {
            eprintln!("Error parsing: {}", path);
        }
    });

    println!("Writing to file");

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(args.output)
        .unwrap();
    let buffered_file = BufWriter::new(file);

    let output = Arc::try_unwrap(accumulator).unwrap().0.into_inner().unwrap();

    serde_json::to_writer(buffered_file, &output).unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn benchmark_parsing(b: &mut Bencher) {
        b.iter(|| ApkParseModel::try_from_path("resources/F-Droid.apk", 0).unwrap());
    }
}