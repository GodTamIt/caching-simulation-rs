#[macro_use] extern crate clap;

mod student;
mod definitions;

use clap::{Arg, App};
use definitions as defs;
use std::io;
use std::io::prelude::BufRead;

fn main() {
    // Lines are necessary until lexical lifetimes
    let l1_size_str: &str = &defs::DEFAULT_L1_SIZE.to_string();
    let l2_size_str: &str = &defs::DEFAULT_L2_SIZE.to_string();
    let block_size_str: &str = &defs::DEFAULT_BLOCK_SIZE.to_string();
    let blocks_per_set_str: &str = &defs::DEFAULT_BLOCKS_PER_SET.to_string();

    let matches = App::new("Cache Simulation")
                    .version(env!("CARGO_PKG_VERSION"))
                    .author(env!("CARGO_PKG_AUTHORS"))
                    .arg(Arg::with_name("l1-size")
                        .visible_alias("c1")
                        .default_value(l1_size_str))
                    .arg(Arg::with_name("l2-size")
                        .visible_alias("c2")
                        .default_value(l2_size_str))
                    .arg(Arg::with_name("block-size")
                        .short("b")
                        .default_value(block_size_str))
                    .arg(Arg::with_name("blocks-per-set")
                        .short("s")
                        .default_value(blocks_per_set_str))
                    .get_matches();

    let l1_size = value_t!(matches.value_of("l1-size"), u64).unwrap_or_else(|e| e.exit());
    let l2_size = value_t!(matches.value_of("l2-size"), u64).unwrap_or_else(|e| e.exit());
    let block_size = value_t!(matches.value_of("block-size"), u64).unwrap_or_else(|e| e.exit());
    let blocks_per_set = value_t!(matches.value_of("blocks-per-set"), u64).unwrap_or_else(|e| e.exit());

    println!("Cache Settings");
    println!("C1: {}", l1_size);
    println!("C2: {}", l2_size);
    println!("B: {}", block_size);
    println!("S: {}", blocks_per_set);
    println!();

    let mut stats = defs::Stats::new();
    let (mut config, mut l1, mut l2) = student::init(l1_size, l2_size, block_size, blocks_per_set);
    
    simulate_accesses_stdin(&mut config, &mut l1, &mut l2, &mut stats);
    
    student::finish(&mut stats);

    println!("{}", stats);
}

fn simulate_accesses_stdin(config: &mut student::Config, l1: &mut student::L1, l2: &mut student::L2, stats: &mut defs::Stats) {
    let mut access_type: defs::AccessType;
    let mut address: u64;
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let mut iter = line.split_whitespace();

        match iter.next() {
            Some("r") => access_type = defs::AccessType::Read,
            Some("w") => access_type = defs::AccessType::Write,
            _ => continue,
        }

        let address_string: &str;
        match iter.next() {
            Some(v) => address_string = v,
            _ => continue,
        }

        match u64::from_str_radix(&address_string[2..], 16) {
            Ok(v) => address = v,
            _ => continue,
        }

        student::cache_access(access_type, address, config, l1, l2, stats);
    }
}