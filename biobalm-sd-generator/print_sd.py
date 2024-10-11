from biobalm import SuccessionDiagram
from biodivine_aeon import BooleanNetwork
import sys
import json

bn = BooleanNetwork.from_file(sys.argv[1])
bn = bn.infer_valid_graph()
bn_inlined = bn.inline_constants(infer_constants=True, repair_graph=True)

if bn != bn_inlined:
	print("The network contains unpercolated constants.")
	print("The generated succession diagram may contain some redundancy.")
	print("It's better to first percolate (inline) constants.")
	sys.exit(1)

sd = SuccessionDiagram(bn)
assert sd.expand_bfs()

result_list = []
for n_id in sd.node_ids():
	data = sd.node_data(n_id)
	all_motifs = []
	for s_id in sd.node_successors(n_id):
		stable_motifs = sd.edge_all_stable_motifs(n_id, s_id)
		all_motifs += stable_motifs
	result_list.append({
		'trap': data['space'],
		'motifs': all_motifs
	})
print(json.dumps(result_list))	