use criterion::{criterion_group, criterion_main, Criterion};
use ssz::{Decode, DecodeError, Encode};
use ssz_types::FixedVector;
use std::hint::black_box;
use typenum::{Unsigned, U131072, U16384};

#[derive(Clone, Debug, Default, PartialEq, Eq, ssz_derive::Encode)]
#[ssz(struct_behaviour = "transparent")]
pub struct ByteVector<N: Unsigned>(FixedVector<u8, N>);

impl<N: Unsigned> ssz::Decode for ByteVector<N> {
    fn is_ssz_fixed_len() -> bool {
        true
    }

    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        FixedVector::new(bytes.to_vec())
            .map(Self)
            .map_err(|e| DecodeError::BytesInvalid(format!("{e:?}")))
    }

    fn ssz_fixed_len() -> usize {
        <FixedVector<u8, N> as ssz::Decode>::ssz_fixed_len()
    }
}

fn benchmark_fixed_vector_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_fixed_vector");

    let fixed_vector_u8 = FixedVector::<u8, U131072>::new(vec![255u8; 131072]).unwrap();
    let fixed_vector_u64 = FixedVector::<u64, U16384>::new(vec![255u64; 16384]).unwrap();
    let fixed_vector_bytes = fixed_vector_u8.as_ssz_bytes();

    group.bench_function("decode_u8_128k", |b| {
        b.iter(|| {
            let vector = FixedVector::<u8, U131072>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });

    group.bench_function("encode_u8_128k", |b| {
        b.iter(|| {
            let bytes = fixed_vector_u8.as_ssz_bytes();
            black_box(bytes);
        });
    });

    group.bench_function("decode_byte_u8_128k", |b| {
        b.iter(|| {
            let vector = ByteVector::<U131072>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });

    group.bench_function("decode_u64_16k", |b| {
        b.iter(|| {
            let vector = FixedVector::<u64, U16384>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });
    group.bench_function("encode_u64_16k", |b| {
        b.iter(|| {
            let bytes = fixed_vector_u64.as_ssz_bytes();
            black_box(bytes);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_fixed_vector_decode);
criterion_main!(benches);
