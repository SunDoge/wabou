#!/usr/bin/env python3
"""Build a linked release UI and run its executable alone in an isolated display."""
import argparse
import json
import pathlib
import shutil
import subprocess
import tempfile
from display import isolated_display
from dev import ROOT, ENV, cargo_tool
CARGO = cargo_tool()
from storage import disk


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cli', action='store_true', help='build the generated facade app through the static CLI path')
    parser.add_argument("--wabou-cli", type=pathlib.Path, help="verify a built Wabou CLI native command adapter")
    args = parser.parse_args()
    args.cli = args.cli or args.wabou_cli is not None
    example = (ROOT / 'apps/native-counter/src/lib.rs').read_text() + '\nuse wabou_native::native;\n'
    entry = '''
fn main() -> std::result::Result<(), String> {
    let options = native::WindowOptions::new().title("Linked native UI");
    native::run_capture::<Counter>(options)
}
'''
    with tempfile.TemporaryDirectory(prefix='wabou-native-static-') as temporary:
        project = pathlib.Path(temporary) / 'project'
        (project / 'src').mkdir(parents=True)
        (project / 'Cargo.toml').write_text(
            '[package]\nname="static-native-proof"\nversion="0.0.0"\nedition="2024"\n'
            '[dependencies]\nwabou-native={path=' + json.dumps(str(ROOT / 'crates/wabou-native')) + ',features=["native"]}\n'
            '[profile.release]\ndebug=0\nstrip="debuginfo"\n')
        (project / 'src/main.rs').write_text(example + entry)
        binary_name = 'static-native-proof'
        if args.cli:
            cli = args.wabou_cli.resolve() if args.wabou_cli else ROOT / 'target/debug/blockui'
            if not args.wabou_cli:
                subprocess.run([CARGO, 'build', '-p', 'blockui-cli'], env=ENV, check=True)
            command = [str(cli), 'native'] if args.wabou_cli else [str(cli)]
            generated = pathlib.Path(temporary) / 'linked-counter'
            created = subprocess.run([*command, 'new', str(generated), '--wabou-native'], env=ENV, text=True, capture_output=True)
            assert created.returncode == 0, created.stdout + created.stderr
            main_path = generated / 'src/main.rs'
            original_main = main_path.read_text()
            main_path.write_text(original_main + '\nfn invalid_entry() { let _: u8 = "wrong type"; }\n')
            library = subprocess.run([*command, 'check', str(generated), '--target-dir', str(ROOT / 'target')],
                                     env=ENV, text=True, capture_output=True)
            assert library.returncode == 0, library.stdout + library.stderr
            static_check = subprocess.run([*command, 'check', str(generated), '--static', '--message-format', 'json',
                                           '--target-dir', str(ROOT / 'target')], env=ENV, text=True, capture_output=True)
            assert static_check.returncode != 0, static_check.stdout + static_check.stderr
            errors = [json.loads(line)['message'] for line in static_check.stdout.splitlines()
                      if line.startswith('{') and json.loads(line).get('reason') == 'compiler-message'
                      and json.loads(line)['message']['level'] == 'error']
            assert any(error.get('code', {}).get('code') == 'E0308' and
                       any(span['is_primary'] and span['file_name'].endswith('src/main.rs') for span in error['spans'])
                       for error in errors), static_check.stdout + static_check.stderr
            main_path.write_text(original_main)
            recovered = subprocess.run([*command, 'check', str(generated), '--static', '--message-format', 'short',
                                        '--target-dir', str(ROOT / 'target')], env=ENV, text=True, capture_output=True)
            assert recovered.returncode == 0, recovered.stdout + recovered.stderr
            doctor = subprocess.run([*command, 'doctor', str(generated), '--static'], env=ENV, text=True, capture_output=True)
            assert doctor.returncode == 0 and 'Native binary: linked-counter' in doctor.stdout, doctor.stdout + doctor.stderr
            binary_only = pathlib.Path(temporary) / 'binary-only'
            (binary_only / 'src').mkdir(parents=True)
            (binary_only / 'Cargo.toml').write_text('[package]\nname="binary-only"\nversion="0.0.0"\nedition="2024"\n')
            (binary_only / 'src/main.rs').write_text('fn main() {}\n')
            doctor = subprocess.run([*command, 'doctor', str(binary_only), '--static'], env=ENV, text=True, capture_output=True)
            assert doctor.returncode == 0 and 'Native binary: binary-only' in doctor.stdout, doctor.stdout + doctor.stderr
            assert not (binary_only / 'target').exists(), 'doctor must not compile a binary'
            dynamic_doctor = subprocess.run([*command, 'doctor', str(binary_only)], env=ENV, text=True, capture_output=True)
            assert dynamic_doctor.returncode != 0 and 'cdylib target' in dynamic_doctor.stderr, dynamic_doctor.stdout + dynamic_doctor.stderr
            denied = subprocess.run([*command, 'dev', str(generated), '--static'], env=ENV, text=True, capture_output=True)
            assert denied.returncode != 0 and 'does not use dylib hot reload' in denied.stderr, denied.stderr
            denied = subprocess.run([*command, 'build', str(generated), '--static', '--web'], env=ENV, text=True, capture_output=True)
            assert denied.returncode != 0, denied.stderr
            manifest_path = generated / 'Cargo.toml'
            original_manifest = manifest_path.read_text()
            manifest_path.write_text(original_manifest.replace('[package.metadata.blockui]', '[package.metadata.blockui]\nstatic-bin = "missing-bin"'))
            denied = subprocess.run([*command, 'build', str(generated), '--static'], env=ENV, text=True, capture_output=True)
            assert denied.returncode != 0 and 'static-bin target' in denied.stderr, denied.stderr
            manifest_path.write_text(original_manifest)
            built = subprocess.run([*command, 'build', str(generated), '--static', '--release', '--target-dir', str(ROOT / 'target')],
                                   env=ENV, text=True, capture_output=True)
            assert built.returncode == 0, built.stdout + built.stderr
            binary_name = 'linked-counter'
            assert built.stdout.strip().splitlines()[-1].endswith(binary_name + ('.exe' if __import__('os').name == 'nt' else ''))
            with isolated_display(ENV) as environment:
                run = subprocess.run([*command, 'run', str(generated), '--static', '--release', '--capture', '--target-dir', str(ROOT / 'target')],
                                     env=environment, text=True, capture_output=True, timeout=120)
            assert run.returncode == 0, run.stdout + run.stderr
            assert not (generated / '.runtime').exists()
        else:
            built = subprocess.run([CARGO, 'build', '--offline', '--release', '--bin', 'static-native-proof',
                                    '--manifest-path', str(project / 'Cargo.toml'), '--target-dir', str(ROOT / 'target')],
                                   env=ENV, text=True, capture_output=True)
            assert built.returncode == 0, built.stdout + built.stderr
        deployment = pathlib.Path(temporary) / 'deployment'
        deployment.mkdir()
        executable = deployment / (binary_name + ('.exe' if __import__('os').name == 'nt' else ''))
        shutil.copy2(ROOT / 'target/release' / executable.name, executable)
        assert list(deployment.iterdir()) == [executable]
        with isolated_display(ENV) as environment:
            environment['BLOCKUI_CARGO'] = str(deployment / 'unavailable-cargo')
            captured = subprocess.run([str(executable), "--capture"], cwd=deployment, env=environment,
                                      text=True, capture_output=True, timeout=120)
        assert captured.returncode == 0, captured.stdout + captured.stderr
        assert 'PASS actual Winit window + Vello capture' in captured.stderr, captured.stderr
        assert not (deployment / '.runtime').exists()
        assert not list(deployment.rglob('*.so')) and not list(deployment.rglob('*.dll')) and not list(deployment.rglob('*.dylib'))
        shutil.copy2(deployment / 'reports/native.png', ROOT / ('reports/wabou-integrated-static.png' if args.wabou_cli else 'reports/wabou-native-static.png'))
        report = dict(passed=True, release=True, wabou_cli=str(args.wabou_cli) if args.wabou_cli else None, cli_static_path=args.cli, executable_allocated_bytes=executable.stat().st_blocks * 512,
                      fixture_disk=disk(pathlib.Path(temporary)), isolated_native_capture=captured.stderr,
                      requires_ui_dylib=False, creates_runtime_generations=False,
                      scope='Single-root linked UI executable; does not prove loader feature stripping or multi-actor families')
    report['fixture_removed'] = not pathlib.Path(temporary).exists()
    (ROOT / ('reports/wabou-integrated-static.json' if args.wabou_cli else 'reports/wabou-native-static-cli.json' if args.cli else 'reports/wabou-native-static.json')).write_text(json.dumps(report, indent=2) + '\n')
    print('PASS linked release UI launches with executable alone and no Cargo or plugin path')


if __name__ == '__main__':
    main()
