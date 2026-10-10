#!/usr/bin/env python3
"""Repository-local Rust frontend checks; rendering and reload checks are opt-in."""
import argparse
from datetime import datetime, timezone
import json
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reload', action='store_true', help='also test real dylib replacements and async completion')
    parser.add_argument('--release', action='store_true', help='also capture a linked release in an isolated Xvfb display')
    parser.add_argument("--native-state", action="store_true", help="also test native widget state and stable-host service dispatch without a window")
    parser.add_argument("--pixels", action="store_true", help="also validate selected native Vello pixels in isolated Xvfb")
    parser.add_argument("--editor", action="store_true", help="also query rust-analyzer for independent ordinary and renamed SDK consumers")
    parser.add_argument("--diagnostics", action="store_true", help="also check authored compiler error locations and recovery")
    parser.add_argument("--full", action="store_true", help="run all native checks, including strict SDK/CLI lint")
    args = parser.parse_args()
    if args.full:
        for option in ["native_state", "reload", "pixels", "editor", "diagnostics", "release"]:
            setattr(args, option, True)
    checks = []

    def finish(passed):
        report = dict(passed=passed, checks=checks, reload_requested=args.reload,
                      release_requested=args.release, native_state_requested=args.native_state, pixels_requested=args.pixels, editor_requested=args.editor, diagnostics_requested=args.diagnostics, full_requested=args.full,
                      completed_at=datetime.now(timezone.utc).isoformat(),
                      scope='Default: Rust SDK, headless guides and CLI. Optional: Linux dylib watcher and isolated release capture. No TSX rendering or cross-platform proof.')
        (ROOT / 'reports').mkdir(exist_ok=True)
        (ROOT / 'reports/native-check.json').write_text(json.dumps(report, indent=2) + '\n')
        return 0 if passed else 1

    def run(name, command):
        started = time.monotonic()
        result = subprocess.run(command, cwd=ROOT)
        checks.append(dict(name=name, passed=result.returncode == 0,
                           elapsed_seconds=round(time.monotonic() - started, 2)))
        if result.returncode:
            return False
        return True

    # The guides are executable examples; reject documentation/test drift before
    # Cargo work so passing tests cannot conceal an untested code fence.
    for guide, fixture in [('TAG_UI.md', 'tag_ui_guide.rs'), ('ASYNC.md', 'async_guide.rs')]:
        base = ROOT / 'crates/wabou-native'
        try:
            example = (base / guide).read_text().split('```rust\n', 1)[1].split('```', 1)[0]
            if example != (base / 'tests' / fixture).read_text():
                raise ValueError('example differs from its executable test')
        except (OSError, IndexError, ValueError) as error:
            print(f'{guide}: {error}', file=sys.stderr)
            checks.append(dict(name='guide-consistency', passed=False, guide=guide, error=str(error)))
            return finish(False)
    commands = [
        ('headless-sdk-and-guides', ['cargo', 'test', '-p', 'blockui-macros', '-p', 'blockui-test',
                                    '-p', 'wabou-native', '--features', 'wabou-native/testing']),
        ('cli-regression', ['cargo', 'test', '-p', 'wabou-cli', '--bin', 'wabou']),
    ]
    if args.full:
        commands += [("public-sdk-lint", ["cargo", "clippy", "-p", "wabou-native", "-p", "blockui-cli", "--all-targets", "--features", "wabou-native/native,wabou-native/testing,wabou-native/lucide", "--", "-D", "warnings"])]
    if args.diagnostics:
        commands += [("authored-diagnostics", [sys.executable, "scripts/native/verify_diagnostics.py"])]
    if args.editor:
        commands += [("workspace-editor", [sys.executable, "scripts/native/verify_macro_ui_lsp.py", "--tags"]),
                     ("sdk-editor", [sys.executable, "scripts/native/verify_editor.py"])]
    if args.native_state:
        commands += [('native-widget-state', ['cargo', 'test', '-p', 'blockui-runtime', '--features', 'native,static-app,testing', '--lib']),
                     ('native-services', ['cargo', 'test', '-p', 'wabou-native', '--features', 'native,testing', '--test', 'native_services'])]
    if args.reload:
        commands += [('dylib-replacement', [sys.executable, 'scripts/native/verify_watch.py', '--trailing-ui', '--wabou-native', '--migration']),
                     ('async-replacement', [sys.executable, 'scripts/native/verify_watch.py', '--async-value', '--wabou-native'])]
    if args.pixels:
        commands += [("native-pixels", [sys.executable, "scripts/native/verify_pixels.py"])]
    if args.release:
        commands += [('cli-build', ['cargo', 'build', '-p', 'wabou-cli']),
                     ('linked-release', [sys.executable, 'scripts/native/verify_static_native.py', '--wabou-cli', 'target/debug/wabou'])]
    passed = True
    for name, command in commands:
        print(f'Checking {name}', flush=True)
        if not run(name, command):
            passed = False
            break
    return finish(passed)


if __name__ == '__main__':
    raise SystemExit(main())
