from json import load
from os.path import abspath, dirname


def append_format(origin: str, content: dict) -> str:
    for k, v in content.items():
        origin += '{{ "address": "{}", "backed_ring": {} }},'.format(k, v)

    return origin


root_dir = dirname(dirname(dirname(abspath(__file__))))

with open(''.join([root_dir, '/.maintain', '/utility', '/crab-claims', '/crab-claims-list.json'])) as f:
    j = load(f)

n_j = '{"dot":['
n_j = append_format(n_j, j['dot'])
n_j = n_j[:-1] + '],'

n_j += '"eth":['
n_j = append_format(n_j, j['eth'])
n_j = n_j[:-1] + '],'

n_j += '"tron":['
n_j = append_format(n_j, j['tron'])
n_j = n_j[:-1] + ']}'

with open(''.join([root_dir, '/node', '/service', '/res', '/crab', '/claims-list.json']), 'w') as f:
    f.write(n_j)
