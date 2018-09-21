//! This library provides a time-stamp counter (TSC) based timer for micro
//! benchmarking.
//!
//! # Example
//!
//! ```
//! # #![feature(test)]
//! # extern crate test;
//! # use self::test::black_box;
//! # use tsc_timer::*;
//! // The function we want to time:
//! pub fn fibonacci(n: u64) -> u64 {
//!     match n {
//!         0 | 1 => 1,
//!         n => fibonacci(n - 1) + fibonacci(n - 2),
//!     }
//! }
//!
//! # if has_invariant_tsc() {
//! // Non-invariant TSCs might produce unreliable results:
//! assert!(has_invariant_tsc(), "The TSC is not invariant!");
//! # }
//!
//! let (duration, result) =
//!     Duration::span(|| black_box(fibonacci(black_box(8))));
//!
//! assert_eq!(result, 34);
//!
//! println!("Reference cycle count: {} cycles.", duration.cycles());
//! // On my machine prints:
//! // "Reference cycle count: 951 cycles."
//! ```
//!
//! # Notes
//!
//! * The TSC runs at a different frequency than the CPU clock frequency, so
//! the cycles reported here are "reference cycles" and not real CPU clock
//! cycles.
//!
//! * If the TSC is not _invariant_ (Nehalem-and-later) the measurements might
//! not be very accurate due to turbo boost, speed-step, power management, etc.
//!
//! * Converting "reference cycles" to time (e.g., nanoseconds) is, in general,
//! not possible to do reliably in user-space.
//!
//! * One might want to disable preemption and hard interrupts before timing
//! (see [How to Benchmark Code Execution Times on Intel® IA-32 and IA-64
//! Instruction Set Architectures][intel_bench_paper]) to further improve the
//! accuracy of the measurements.
//!
//! # References
//!
//! * [How to Benchmark Code Execution Times on Intel® IA-32 and IA-64
//! Instruction Set Architectures][intel_bench_paper]
//! * [Pitfalls of TSC usage][pitfalls_tsc]
//! * [Time Stamps Counters][tinola_blog]
//! * [SO answer to "Get CPU cycle count"][so_cpu_cycles]
//! * [So answer to "Using Time stamp counter to get the time
//! stamp"][so_time_stamp]
//!
//! [intel_bench_paper]:
//! https://www.intel.com/content/dam/www/public/us/en/documents/white-papers/ia-32-ia-64-benchmark-code-execution-paper.pdf
//! [pitfalls_tsc]: http://oliveryang.net/2015/09/pitfalls-of-TSC-usage/
//! [tinola_blog]: http://blog.tinola.com/?e=54
//! [so_cpu_cycles]: https://stackoverflow.com/a/51907627/1422197
//! [so_time_stamp]: https://stackoverflow.com/a/42490374/1422197
#![feature(asm, stdsimd, test)]
#![cfg_attr(not(test), no_std)]

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
compile_error!(
    "The TSC crate only supports the \"x86\" and \"x86_64\" architectures"
);

extern crate test;

use core::ops;

#[cfg(target_arch = "x86")]
use core::arch::x86 as arch;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as arch;

/// Returns true if the CPU has an invariant TSC.
///
/// Without an invariant TSC, the timings reported by this library might be
/// unreliable.
pub fn has_invariant_tsc() -> bool {
    use self::arch::{has_cpuid, CpuidResult, __cpuid};

    // The invariant TSC is advertised in the CPUID.80000007H:EDX[8] bit.

    // CPU doesn't have a CPUID instruction => too old to have an invariant
    // TSC.
    if !has_cpuid() {
        return false;
    }

    // Obtain the largest basic CPUID leaf supported by the CPUID
    let CpuidResult { eax: max_basic_leaf, .. } = unsafe { __cpuid(0_u32) };

    // Earlier Intel 486 => too old to have an invariant TSC.
    if max_basic_leaf < 1 {
        return false;
    }

    // Obtain the largest extended CPUID leaf supported by the CPUID
    let CpuidResult { eax: max_extended_leaf, .. } =
        unsafe { __cpuid(0x8000_0000_u32) };

    // CPU doesn't have "Advanced Power Management Information" => too old to
    // have an invariant TSC.
    if max_extended_leaf < 7 {
        return false;
    }

    let CpuidResult { edx, .. } = unsafe { __cpuid(0x8000_0007_u32) };

    // Test CPUID.80000007H:EDX[8], if the bit is set, the CPU has an
    // invariant TSC
    edx & (1 << 8) != 0
}

/// Start time instant
pub struct Start(u64);

impl Start {
    /// Start measurement
    pub fn now() -> Self {
        unsafe {
            let _ = arch::__cpuid(0);
            Start(core::mem::transmute(arch::_rdtsc()))
        }
    }
}

/// Stop time instant
pub struct Stop(u64);

impl Stop {
    /// Stop measurement
    pub fn now() -> Self {
        unsafe {
            let mut core: u32 = 0;
            let r = arch::__rdtscp(&mut core as *mut _) as u64;
            let _ = arch::__cpuid(0);
            Stop(r)
        }
    }
}

/// Duration between two time instants
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Duration(u64);

impl Duration {
    /// Number of clock cycles
    pub fn cycles(self) -> u64 {
        self.0
    }

    /// Returns a tuple of the execution duration of the function `f` and its
    /// result.
    pub fn span<R, F: Fn() -> R>(f: F) -> (Self, R) {
        let start = Start::now();
        let result = f();
        let stop = Stop::now();
        let measurement_duration = stop - start;
        let measurement_overhead = Self::span_overhead();
        assert!(measurement_overhead <= measurement_duration);
        let f_duration = measurement_duration - measurement_overhead;
        (f_duration, result)
    }

    /// Returns the overhead that is intrinsic to [`span`].
    ///
    /// That is, how many cycles does it take to time a no-op.
    fn span_overhead() -> Self {
        let start = Start::now();
        let _result = test::black_box(0);
        let stop = Stop::now();
        stop - start
    }
}

impl ops::Sub<Start> for Stop {
    type Output = Duration;
    fn sub(self, start: Start) -> Duration {
        debug_assert!(
            self.0 > start.0,
            "stop time instant happened after start time instant"
        );
        Duration(self.0 - start.0)
    }
}

impl ops::Sub<Duration> for Duration {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        debug_assert!(self.0 > other.0, "subtracting durations overflows");
        Duration(self.0 - other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn start_stop() {
        let start = Start::now();
        let _noop = test::black_box(0);
        let stop = Stop::now();
        let dur1 = stop - start;

        fn foo() -> i32 {
            test::black_box(test::black_box(2_i32).pow(test::black_box(29)))
        }

        let (dur2, r) = Duration::span(foo);
        assert_eq!(r, 536_870_912);

        if has_invariant_tsc() {
            println!(
                "dur2: {} cycles, dur1: {} cycles",
                dur2.cycles(),
                dur1.cycles()
            );
        }
    }

    #[test]
    fn print_span_overhead() {
        println!(
            "span overhead: {} cycles",
            Duration::span_overhead().cycles()
        );
    }

    #[test]
    fn invariant_tsc() {
        if !has_invariant_tsc() {
            println!("the cpu does not have an invariant TSC");
        }
    }
}
