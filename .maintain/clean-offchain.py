from glob import iglob
from os.path import abspath, dirname
from pathlib import Path
from shutil import rmtree

for pattern in ['*eth_offchain*']:
    for p in iglob(''.join([dirname(dirname(abspath(__file__))), '/target', '/release', '/**/', pattern]), recursive=True):
        print('removed:', p)
        p = Path(p)
        if p.is_dir():
            rmtree(p)
        elif p.is_file:
            p.unlink()
