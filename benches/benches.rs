use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};

use splynters::SplinterWrapper;

use rand::{Rng, thread_rng};


fn main() {}


// fn generate_random_splinter_wrapper(size: usize) -> SplinterWrapper {
//     let mut rng = thread_rng();
//     let bytes: Bytes = (0..size).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>().into();
//     SplinterWrapper::new(bytes)
// }

// fn run_splynters_benchmarks(c: &mut Criterion) {
    // let mut group = c.benchmark_group("SplinterWrapper And Operations");
    //
    // for size in [1024, 10 * 1024, 100 * 1024].iter() { // 1KB, 10KB, 100KB
    //     group.throughput(criterion::Throughput::Bytes(*size as u64));
    //
    //     // Benchmark for and_simple
    //     group.bench_with_input(criterion::BenchmarkId::new("and_simple", size), size, |b, &size| {
    //         b.iter_batched(
    //             || {
    //                 let s1 = generate_random_splinter_wrapper(size);
    //                 let s2 = generate_random_splinter_wrapper(size);
    //                 (s1, s2)
    //             },
    //             |(mut s1, s2)| {
    //                 s1.and_simple(s2).unwrap()
    //             },
    //             criterion::BatchSize::SmallInput,
    //         );
    //     });
    //
    //     // Benchmark for and_chunked
    //     group.bench_with_input(criterion::BenchmarkId::new("and_chunked", size), size, |b, &size| {
    //         b.iter_batched(
    //             || {
    //                 let s1 = generate_random_splinter_wrapper(size);
    //                 let s2 = generate_random_splinter_wrapper(size);
    //                 (s1, s2)
    //             },
    //             |(mut s1, s2)| {
    //                 s1.and_chunked(s2).unwrap()
    //             },
    //             criterion::BatchSize::SmallInput,
    //         );
    //     });
    // }
    // group.finish();
    //
    // let mut group = c.benchmark_group("Splinter Setup Cost");
    //
    // for size in [1024, 10 * 1024, 100 * 1024].iter() { // 1KB, 10KB, 100KB
    //     group.throughput(criterion::Throughput::Bytes(*size as u64));
    //
    //     // Benchmark for and_simple
    //     group.bench_with_input(criterion::BenchmarkId::new("and_simple", size), size, |b, &size| {
    //         b.iter(
    //             || {
    //                 let s1 = generate_random_splinter_wrapper(size);
    //                 let s2 = generate_random_splinter_wrapper(size);
    //                 (s1, s2)
    //             }
    //         );
    //     });
    //
    //     // Benchmark for and_chunked
    //     group.bench_with_input(criterion::BenchmarkId::new("and_chunked", size), size, |b, &size| {
    //         b.iter(
    //             || {
    //                 let s1 = generate_random_splinter_wrapper(size);
    //                 let s2 = generate_random_splinter_wrapper(size);
    //                 (s1, s2)
    //             }            
    //         );
    //     });
    // }
    // group.finish();
// }


// // These macros generate the necessary main function for the benchmark.
// criterion_group!(benches, run_splynters_benchmarks);
// criterion_main!(benches);




// fn main() {
//     let bytes1 = Bytes::from(vec![0b11110000, 0b10101010, 0b00001111, 0b11001100, 0b11111111, 0b00000000, 0b11111111, 0b00000000, 0b10101010]);
//     let bytes2 = Bytes::from(vec![0b11001100, 0b11111111, 0b01010101, 0b11001100, 0b11111111, 0b11111111, 0b00000000, 0b11111111]); // Shorter
//
//     println!("Bytes 1:          {:08b}", bytes1.as_ref());
//     println!("Bytes 2:          {:08b}", bytes2.as_ref());
//     println!("------------------------------------------------------------------------------------------");
//
//
//     // // --- Simple AND ---
//     // let and_result_simple = bitwise_and_simple(&bytes1, &bytes2);
//     // // Expected: [11000000, 10101010, 00000101, 11001100, 11111111, 00000000, 00000000, 00000000]
//     // println!("Simple AND Result:  {:08b} (Length: {})", and_result_simple.as_ref(), and_result_simple.len());
//     //
//     // // --- Optimized AND ---
//     // let and_result_optimized = bitwise_and_optimized(&bytes1, &bytes2);
//     // println!("Optimized AND Result:{:08b} (Length: {})", and_result_optimized.as_ref(), and_result_optimized.len());
//     // assert_eq!(and_result_simple, and_result_optimized);
//     //
//     // // --- Simple OR ---
//     // let or_result_simple = bitwise_or_simple(&bytes1, &bytes2);
//     // // Expected (first 8 bytes): [11111100, 11111111, 01011111, 11001100, 11111111, 11111111, 11111111, 11111111, 10101010]
//     // println!("Simple OR Result:   {:08b} (Length: {})", or_result_simple.as_ref(), or_result_simple.len());
//
// }
