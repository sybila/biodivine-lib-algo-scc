# cejn

A very much WIP implementation of the [chain](https://link.springer.com/chapter/10.1007/978-3-031-30820-8_22) algorithm

This one is the attempt for implementation in Rust specifically

### Testing

You probably want to test with `--release` enabled, as many of the tests perform non-trivial
symbolic computations.

By default, the tests only consider very small networks (~10 variables).
If feature `expensive-tests` is enabled, networks up to 15 variables are consider.
However, at the moment, this can require larger than default stack size.

Finally, if console output is enabled, it is recommended to run the tests single 
threaded otherwise the output may be scrambled.

To run basic tests without output, execute the command below. This should be completed 
fairly fast (seconds to minutes).
```
cargo test --release
```

To run extended tests with output enabled, execute the command below. This should take 
a substantial time (minutes to hours).
```
RUST_MIN_STACK=60000; cargo test --release --all-features -- --nocapture 
```

### Benchmarking

Each algorithm can be benchmarked by an executable in `examples`. Each executable takes
a model path as input, and outputs the number of discovered SCCs, followed by the sizes of 
the non-trivial SCCs. You are expected to use `time` to measure the runtime of the executable:

```
time cargo run --release --examples chain -- ./path/to/model.aeon
```

To run the benchmark for a collection of models, you can use the `bench.py` script.
This script takes a timeout (applied through the unix `timeout` utility), a path to a
folder with model files, and a path to an executable or a python script. It then applies
a per-benchmark timeout, executes the target program for each model file, and measures the 
execution time. The results are aggregated into a CSV table (this includes the runtime
and the last line of output printed by the executable). Furthermore, the
raw output of each executable is saved into a separate text file. The name of the output
directory contains the name of the model collection, the name of the benchmarked program,
and a timestamp. This ensures traceability of results. 

We recommend using `ulimit`
to restrict the process to a fixed memory limit for repeatability and stability. For additional
comments, see also `bench_all.sh`.

```
# Make sure to first run `cargo --build --release` to generate up-to-date executables. 
python3 ./bench.py 1h ./models/bbm-inputs-true ./target/release/examples/chain
```

 > Known limitations: 
 >  * Sometimes, not all output is written into the benchmark output file if the benchmark is
 >    killed either due to timeout or due to out-of-memory error.
 >  * Sometimes, an error code is not propagated correctly and a benchmark will be marked as
 >    successful even though it failed. In such case, the last line of output is still printed
 >    into the summary table, and you should manually check that this last line is reasonable.
 >  * If you interrupt the "management" Python process (e.g. using Ctrl+C), the last benchmark 
 >    is not always killed properly. Make sure you don't have such "zombie" processes running.