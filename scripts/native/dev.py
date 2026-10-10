"""Cargo and watcher configuration for repository-local native verification."""
import json, os, pathlib, subprocess
ROOT = pathlib.Path(__file__).resolve().parents[2]
ENV = dict(os.environ)
def cargo_tool():
    cargo = subprocess.check_output(['rustup', 'which', 'cargo'], env=ENV, text=True).strip()
    ENV['RUSTC'] = subprocess.check_output(['rustup', 'which', 'rustc'], env=ENV, text=True).strip()
    return cargo

def configuration(cargo, manifest, package, features):
    options = ['--features', features] if features else []
    metadata = json.loads(subprocess.check_output([cargo, 'metadata', '--format-version', '1', '--manifest-path', str(manifest), *options], env=ENV, text=True))
    packages = {pkg['id']: pkg for pkg in metadata['packages']}
    selected = [pkg for pkg in packages.values() if pkg['name'] == package]
    if len(selected) != 1:
        raise ValueError(f'Expected exactly one package named {package!r}')
    pkg = selected[0]
    targets = [target for target in pkg['targets'] if 'cdylib' in target['crate_types']]
    if len(targets) != 1:
        raise ValueError(f'{package} must contain exactly one cdylib target')
    graph = {node['id']: node['dependencies'] for node in metadata['resolve']['nodes']}
    visited = set()
    def walk(identity):
        if identity in visited:
            return
        visited.add(identity)
        for dependency in graph.get(identity, []):
            walk(dependency)
    walk(pkg['id'])
    sources = [str(pathlib.Path(packages[identity]['manifest_path']).parent) for identity in sorted(visited) if packages[identity]['source'] is None]
    workspace = pathlib.Path(metadata['workspace_root'])
    sources += [str(workspace/'Cargo.toml'), str(workspace/'Cargo.lock')]
    target = ROOT/'target'
    artifact = target/'debug'/f"lib{targets[0]['name']}.so"
    return dict(BLOCKUI_DEV_MANIFEST=str(manifest), BLOCKUI_DEV_PACKAGE=package,
                BLOCKUI_DEV_ARTIFACT=str(artifact), BLOCKUI_DEV_TARGET=str(target),
                BLOCKUI_DEV_SOURCES=os.pathsep.join(sources), BLOCKUI_DEV_FEATURES=features)
