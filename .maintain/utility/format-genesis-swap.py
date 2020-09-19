from json import load
from os.path import abspath, dirname
from sys import argv

root_dir = dirname(dirname(dirname(abspath(__file__))))

with open(''.join([root_dir, '/.maintain', '/utility', '/', argv[1]])) as r:
    with open(''.join([root_dir, '/node', '/service', '/res', '/', argv[1]]), 'w') as w:
        w.write(''.join([
            '[',
            ','.join(['{{"address":"0x{}","balance":{}}}'.format(k, int(v))
                      for k, v in load(r).items()]),
            ']'
        ]))
