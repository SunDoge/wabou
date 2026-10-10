"""Dedicated Xvfb lifecycle; never use the active desktop's display or input."""
import contextlib,os,pathlib,subprocess,tempfile,time
@contextlib.contextmanager
def isolated_display(base_env):
    display=next(f':{n}' for n in range(90,150) if not pathlib.Path(f'/tmp/.X11-unix/X{n}').exists() and not pathlib.Path(f'/tmp/.X{n}-lock').exists())
    with tempfile.TemporaryDirectory(prefix='identity-xdg-') as xdg:
        os.chmod(xdg,0o700)
        process=subprocess.Popen(['Xvfb',display,'-screen','0','1024x768x24','-nolisten','tcp'],stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
        try:
            for _ in range(100):
                if pathlib.Path(f'/tmp/.X11-unix/X{display[1:]}').exists():break
                if process.poll() is not None:raise RuntimeError(process.stderr.read().decode())
                time.sleep(.05)
            else:raise TimeoutError('Xvfb did not create its isolated socket')
            env=dict(base_env,DISPLAY=display,XDG_RUNTIME_DIR=xdg);env.pop('WAYLAND_DISPLAY',None)
            yield env
        finally:
            process.terminate();process.wait(timeout=5)
