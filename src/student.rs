use definitions::{AccessType, Stats};
use std::vec::Vec;

/// The stuct that you may use to store the metadata for each block in the L1 and L2 caches.
#[derive(Clone, Debug)]
struct CacheBlock {
    tag: u64,
    valid: bool,
    dirty: bool,

    // Add another variable here to perform the LRU replacement.
    // Look into using a counter variable that will keep track of the oldest block in a set.
    last_clock_access: u64,
}

impl CacheBlock {
    pub fn new() -> CacheBlock {
        CacheBlock {
            tag: 0,
            valid: false,
            dirty: false,
            last_clock_access: 0,
        }
    }
}

#[allow(non_snake_case)]
pub struct Config {
    C1: u64,
    C2: u64,
    B: u64,
    S: u64,

    cache_clock: u64,
}

pub struct L1 {
    cache: Vec<CacheBlock>,
}

impl L1 {
    pub fn new(c: u64, b: u64) -> L1 {
        let total_blocks = (1 << (c - b)) as usize;
        L1 {
            cache: vec!(CacheBlock::new(); total_blocks)
        }
    }
}

pub struct L2 {
    cache: Vec<Vec<CacheBlock>>,
}

impl L2 {
    pub fn new(c: u64, b: u64, s: u64) -> L2 {
        let blocks_per_set = (1 << (c - b - s)) as usize;
        let num_sets = (1 << s) as usize;

        // Initialize 2-D array for L2 cache
        let mut l2_cache: Vec<Vec<CacheBlock>> = Vec::with_capacity(blocks_per_set);
        for _ in 0..blocks_per_set {
            let cache_line: Vec<CacheBlock> = vec!(CacheBlock::new(); num_sets);
            l2_cache.push(cache_line);
        }

        L2 {
            cache: l2_cache
        }
    }
}

/// Initialize cache with the passed in arguments
pub fn init(c1: u64, c2: u64, b: u64, s: u64) -> (Config, L1, L2) {
    let config = Config {
        C1: c1,
        C2: c2,
        B: b,
        S: s,
        cache_clock: 0,
    };

    let l1_cache = L1::new(c1, b);
    let l2_cache = L2::new(c2, b, s);

    (config, l1_cache, l2_cache)
}

pub fn cache_access(access_type: AccessType, address: u64, config: &mut Config, l1: &mut L1, l2: &mut L2, stats: &mut Stats) {
    // Check that cache_clock hasn't hit its limit yet and increment the cache_clock before each access
    if config.cache_clock == u64::max_value() {
        panic!("The cache clock has hit the unsigned 64-bit integer max value and will overflow!");
    }
    config.cache_clock += 1;

    // Update cache-wide stats
    stats.accesses += 1;
    match access_type {
        AccessType::Read => stats.reads += 1,
        AccessType::Write => stats.writes += 1,
    }

    let l1_tag = get_tag(address, config.C1, 0);
    let l1_index = get_index(address, config.C1, config.B, 0);

    /**************** L1 Cache Lookup ******************/
    {
        let l1_block = &mut l1.cache[l1_index as usize];
        if l1_block.valid && l1_block.tag == l1_tag {
            l1_block.last_clock_access = config.cache_clock;
            l1_block.dirty |= access_type == AccessType::Write;

            // Update corresponding L2 entry access time as well
            let (l2_set_index, _, l2_index) = find_l2_block(l1_tag, l1_index, &config, &l2).unwrap();
            l2.cache[l2_index as usize][l2_set_index].last_clock_access = config.cache_clock;
            
            return;
        }
    }

    // Update L1 miss stats
    match access_type {
        AccessType::Read => stats.l1_read_misses += 1,
        AccessType::Write => stats.l1_write_misses += 1,
    }

    /**************** L2 Cache Lookup ******************/
    match find_l2_block(l1_tag, l1_index, &config, &l2) {
        Some((l2_set_index, _, l2_index)) => {
            l2.cache[l2_index as usize][l2_set_index].last_clock_access = config.cache_clock;
            
            // Bring block into L1 cache as well
            allocate_l1_block(l1, l2, l1_index, &config);
            update_block(&mut l1.cache[l1_index as usize], l1_tag, true, access_type == AccessType::Write, config.cache_clock);

            return;
        },
        None => (),
    };

    // Update L2 miss stats
    match access_type {
        AccessType::Read => stats.l2_read_misses += 1,
        AccessType::Write => stats.l2_write_misses += 1,
    }

    /**************** Bring block from memory ******************/
    let l2_tag = convert_tag_l1_to_l2(l1_tag, l1_index, config);
    let l2_index = convert_index_l1_to_l2(l1_tag, l1_index, config);

    let l2_victim_set_index = find_l2_victim(l2_index, &l2);
    {
        let l2_victim = &mut l2.cache[l2_index as usize][l2_victim_set_index];

        // L2 victim is valid and must be evicted
        if l2_victim.valid {
            let l1_victim_tag = convert_tag_l2_to_l1(l2_victim.tag, l2_index, config);
            let l1_victim_index = convert_index_l2_to_l1(l2_victim.tag, l2_index, config);

            // l2 victim has a corresponding block in L1 that must also be evicted
            {
                let l1_victim = &mut l1.cache[l1_victim_index as usize];
                if l1_victim.valid && l1_victim.tag == l1_victim_tag {
                    l1_victim.valid = false;
                    
                    if l1_victim.dirty || l2_victim.dirty {
                        stats.write_backs += 1;
                    }
                } else if l2_victim.dirty {
                    stats.write_backs += 1;
                }
            }
        }

        update_block(l2_victim, l2_tag, true, access_type == AccessType::Write, config.cache_clock);
    }

    // Bring block in L1 cache
    allocate_l1_block(l1, l2, l1_index, config);
    // Note: L1 is not dirty because L2 should be dirty
    update_block(&mut l1.cache[l1_index as usize], l1_tag, true, false, config.cache_clock);
}

fn allocate_l1_block(l1: &mut L1, l2: &mut L2, l1_index: u64, config: &Config) {
    let l1_block = &l1.cache[l1_index as usize];

    if l1_block.valid && l1_block.dirty {
        let l2_victim_tag = convert_tag_l1_to_l2(l1_block.tag, l1_index, config);
        let l2_victim_index = convert_index_l1_to_l2(l1_block.tag, l1_index, config);

        for l2_block in &mut l2.cache[l2_victim_index as usize] {
            if l2_block.valid && l2_block.tag == l2_victim_tag {
                l2_block.dirty = true;
                break;
            }
        }

    }
}

fn find_l2_block(l1_tag: u64, l1_index: u64, config: &Config, l2: &L2) -> Option<(usize, u64, u64)> {
    let l2_tag = convert_tag_l1_to_l2(l1_tag, l1_index, config);
    let l2_index = convert_index_l1_to_l2(l1_tag, l1_index, config);

    for i in 0..l2.cache[l2_index as usize].len() {
        let l2_block = &l2.cache[l2_index as usize][i];
        if l2_block.valid && l2_block.tag == l2_tag {
            return Some((i, l2_tag, l2_index));
        }
    }

    None
}

fn find_l2_victim(l2_index: u64, l2: &L2) -> usize {
    let l2_index = l2_index as usize;
    let mut lru_time = u64::max_value();
    let mut lru_index: usize = 0;

    for i in 0..l2.cache[l2_index as usize].len() {
        let l2_block = &l2.cache[l2_index as usize][i];
        if !l2_block.valid {
            return i;
        } else if l2_block.last_clock_access < lru_time {
            lru_time = l2_block.last_clock_access;
            lru_index = i;
        }
    }

    lru_index
}

fn update_block(block: &mut CacheBlock, tag: u64, valid: bool, dirty: bool, last_clock_access: u64) {
    block.tag = tag;
    block.valid = valid;
    block.dirty = dirty;
    block.last_clock_access = last_clock_access;
}

/// Perform any final calculations before the statistics are outputted by the driver.
pub fn finish(stats: &mut Stats) {
    stats.read_misses = stats.l1_read_misses + stats.l2_read_misses;
    stats.write_misses = stats.l1_write_misses + stats.l2_write_misses;
    stats.misses = stats.read_misses + stats.write_misses;
    stats.l1_miss_rate = (stats.l1_read_misses + stats.l1_write_misses) as f64 / stats.accesses as f64;

    let l2_accesses: f64 = (stats.l1_read_misses + stats.l1_write_misses) as f64;
    stats.l2_miss_rate = (stats.l2_read_misses + stats.l2_write_misses) as f64 / l2_accesses;
    stats.miss_rate = stats.misses as f64 / stats.accesses as f64;
    stats.l2_avg_access_time = stats.l2_access_time as f64 + stats.l2_miss_rate * stats.memory_access_time as f64;
    stats.avg_access_time = stats.l1_access_time as f64 + stats.l1_miss_rate * stats.l2_avg_access_time;
}


/// Computes the tag of a given address based on the parameters passed in
fn get_tag(address: u64, c: u64, s: u64) -> u64 {
    address >> (c - s)
}


/// Subroutine to compute the Index of a given address based on the parameters passed in
fn get_index(address: u64, c: u64, b: u64, s: u64) -> u64 {
    (address >> b) & ((1u64 << (c - b - s)) - 1)
}


/**** DO NOT MODIFY CODE BELOW THIS LINE UNLESS YOU ARE ABSOLUTELY SURE OF WHAT YOU ARE DOING ****/

/*
    Note:   The below functions will be useful in converting the L1 tag and index into corresponding L2
            tag and index. These should be used when you are evicitng a block from the L1 cache, and
            you need to update the block in L2 cache that corresponds to the evicted block.

            The newly added functions will be useful for converting L2 indecies ang tags into the corresponding
            L1 index and tags. Make sure to understand how they are working.
*/


/// Converts the tag stored in an L1 block and the index of that L1 block into corresponding tag of the L2 block
fn convert_tag_l1_to_l2(tag: u64, index: u64, config: &Config) -> u64 {
    let reconstructed_address = (tag << (config.C1 - config.B)) | index;
    reconstructed_address >> (config.C2 - config.B - config.S)
}

/// Converts the tag stored in an L1 block and the index of that L1 block into corresponding index of the L2 block
fn convert_index_l1_to_l2(tag: u64, index: u64, config: &Config) -> u64 {
    // Reconstructed address without the block offset bits
    let reconstructed_address = (tag << (config.C1 - config.B)) | index;
    // Create index mask for L2 without including the block offset bits
    reconstructed_address & ((1u64 << (config.C2 - config.B - config.S)) - 1)
}


/// Converts the tag stored in an L2 block and the index of that L2 block into corresponding tag of the L1 cache
fn convert_tag_l2_to_l1(tag: u64, index: u64, config: &Config) -> u64 {
    let reconstructed_address = (tag << (config.C2 - config.B - config.S)) | index;
    reconstructed_address >> (config.C1 - config.B)
}


/// Converts the tag stored in an L2 block and the index of that L2 block into corresponding index of the L1 block
fn convert_index_l2_to_l1(tag: u64, index: u64, config: &Config) -> u64 {
    let reconstructed_address = (tag << (config.C2 - config.B - config.S)) | index;
    reconstructed_address & ((1u64 << (config.C1 - config.B)) - 1)
}
