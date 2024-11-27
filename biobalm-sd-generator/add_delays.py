from biodivine_aeon import *
import sys

bn = BooleanNetwork.from_file(sys.argv[1])

in_vars = []
out_vars = []
for var in bn.variable_names():
	in_vars.append(f"{var}_in")
	out_vars.append(f"{var}_out")

all_vars = []
for (i,o) in zip(in_vars, out_vars):
	all_vars.append(i)
	all_vars.append(o)


new_bn = BooleanNetwork(all_vars)

# Add "value propagation" functions.
for i in range(bn.variable_count()):
	new_bn.ensure_regulation({
		'source': in_vars[i],
		'target': out_vars[i],
		'sign': '+',
		'essential': True
	})
	new_fn = UpdateFunction.mk_var(new_bn, in_vars[i])
	new_bn.set_update_function(out_vars[i], new_fn)

# Copy existing regulations:
old_names = bn.variable_names()
old_to_in = { old_names[i]: in_vars[i] for i in range(bn.variable_count()) }
old_to_out = { old_names[i]: out_vars[i] for i in range(bn.variable_count()) }

for reg in bn.regulations():
	new_reg = {
		'source': old_to_out[bn.get_variable_name(reg['source'])],
		'target': old_to_in[bn.get_variable_name(reg['target'])],
		'sign': reg['sign'],
		'essential': reg['essential']
	}
	new_bn.ensure_regulation(new_reg)

# Copy existing functions
for old_name in bn.variable_names():
	old_fn = bn.get_update_function(old_name)
	new_fn = old_fn.rename_all(new_bn, old_to_out)
	new_bn.set_update_function(old_to_in[old_name], new_fn)

print(new_bn.to_aeon())

