"""Owned allocated storage accounting without following symbolic links."""
import stat


def validate_budget(budget_bytes):
    """Deletion budgets are nonnegative integer byte counts, never NaN or bool."""
    if type(budget_bytes) is not int or budget_bytes < 0:
        raise ValueError('Cache budget must be a nonnegative integer byte count')


def category(relative):
    parts = relative.parts
    # Cargo --target nests the profile under the target triple. Keep host and
    # target artifacts in the same additive categories, with global inode
    # deduplication still performed by disk(). Do not search arbitrary descendants
    # for a profile name: build-script output may itself contain debug/deps.
    profile_index = 0
    if len(parts) >= 2 and parts[0] not in ('debug', 'release'):
        profile_index = 1
    if len(parts) > profile_index + 1 and parts[profile_index] in ('debug', 'release'):
        return {'deps': 'dependencies', 'incremental': 'incremental',
                'build': 'build_scripts', 'examples': 'examples',
                '.fingerprint': 'fingerprints'}.get(parts[profile_index + 1], 'artifacts')
    return 'other'


def disk(path, breakdown=False):
    if path.is_symlink():
        raise ValueError(f'Refusing to report symlinked root as owned storage: {path}')
    seen = set()
    logical = allocated = files = 0
    groups = {}
    if path.exists():
        entries = path.rglob('*')
        if breakdown:
            # Attribute links to dependencies before their copied top-level name.
            # Counting every category separately would inflate the combined total.
            priority = {'dependencies': 0, 'examples': 1, 'artifacts': 2,
                        'incremental': 3, 'build_scripts': 4, 'fingerprints': 5, 'other': 6}
            entries = sorted(entries, key=lambda p: (priority[category(p.relative_to(path))], str(p)))
        for p in entries:
            try:
                st = p.lstat()
            except FileNotFoundError:
                # A build or owned-cache trim may remove an enumerated entry.
                continue
            if not stat.S_ISREG(st.st_mode):
                continue
            ident = (st.st_dev, st.st_ino)
            if ident in seen:
                continue
            seen.add(ident)
            logical += st.st_size
            allocated += st.st_blocks * 512
            files += 1
            if breakdown:
                group = groups.setdefault(category(p.relative_to(path)),
                                          dict(logical_bytes=0, allocated_bytes=0, files=0))
                group['logical_bytes'] += st.st_size
                group['allocated_bytes'] += st.st_blocks * 512
                group['files'] += 1
                if category(p.relative_to(path)) == 'dependencies':
                    kind = {'.rlib': 'rust_libraries', '.rmeta': 'rust_metadata',
                            '.so': 'shared_libraries', '.dylib': 'shared_libraries',
                            '.dll': 'shared_libraries', '.o': 'object_files',
                            '.d': 'dependency_info'}.get(p.suffix)
                    if kind is None:
                        kind = 'executable_files' if st.st_mode & 0o111 else 'other'
                    detail = group.setdefault('kinds', {}).setdefault(kind,
                        dict(logical_bytes=0, allocated_bytes=0, files=0))
                    detail['logical_bytes'] += st.st_size
                    detail['allocated_bytes'] += st.st_blocks * 512
                    detail['files'] += 1

    result = dict(logical_bytes=logical, allocated_bytes=allocated, files=files)
    if breakdown:
        result['categories'] = groups
    return result
