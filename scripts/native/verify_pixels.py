#!/usr/bin/env python3
"""Explicit native UI pixel checks in an isolated display, without desktop input."""
import json
import subprocess
import time
from dev import ROOT, ENV, cargo_tool
from display import isolated_display


def main():
    checks = []
    cargo = cargo_tool()
    with isolated_display(ENV) as environment:
        for case in ['hover_pixels', 'button_appearance_pixels', 'reactive_style_pixels']:
            started = time.monotonic()
            result = subprocess.run([cargo, 'test', '-p', 'blockui-runtime', '--features',
                                     'native,static-app,testing', '--lib', case, '--', '--ignored'],
                                    cwd=ROOT, env=environment, text=True, capture_output=True)
            passed = result.returncode == 0 and '1 passed; 0 failed' in result.stdout
            checks.append(dict(case=case, passed=passed, elapsed_seconds=round(time.monotonic() - started, 2)))
            print(result.stdout, end='')
            if not passed:
                print(result.stderr, end='')
                break
    report = dict(passed=len(checks) == 3 and all(check['passed'] for check in checks),
                  checks=checks, isolated_display=True,
                  scope='Native offscreen Vello pixels: hover/pressed/disabled at 1x and 2x, button appearance, reactive styles. No desktop input or actual dylib reload.')
    (ROOT / 'reports').mkdir(exist_ok=True)
    (ROOT / 'reports/native-pixels.json').write_text(json.dumps(report, indent=2) + '\n')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
