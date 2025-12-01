import os
import re
import sys

def list_source_files(root):
    exts = {'.chim', '.nova'}
    files = []
    for dirpath, dirnames, filenames in os.walk(root):
        for f in filenames:
            _, ext = os.path.splitext(f)
            if ext in exts:
                files.append(os.path.join(dirpath, f))
    return files

def normalize_default_branch(text):
    pattern = re.compile(r'^(\s*)默认\s*->', re.MULTILINE)
    return pattern.sub(r'\1_ ->', text)

def process_file(path, fix):
    with open(path, 'r', encoding='utf-8') as fh:
        content = fh.read()
    normalized = normalize_default_branch(content)
    changed = normalized != content
    if fix and changed:
        with open(path, 'w', encoding='utf-8') as fh:
            fh.write(normalized)
    return changed

def main():
    args = sys.argv[1:]
    check = '--check' in args
    fix = '--fix' in args or not check
    root = '.'
    files = list_source_files(root)
    changed = []
    for f in files:
        if process_file(f, fix):
            changed.append(f)
    if check:
        if changed:
            for f in changed:
                print(f)
            sys.exit(1)
        sys.exit(0)

if __name__ == '__main__':
    main()
