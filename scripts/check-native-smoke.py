#!/usr/bin/env python3
"""Linux Tauri/WebKit smoke test, using isolated XDG data and explicit fixtures.

First build the embedded native app with:
  npx tauri build --debug --no-bundle --config apps/desktop/tauri.conf.json
Then run:
  xvfb-run -a -s '-screen 0 1440x900x24' python3 scripts/check-native-smoke.py

Requires Xvfb, ImageMagick import, libX11, libXtst and the Tauri runtime libraries.
This checks the native shell and data migration, not Windows or Minecraft play.
"""
from pathlib import Path
import os, subprocess, tempfile, time, json, ctypes, signal
root = Path(__file__).resolve().parents[1]
shots = root / 'checks/continuation/native'
shots.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='solara-native-smoke-') as data:
    base = Path(data)
    env = os.environ.copy()
    for name, folder in [('XDG_DATA_HOME','data'),('XDG_CONFIG_HOME','config'),('XDG_CACHE_HOME','cache'),('XDG_RUNTIME_DIR','runtime')]:
        path = base / folder
        path.mkdir(mode=0o700)
        env[name] = str(path)
    app_data = base / 'data/in.solara.launcher'
    instance_dir = app_data / 'custom-instances/11111111-2222-4333-8444-555555555555'
    world = instance_dir / 'game/saves/PreservationFixture/level.dat'
    world.parent.mkdir(parents=True)
    world.write_bytes(b'NATIVE_PRESERVATION_FIXTURE')
    (instance_dir / 'instance.json').write_text(json.dumps({
        'schemaVersion':1,'id':instance_dir.name,'name':'Native preservation fixture',
        'gameVersion':'1.21.4','loader':'vanilla','loaderVersion':None,'memory':2048,
        'javaPath':'auto','state':'installed','icon':'preset:dawn-crest','pinned':True,
    }))
    prefs_file = app_data / 'preferences.json'
    prefs_file.write_text(json.dumps({'prefsVersion':1,'onLaunch':'close','onboardingCompleted':True,'memory':2048}))
    x11 = ctypes.CDLL('libX11.so.6')
    xtst = ctypes.CDLL('libXtst.so.6')
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    display = x11.XOpenDisplay(None)
    x11.XStringToKeysym.argtypes = [ctypes.c_char_p]
    x11.XStringToKeysym.restype = ctypes.c_ulong
    x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p,ctypes.c_ulong]
    x11.XKeysymToKeycode.restype = ctypes.c_uint
    x11.XFlush.argtypes = [ctypes.c_void_p]
    xtst.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p,ctypes.c_uint,ctypes.c_int,ctypes.c_ulong]
    def key(name,down):
        code=x11.XKeysymToKeycode(display,x11.XStringToKeysym(name.encode()))
        xtst.XTestFakeKeyEvent(display,code,int(down),0)
        x11.XFlush(display)
    def shortcut(names):
        for name in names: key(name,True)
        for name in reversed(names): key(name,False)
        time.sleep(1)
    env['WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS'] = '1'
    env['WEBKIT_DISABLE_DMABUF_RENDERER'] = '1'
    log = open(shots / 'native-smoke.log', 'w')
    app = subprocess.Popen(['dbus-run-session', '--', str(root / 'target/debug/solaralauncher')], env=env, stdout=log, stderr=log, start_new_session=True)
    try:
        time.sleep(8)
        if app.poll() is not None:
            raise RuntimeError(f'Native desktop exited before screenshot: {app.returncode}')
        subprocess.run(['import','-window','root',str(shots / 'startup.png')], check=True, timeout=10)
        print('Native desktop remained running after startup. Screenshot:', shots / 'startup.png', flush=True)
        assert world.read_bytes() == b'NATIVE_PRESERVATION_FIXTURE', 'Startup modified existing worlds'
        prefs = json.loads(prefs_file.read_text())
        assert prefs['onLaunch'] == 'keep' and prefs['prefsVersion'] == 2, 'Legacy close default was not migrated'
        assert prefs['memory'] == 2048, 'Migration lost other preferences'
        assert (app_data / '.clean_slate_v23_reset').exists(), 'Retired marker not recorded'
        shortcut(['Control_L','comma'])
        subprocess.run(['import','-window','root',str(shots / 'settings.png')],check=True,timeout=10)
        shortcut(['Escape'])
        subprocess.run(['import','-window','root',str(shots / 'settings-closed.png')],check=True,timeout=10)
        assert app.poll() is None, 'Desktop closed while using settings'
        assert world.read_bytes() == b'NATIVE_PRESERVATION_FIXTURE'
        print('PASS: native startup preserves existing world data without a reset marker; legacy Close becomes Keep; other preferences preserved; Settings opens/closes while app remains running.',flush=True)
    finally:
        if app.poll() is None:
            os.killpg(app.pid, signal.SIGTERM)
            try: app.wait(timeout=8)
            except subprocess.TimeoutExpired:
                os.killpg(app.pid, signal.SIGKILL)
                app.wait()
        log.close()
