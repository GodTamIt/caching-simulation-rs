#![allow(dead_code)]

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

            l1_access_time: 0,
            l2_access_time: 0,
            memory_access_time: 0,

            l1_miss_rate: 0.0,
            l2_miss_rate: 0.0,
            miss_rate: 0.0,

            l2_avg_access_time: 0.0,
            avg_access_time: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AccessType {
    Read,
    Write
}