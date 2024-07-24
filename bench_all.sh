#!/bin/bash

# Ensure the binaries are fresh.
cargo build --release

# Limit memory consumption to 8GB.
# WARNING: This won't work on macOS.
ulimit -v 8388608

# bench.py [timeout] [model_directory] [executable or script.py]
# Optional, but mutually exclusive: -i (interactive mode), -p X (parallel mode).

# Models in bbm-inputs-true are (in theory) the "easier" variants, since the input variables
# have fixed values.

# For actually meaningful final results, it is recommended to increase the timeout, but high
# timeout means very long runtime if a lot of models fail, so we probably don't want to increase
# this too much until the algorithm performs reasonably well.

python3 ./bench.py 1m ./models/bbm-inputs-true ./target/release/fwd_bwd
python3 ./bench.py 1m ./models/bbm-inputs-true ./target/release/chain
python3 ./bench.py 1m ./models/bbm-inputs-true ./target/release/chain_saturation
