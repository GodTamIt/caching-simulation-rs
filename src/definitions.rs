#![allow(dead_code)]
use std::fmt;

pub const DEFAULT_L1_SIZE: u32 = 10;
pub const DEFAULT_L2_SIZE: u32 = 15;
pub const DEFAULT_BLOCK_SIZE: u32 = 5;
pub const DEFAULT_BLOCKS_PER_SET: u32 = 3;

pub struct Stats {
    pub accesses: u64,
    pub reads: u64,
    pub read_misses: u64,
    pub writes: u64,
    pub write_misses: u64,
    pub misses: u64,
    pub write_backs: u64,

    pub l1_read_misses: u64,
    pub l1_write_misses: u64,

    pub l2_read_misses: u64,
    pub l2_write_misses: u64,

    pub l1_access_time: u64,
    pub l2_access_time: u64,
    pub memory_access_time: u64,

    pub l1_miss_rate: f64,
    pub l2_miss_rate: f64,
    pub miss_rate: f64,

    pub l2_avg_access_time: f64,
    pub avg_access_time: f64,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            accesses: 0,
            reads: 0,
            read_misses: 0,
            writes: 0,
            write_misses: 0,
            misses: 0,
            write_backs: 0,

            l1_read_misses: 0,
            l1_write_misses: 0,

            l2_read_misses: 0,
            l2_write_misses: 0,

            l1_access_time: 2,
            l2_access_time: 10,
            memory_access_time: 100,

            l1_miss_rate: 0.0,
            l2_miss_rate: 0.0,
            miss_rate: 0.0,

            l2_avg_access_time: 0.0,
            avg_access_time: 0.0,
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Cache Statistics").unwrap();
        writeln!(f, "Accesses: {}", self.accesses).unwrap();
        writeln!(f, "Reads: {}", self.reads).unwrap();
        writeln!(f, "Read misses: {}", self.read_misses).unwrap();
        writeln!(f, "Writes: {}", self.writes).unwrap();
        writeln!(f, "Write misses: {}", self.write_misses).unwrap();
        writeln!(f, "Misses: {}", self.misses).unwrap();
        writeln!(f, "Writebacks: {}", self.write_backs).unwrap();
        
        // L1 misses
        writeln!(f, "L1 read misses: {}", self.l1_read_misses).unwrap();
        writeln!(f, "L1 write misses: {}", self.l1_write_misses).unwrap();

        // L2 misses
        writeln!(f, "L2 read misses: {}", self.l2_read_misses).unwrap();
        writeln!(f, "L2 write misses: {}", self.l2_write_misses).unwrap();

        // Access times
        writeln!(f, "L1 access time: {}", self.l1_access_time).unwrap();
        writeln!(f, "L2 access time: {}", self.l2_access_time).unwrap();
        writeln!(f, "Memory access time: {}", self.memory_access_time).unwrap();

        // Miss rates
        writeln!(f, "L1 Miss rate: {}", self.l1_miss_rate).unwrap();
        writeln!(f, "L2 Miss rate: {}", self.l2_miss_rate).unwrap();
        writeln!(f, "Miss rate: {}", self.miss_rate).unwrap();

        // Average access times
        writeln!(f, "L2 average access time: {}", self.l2_avg_access_time).unwrap();
        write!(f, "Average access time (AAT): {}", self.avg_access_time)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessType {
    Read,
    Write
}