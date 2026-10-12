#!/usr/bin/env python3
"""Actual rust-analyzer queries for independent ordinary and renamed SDK imports."""
import json
import pathlib
import subprocess
import sys
import tempfile
from dev import ROOT, ENV


def main():
    results = []
    with tempfile.TemporaryDirectory(prefix='wabou-native-editor-') as temporary:
        for name, alias in [('facade', 'wabou_native'), ('renamed-facade', 'native')]:
            project = pathlib.Path(temporary) / name
            (project / 'src').mkdir(parents=True)
            dependency = f'{alias} = {{ package = "wabou-native", path = {json.dumps(str(ROOT / "native/wabou-native"))}, features = ["testing"] }}\n'
            (project / 'Cargo.toml').write_text(f'[package]\nname="editor-{name}"\nversion="0.0.0"\nedition="2024"\n[dependencies]\n' + dependency)
            source = (ROOT / 'native/wabou-native/tests/tag_ui_guide.rs').read_text().replace('wabou_native::', f'{alias}::')
            (project / 'src/lib.rs').write_text(source)
            subprocess.run([sys.executable, str(ROOT / 'scripts/native/verify_macro_ui_lsp.py'),
                            '--project', str(project)], cwd=ROOT, env=ENV, check=True)
            report = json.loads((ROOT / 'reports/external-tag-ui-lsp.json').read_text())
            results.append(dict(case=name, queries=report))
    report = dict(passed=True, cases=results, fixture_removed=not pathlib.Path(temporary).exists(),
                  scope='Actual Rust analyzer tag completion/navigation against independent facade and renamed facade consumers')
    (ROOT / 'reports/native-editor.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS independent native SDK editor consumers')


if __name__ == '__main__':
    main()
