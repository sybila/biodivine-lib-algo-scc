from biodivine_aeon import BooleanNetwork
import sys

# A simple script to produce a BN with inlined constant nodes.

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.inline_constants(infer_constants=True, repair_graph=True)

print(bn.to_aeon())