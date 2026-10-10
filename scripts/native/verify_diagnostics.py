#!/usr/bin/env python3
"""Real compiler checks for authored error positions in ordinary and renamed SDK imports."""
import json, pathlib, subprocess, tempfile
from dev import ROOT, ENV, cargo_tool
CARGO = cargo_tool()

def verify_diagnostics(project, alias):
    path = project / 'src/lib.rs'
    original = path.read_text()
    reports = []
    try:
        for name, parent, child in [
            ('text-type', 'Column::default()', 'Text::new(Column::default())'),
            ('style-type', 'Column::default()', 'Text::new("Test").style("red")'),
            ('child-type', 'Column::default()', 'String::from("Test")'),
            ('unknown-property', 'Column { paddin: 8.0, ..default() }', 'Text::new("Test")'),
        ]:
            source = (f'use {alias}::*;\n#[app]\nstruct Bad;\n#[app_impl]\nimpl Bad {{\n'
                      f'    fn view(&self) -> View {{\n        #[ui]\n        {{\n            {parent};\n'
                      f'            #[ui]\n            {child};\n        }}\n    }}\n}}\n')
            path.write_text(source)
            checked = subprocess.run([CARGO, 'check', '--offline', '--lib', '--message-format=json',
                                      '--manifest-path', str(project / 'Cargo.toml'),
                                      '--target-dir', str(ROOT / 'target')], env=ENV, text=True, capture_output=True)
            assert checked.returncode != 0, checked.stdout
            expected_line = 9 if name == 'unknown-property' else 11
            messages = []
            for line in checked.stdout.splitlines():
                if not line.startswith('{'):
                    continue
                event = json.loads(line)
                if event.get('reason') != 'compiler-message' or event['message']['level'] != 'error':
                    continue
                message = event['message']
                assert 'could not find `blockui`' not in message['message'], message
                for span in message['spans']:
                    if span['is_primary'] and span['file_name'].endswith('src/lib.rs') and span['line_start'] == expected_line:
                        messages.append(message['message'])
            assert messages, checked.stdout + checked.stderr
            reports.append(dict(case=name, authored_line=expected_line, messages=messages))
    finally:
        path.write_text(original)
    recovered = subprocess.run([CARGO, 'check', '--offline', '--lib', '--manifest-path', str(project / 'Cargo.toml'),
                                '--target-dir', str(ROOT / 'target')], env=ENV, text=True, capture_output=True)
    assert recovered.returncode == 0, recovered.stdout + recovered.stderr
    return reports


def main():
    cases = []
    with tempfile.TemporaryDirectory(prefix='wabou-native-diagnostics-') as temporary:
        for alias in ['wabou_native', 'native']:
            project = pathlib.Path(temporary) / alias
            (project / 'src').mkdir(parents=True)
            (project / 'Cargo.toml').write_text('[package]\nname="diagnostics-' + alias.replace('_', '-') + '"\nversion="0.0.0"\nedition="2024"\n[dependencies]\n' + alias + '={package="wabou-native",path=' + json.dumps(str(ROOT / 'crates/wabou-native')) + '}\n')
            (project / 'src/lib.rs').write_text(f'use {alias}::*;\npub fn valid() -> View {{ Text::new("Valid").into_view() }}\n')
            cases.append(dict(alias=alias, diagnostics=verify_diagnostics(project, alias)))
    report = dict(passed=True, cases=cases, fixture_removed=not pathlib.Path(temporary).exists(),
                  scope='Actual authored primary spans for text/style/child/property errors and repaired compiler checks')
    (ROOT / 'reports').mkdir(exist_ok=True)
    (ROOT / 'reports/native-diagnostics.json').write_text(json.dumps(report, indent=2) + '\n')
    print('PASS authored diagnostics and recovery for ordinary/renamed SDK imports')

if __name__ == '__main__':
    main()
