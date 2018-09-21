# `x86`/`x86_64` Time-stamp-counter (TSC) timer

[![Travis-CI Status]][travis] [![Appveyor Status]][appveyor] [![Latest Version]][crates.io] [![docs]][master_docs]

[API docs of the `tsc` crate `master` branch][master_docs]

This library provides a time-stamp counter (TSC) based timer for micro
benchmarking:

```rust
// The function we want to time:
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// Non-invariant TSCs might produce unreliable results:
assert!(has_invariant_tsc(), "The TSC is not invariant!");

let (duration, result) = Duration::span(|| black_box(fibonacci(black_box(8))));

assert_eq!(result, 34);

println!("Reference cycle count: {} cycles.", duration.cycles());
//! // On my machine prints: "Reference cycle count: 951 cycles."
```

## Notes

* The TSC runs at a different frequency than the CPU clock frequency, so the
  cycles reported here are "reference cycles" and not real CPU clock cycles.

* If the TSC is not _invariant_ (Nehalem-and-later) the measurements might
  not be very accurate due to turbo boost, speed-step, power management, etc.
* Converting "reference cycles" to time (e.g., nanoseconds) is, in general,
  not possible to do reliably in user-space.

* One might want to disable preemption and hard interrupts before timing
  (see [How to Benchmark Code Execution Times on Intel® IA-32 and IA-64
  Instruction Set Architectures][intel_bench_paper]) to further improve the
  accuracy of the measurements.
# References

  * [How to Benchmark Code Execution Times on Intel® IA-32 and IA-64
  Instruction Set Architectures][intel_bench_paper]
  * [Pitfalls of TSC usage][pitfalls_tsc]
  * [Time Stamps Counters][tinola_blog]
  * [SO answer to "Get CPU cycle count"][so_cpu_cycles]
  * [So answer to "Using Time stamp counter to get the time stamp"][so_time_stamp]

## License

This project is licensed under either of

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
  ([LICENSE-APACHE](LICENSE-APACHE))

* [MIT License](http://opensource.org/licenses/MIT)
  ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

We welcome all people who want to contribute.

Contributions in any form (issues, pull requests, etc.) to this project
must adhere to Rust's [Code of Conduct].

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `tsc` by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[travis]: https://travis-ci.org/gnzlbg/tsc
[Travis-CI Status]: https://travis-ci.org/gnzlbg/tsc.svg?branch=master
[appveyor]: https://ci.appveyor.com/project/gnzlbg/tsc
[Appveyor Status]: https://ci.appveyor.com/api/projects/status/d9gs34kvj6j3k96g?svg=true
[Latest Version]: https://img.shields.io/crates/v/tsc.svg
[crates.io]: https://crates.io/crates/tsc
[docs]: https://docs.rs/tsc/badge.svg
[docs.rs]: https://docs.rs/tsc/
[master_docs]: https://gnzlbg.github.io/tsc/tsc/
[Code of Conduct]: https://www.rust-lang.org/en-US/conduct.html
[intel_bench_paper]:
https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/ia-32-ia-64-benchmark-code-execution-paper.pdf
[pitfalls_tsc]: http://oliveryang.net/2015/09/pitfalls-of-TSC-usage/
[tinola_blog]: http://blog.tinola.com/?e=54
[so_cpu_cycles]: https://stackoverflow.com/a/51907627/1422197
[so_time_stamp]: https://stackoverflow.com/a/42490374/1422197
