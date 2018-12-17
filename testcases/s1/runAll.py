import os
import subprocess
import sys


def read_txt_file(filename):
    with open(filename, 'r') as f:
        txt = f.read().strip()
    # Python should be able to do it automatically, but just in case...
    txt = txt.replace('\r', '')
    return txt


def main():
    decafc = os.path.join('..', '..', 'target', 'release', 'decaf')
    names = sys.argv[1:]
    if not names:
        names = sorted(os.listdir('.'))
    for name in names:
        bname, ext = os.path.splitext(name)
        if ext != '.decaf':
            continue
        # Run the test case, redirecting stdout/stderr to output/bname.result
        subprocess.call([decafc, '-l', name],
                        stdout=open(os.path.join('output', bname + '.result'), 'w'),
                        stderr=subprocess.STDOUT)
        # Check the result
        expected = read_txt_file(os.path.join('result', bname + '.result'))
        actual = read_txt_file(os.path.join('output', bname + '.result'))
        if expected == actual:
            info = 'OK :)'
        else:
            info = 'ERROR!'
        print('{0:<30}{1}'.format(name, info))


if __name__ == '__main__':
    main()
