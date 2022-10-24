use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};
#[repr(align(64))]
struct Cacheline {
    data: u32,
    _padding: [u8; 60],
}
fn init(count: usize) -> Vec<Cacheline> {
    let mut v = Vec::new();
    for i in 0..count {
        v.push(Cacheline {
            data: i as u32,
            _padding: [0; 60],
        });
    }
    v
}

fn clear_cache(mem: &mut Vec<Cacheline>) {
    let mut rng = thread_rng();
    for i in 0..mem.len() {
        mem[i].data = rng.gen();
    }
}
fn test(mem: &mut Vec<Cacheline>, size: usize, count: usize) {
    clear_cache(mem);
    let start = Instant::now();
    let mut acc = u32::MAX;
    for j in 0..count {
        for i in 0..size {
            acc ^= mem[i].data | j as u32;
        }
    }
    let end = Instant::now();
    let elapsed = end - start;
    let ns = elapsed.as_secs_f64() as f64 * 1e9 / (count as f64 * size as f64);
    println!(
        "total:{:.5}s size: {:12} read latency: {:.3}ns unused({})",
        elapsed.as_secs_f64() as f64,
        pretty_print_size(size * std::mem::size_of::<Cacheline>()),
        ns,
        acc
    );
}
fn pretty_print_size(size: usize) -> String {
    if size <= 1024 {
        format!("{}B", size)
    } else if size <= 1024 * 1024 {
        format!("{}KB", size / 1024)
    } else if size <= 1024 * 1024 * 1024 {
        format!("{}MB", size / 1024 / 1024)
    } else {
        format!("{}GB", size / 1024 / 1024 / 1024)
    }
}
#[repr(align(64))]
struct PtrChain {
    ptr: *const PtrChain,
    data: u32,
}
fn init_ptr_chain(count: usize) -> Vec<PtrChain> {
    let mut v = Vec::new();
    let mut rng = thread_rng();
    for i in 0..count {
        v.push(PtrChain {
            ptr: std::ptr::null_mut(),
            data: rng.gen(),
        });
    }

    v
}
fn build_ptr_chain(v: &mut Vec<PtrChain>, count: usize) {
    let mut indices = (0..count).collect::<Vec<_>>();
    let mut rng = thread_rng();
    indices.shuffle(&mut rng);
    for i in 0..count {
        v[indices[i]].ptr = &v[indices[(i + 1) % count]] as *const _;
    }
}
fn run_ptr_chain(chain: &mut Vec<PtrChain>, size: usize, count: usize) {
    let mut rng = thread_rng();
    for i in 0..size {
        chain[i].data = rng.gen();
    }
    build_ptr_chain(chain, size);
    let mut ptr = &chain[0] as *const PtrChain;
    macro_rules! run1 {
        () => {
            ptr = (*ptr).ptr;
        };
    }
    macro_rules! run5 {
        () => {
            run1!();
            run1!();
            run1!();
            run1!();
            run1!();
        };
    }
    macro_rules! run20 {
        () => {
            run5!();
            run5!();
            run5!();
            run5!();
        };
    }
    macro_rules! run100 {
        () => {
            run20!();
            run20!();
            run20!();
            run20!();
            run20!();
        };
    }
    macro_rules! run200 {
        () => {
            run100!();
            run100!();
        };
    }
    let start = Instant::now();
    for _ in 0..count {
        unsafe {
            run200!();
        }
    }
    let end = Instant::now();
    let elapsed = end - start;
    let acc = unsafe { (*ptr).data };

    let ns = elapsed.as_nanos() as f64 / (count as f64 * 200.0);
    println!(
        "total:{:.5}s size: {:12} read latency: {:.3}ns unused({})",
        elapsed.as_secs_f64() as f64,
        pretty_print_size(size * std::mem::size_of::<PtrChain>()),
        ns,
        acc
    );
}
fn test_seq() {
    assert!(std::mem::size_of::<Cacheline>() == 64);
    let mut mem0 = init((1024 * 1024 * 1024 * 2usize) / std::mem::size_of::<Cacheline>());
    let mut mem1 = init((1024 * 1024 * 1024 * 2usize) / std::mem::size_of::<Cacheline>());
    let mut size = 64;
    while size <= mem0.len() {
        test(&mut mem0, size, 64);
        std::mem::swap(&mut mem0, &mut mem1);
        size *= 2;
    }
}
fn test_random() {
    assert!(std::mem::size_of::<PtrChain>() == 64);
    let mut chain = init_ptr_chain(1024 * 1024 * 64usize / std::mem::size_of::<PtrChain>());
    let mut size = 64;
    while size <= chain.len() {
        run_ptr_chain(&mut chain, size, 1024 * 64);
        size *= 2;
    }
}
fn main() {
    test_seq();
    test_random();
}
