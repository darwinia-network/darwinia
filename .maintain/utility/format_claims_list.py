import json

def append_format(origin: str, content: dict) -> str:
	for k, v in content.items():
		origin += '{{ "address": "{}", "backed_ring": {} }},'.format(k, v)

	return origin

with open('genesis_sample.json') as f:
	j = json.load(f)

n_j = '{ "eth": ['
n_j = append_format(n_j, j['eth'])
n_j = n_j[:-1] + '],'

n_j += '"tron": ['
n_j = append_format(n_j, j['tron'])
n_j = n_j[:-1] + '],'

n_j += '"dot": ['
n_j = append_format(n_j, j['dot'])
n_j = n_j[:-1] + ']}'

with open('claims_list.json', 'w+') as f:
	f.write(n_j)

