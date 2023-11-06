use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use smallvec::smallvec;
use ssz_types::BitList;
use typenum::U2048;

type BitList2048 = BitList<U2048>;

fn is_disjoint(c: &mut Criterion) {
    let x = BitList2048::from_raw_bytes(smallvec![0xff; 2048 / 8], 2048).unwrap();
    let y = BitList2048::from_raw_bytes(smallvec![0x00; 2048 / 8], 2048).unwrap();

    c.bench_with_input(
        BenchmarkId::new("bitfield_is_disjoint", 2048),
        &(x.clone(), y.clone()),
        |b, &(ref x, ref y)| {
            b.iter(|| assert!(x.is_disjoint(&y)));
        },
    );

    c.bench_with_input(
        BenchmarkId::new("bitfield_is_disjoint_by_intersection", 2048),
        &(x.clone(), y.clone()),
        |b, &(ref x, ref y)| {
            b.iter(|| assert!(x.intersection(&y).is_zero()));
        },
    );
}

criterion_group!(benches, is_disjoint);
criterion_main!(benches);
