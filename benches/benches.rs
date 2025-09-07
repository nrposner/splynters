
use std::hint::black_box;
use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion};


use splynters::*;






fn run_splynters_benchmarks(c: &mut Criterion) {
    c.bench_function("find_5_distributions_with_setup", |b| {
        b.iter(|| {





            // let params = set_parameters(4.77, 3.2031, 100, 0, 10, None, None, 1, None, RestrictionsOption::Default, false).unwrap();
            //
            // let mut rng = StdRng::seed_from_u64(42);
            //
            // find_possible_distributions(
            //     black_box(&params), 
            //     black_box(5), 
            //     black_box(false), 
            //     &mut rng
            // )


        })
    });
}


// These macros generate the necessary main function for the benchmark.
criterion_group!(benches, run_splynters_benchmarks);
criterion_main!(benches);




// fn main() {
//     let bytes1 = Bytes::from(vec![0b11110000, 0b10101010, 0b00001111, 0b11001100, 0b11111111, 0b00000000, 0b11111111, 0b00000000, 0b10101010]);
//     let bytes2 = Bytes::from(vec![0b11001100, 0b11111111, 0b01010101, 0b11001100, 0b11111111, 0b11111111, 0b00000000, 0b11111111]); // Shorter
//
//     println!("Bytes 1:          {:08b}", bytes1.as_ref());
//     println!("Bytes 2:          {:08b}", bytes2.as_ref());
//     println!("------------------------------------------------------------------------------------------");
//
//
//
//
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
