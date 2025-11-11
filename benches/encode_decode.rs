use criterion::{criterion_group, criterion_main, Criterion};
use ssz::{Decode, Encode};
use ssz_types::{FixedVector, VariableList};
use std::hint::black_box;
use std::time::Duration;
use typenum::{U1048576, U131072};

fn benchmark_fixed_vector(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixed_vector");

    let fixed_vector_u8 = FixedVector::<u8, U1048576>::new(vec![255u8; 1048576]).unwrap();
    let fixed_vector_u64 = FixedVector::<u64, U131072>::new(vec![255u64; 131072]).unwrap();
    let fixed_vector_bytes = fixed_vector_u8.as_ssz_bytes();

    group.warm_up_time(Duration::from_secs(15));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("decode_u8_1m", |b| {
        b.iter(|| {
            let vector = FixedVector::<u8, U1048576>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });

    group.bench_function("encode_u8_1m", |b| {
        b.iter(|| {
            let bytes = fixed_vector_u8.as_ssz_bytes();
            black_box(bytes);
        });
    });

    group.bench_function("decode_u64_128k", |b| {
        b.iter(|| {
            let vector = FixedVector::<u64, U131072>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });
    group.bench_function("encode_u64_128k", |b| {
        b.iter(|| {
            let bytes = fixed_vector_u64.as_ssz_bytes();
            black_box(bytes);
        });
    });

    group.finish();
}

fn benchmark_variable_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("variable_list");

    let variable_list_u8 = VariableList::<u8, U1048576>::new(vec![255u8; 1048576]).unwrap();
    let variable_list_u64 = VariableList::<u64, U131072>::new(vec![255u64; 131072]).unwrap();
    let variable_list_bytes = variable_list_u8.as_ssz_bytes();

    group.warm_up_time(Duration::from_secs(15));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("decode_u8_1m", |b| {
        b.iter(|| {
            let vector =
                VariableList::<u8, U1048576>::from_ssz_bytes(&variable_list_bytes).unwrap();
            black_box(vector);
        });
    });

    group.bench_function("encode_u8_1m", |b| {
        b.iter(|| {
            let bytes = variable_list_u8.as_ssz_bytes();
            black_box(bytes);
        });
    });

    group.bench_function("decode_u64_128k", |b| {
        b.iter(|| {
            let vector =
                VariableList::<u64, U131072>::from_ssz_bytes(&variable_list_bytes).unwrap();
            black_box(vector);
        });
    });
    group.bench_function("encode_u64_128k", |b| {
        b.iter(|| {
            let bytes = variable_list_u64.as_ssz_bytes();
            black_box(bytes);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_fixed_vector, benchmark_variable_list);
criterion_main!(benches);
