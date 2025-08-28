use criterion::{Criterion, criterion_group, criterion_main};
use ssz::{Decode, DecodeError, Encode};
use ssz_types::FixedVector;
use std::hint::black_box;
use typenum::{U131072, Unsigned};

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

#[derive(Clone, Debug, Default, PartialEq, Eq, ssz_derive::Encode)]
#[ssz(struct_behaviour = "transparent")]
pub struct FastU8(pub u8);

impl ssz::Decode for FastU8 {
    #[inline(always)]
    fn is_ssz_fixed_len() -> bool {
        true
    }

    #[inline(always)]
    fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() == 1 {
            Ok(FastU8(bytes[0]))
        } else {
            Err(DecodeError::BytesInvalid("invalid".to_string()))
        }
    }

    #[inline(always)]
    fn ssz_fixed_len() -> usize {
        1
    }
}

fn benchmark_fixed_vector_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_fixed_vector");

    let fixed_vector = FixedVector::<u8, U131072>::new(vec![255u8; 131072]).unwrap();
    let fixed_vector_bytes = fixed_vector.as_ssz_bytes();

    group.bench_function("u8_128k", |b| {
        b.iter(|| {
            let vector = FixedVector::<u8, U131072>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });

    group.bench_function("fast_u8_128k", |b| {
        b.iter(|| {
            let vector =
                FixedVector::<FastU8, U131072>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });

    group.bench_function("byte_u8_128k", |b| {
        b.iter(|| {
            let vector = ByteVector::<U131072>::from_ssz_bytes(&fixed_vector_bytes).unwrap();
            black_box(vector);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_fixed_vector_decode);
criterion_main!(benches);
