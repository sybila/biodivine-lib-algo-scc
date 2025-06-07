# Biobalm generator for succession diagrams

This folder contains basic scripts for generating succession diagram data using `biobalm`.

### Create virtual environment and install `biobalm`

Best way to use Python libraries is through [virtual environments](https://docs.python.org/3/library/venv.html), since this allows us to use python version and combintation of dependencies that is independent of what is installed by the operating system.

To create a virtual environment:
```
python3 -m venv ./venv
```

Then, activate it in your current terminal session:
```
source ./venv/bin/activate
```

(You have to do this in every terminal window where the virtual environment should be used. This replaces your "default" python installation with the one from the `./venv` folder. Alternatively, you can directly use `./venv/bin/python` or `./venv/bin/pip` to run commands.)

To install biobalm after the environment is activated:
```
pip install biobalm==0.4.1
```

(Newer versions should be also supported, but the originally used version is `0.4.1`)

### Inline constant nodes

For best results, its good to remove constant nodes from the model, as these do not cause "interesting" behavior, only bloat the state space. For this, you can use `inline_constants.py` (similar function is available in Rust roo). The output is an `.aeon` file.

```
python3 inline_constants.py ./path/to/network
```

To perform inlining on all `.aeon` files in a folder, you can use `./inline_all.sh /path/to/folder`. However, note that this is destructive and will overwrite the models in the original folder.

Also note that if the network is fully determined by its inputs (i.e. every variable becomes fixed due to the input values), then the output network is empty. We can ignore such networks, as their behavior is "trivial".

### Generated succession diagram data

In `models-randomized-inputs.zip`, we give a wide range of models across input valuations.