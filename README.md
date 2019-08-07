# Single Error Correction, Double Error Detection for Everyone
This crate provides the `Secded` trait, which allows one to add a Hamming Code + Parity based SECDED correction code to any payload.

Encoding and decoding is always done __"In Place"__: the `Secded::code_size()` last bits of the passed buffer should always be 0. Failing to respect this constraint will cause panics. You can disable the checks using the `"no_panic"` feature.

## Implementations
Implementations provided by this crate are listed from fastest to slowest.

### Secded64
This is the fastest implementation provided by this crate, and the one I recommend using unless you need to encode larger than 57 bits payloads.

### Secded128
Almost as fast as Secded64 on x86_64 machines (the slight performance hit being due to the use of 2 cache lines instead of 1 for the encoding/decoding matrix), I haven't tested it on other architectures. Support for u128 is still a bit iffy at the time of writing.

You should use this if your platform has good support for u128 and you need to encode between 58 and 120 bits.

### SecdedDynamic
It can work with any size of encoding, but is much slower than the other 2 implementations (about 10 times slower when working with the same encoding size). It also requires `libstd` to function.  
It is hidden behind the `"dyn"` feature flag.

## FFI
In `secded.h`, you'll find the header for this crate's FFI. Note that the FFI is only built if the `"ffi"` feature is requested, which the provided `CMakeList.txt` does automatically.

## Benchmarks
These benchmarks are only indicative, but feel free to run them yourself using `cargo +nightly bench --features "dyn bench" secded`.  
```
test secded_128::decode          ... bench:          23 ns/iter (+/- 1)
test secded_128::decode_1err     ... bench:          50 ns/iter (+/- 2)
test secded_128::encode          ... bench:          27 ns/iter (+/- 5)
test secded_64::decode           ... bench:          15 ns/iter (+/- 0)
test secded_64::decode_1err      ... bench:          41 ns/iter (+/- 4)
test secded_64::encode           ... bench:          17 ns/iter (+/- 1)
test secded_dynamic::decode      ... bench:         397 ns/iter (+/- 26)
test secded_dynamic::decode_1err ... bench:         502 ns/iter (+/- 30)
test secded_dynamic::encode      ... bench:         411 ns/iter (+/- 27)
```

## How It Works
The correction matrix `C` is built by concatenating the column vector (most significant bit at the top) representations of each encodable integer with a bit count higher than one.

Typically, encoding would use `encoded = data * G`, where `data` is a column vector of `N` bits, and `G` is the `N` sized Identity Matrix on top of `C`.

Instead, this implementation relies on data being `N` bits and `code_size` `0`s, so that the same computation `r = data * H + P` can be used to compute both the correction code at encoding and the syndrome at decoding. `H` is then `[C I 0]` where `I` is the `code_size` sized Identity Matrix, `0` is an appropriately sized null column vector, and `P` is a vector of `0`s, with the last bit set to the parity of `data * H`.

At encoding, the last `code_size` bits of `data` are replaced with `r`.  
At decoding, if an error is detected (non-null, known syndrome), it is corrected in-place, and the last `code_size` bits are reset to `0` to avoid misinterpretations and allow for immediate re-encoding even after mutating the data.