use std::io::{Cursor, Read};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::{rngs::SmallRng, Rng, SeedableRng};

fn take_u64(input: &mut &[u8]) -> Option<u64> {
    let (num, rem) = input.split_at_checked(8)?;
    *input = rem;
    Some(u64::from_le_bytes(num.try_into().unwrap())) // SAFETY: Known to be the right length
}

fn take_u64_unchecked(input: &mut &[u8]) -> u64 {
    let (num, rem) = input.split_at(8);
    *input = rem;
    u64::from_le_bytes(num.try_into().unwrap()) // SAFETY: Known to be the right length
}

fn bench_read_u64(c: &mut Criterion) {
    const SIZE: usize = 8192;
    const SEED: u64 = 42;

    let mut rng = SmallRng::seed_from_u64(SEED);
    let data: Vec<_> = (0..SIZE)
        .map(|_| rng.random::<u64>().to_le_bytes())
        .flatten()
        .collect();

    let slice: &[u8] = data.as_ref();
    c.bench_with_input(BenchmarkId::new("u64_take", data.len()), &slice, |b, d| {
        let mut out = 0;
        b.iter(|| {
            let mut data = d.as_ref();
            for _ in 0..SIZE {
                out = take_u64(&mut data).unwrap();
            }
        });
    });

    let slice: &[u8] = data.as_ref();
    c.bench_with_input(BenchmarkId::new("u64_take_unchecked", data.len()), &slice, |b, d| {
        let mut out = 0;
        b.iter(|| {
            let mut data = d.as_ref();
            for _ in 0..SIZE {
                out = take_u64_unchecked(&mut data);
            }
        });
    });
}

criterion_group!(benches, bench_read_u64);
criterion_main!(benches);
