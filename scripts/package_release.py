"""Package the native Rust binary together with its built frontend."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TARGETS = {
    "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin", "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc",
}


def smoke_test(package, binary):
    with tempfile.TemporaryDirectory() as tmp:
        home = Path(tmp)
        env = dict(os.environ, HOME=str(home), USERPROFILE=str(home))
        env.pop("AI_SESSION_WEB_DIR", None)
        executable = package / binary
        subprocess.run([str(executable), "--help"], cwd=home, env=env,
                       check=True, timeout=30)
        with subprocess.Popen([str(executable)], cwd=home, env=env) as proc:
            try:
                deadline = time.monotonic() + 30
                while time.monotonic() < deadline:
                    if proc.poll() is not None:
                        raise RuntimeError(f"Server exited: {proc.returncode}")
                    try:
                        with urllib.request.urlopen("http://127.0.0.1:8788/", timeout=2) as r:
                            assert r.status == 200 and b"<html" in r.read().lower()
                        with urllib.request.urlopen("http://127.0.0.1:8788/api/sources", timeout=2) as r:
                            assert isinstance(json.load(r), list)
                        return
                    except OSError:
                        time.sleep(0.5)
                raise RuntimeError("Server did not become ready")
            finally:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("target", choices=sorted(TARGETS))
    parser.add_argument("--smoke-test", action="store_true")
    args = parser.parse_args()
    windows = "windows" in args.target
    binary = "ai-session-export-server" + (".exe" if windows else "")
    src = ROOT / "server" / "target" / args.target / "release" / binary
    web = ROOT / "web" / "dist"
    if not src.is_file() or not (web / "index.html").is_file():
        raise SystemExit("Build the Rust release and web/dist before packaging.")
    output = ROOT / "release-assets"
    output.mkdir(exist_ok=True)
    name = f"ai-session-export-{args.target}"
    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp) / name
        stage.mkdir()
        shutil.copy2(src, stage / binary)
        shutil.copytree(web, stage / "web" / "dist")
        for filename in ("README.md", "LICENSE"):
            shutil.copy2(ROOT / filename, stage / filename)
        archive = shutil.make_archive(str(output / name),
                                      "zip" if windows else "gztar",
                                      root_dir=tmp, base_dir=name)
        if args.smoke_test:
            extracted = Path(tmp) / "extracted"
            extracted.mkdir()
            shutil.unpack_archive(archive, extracted)
            smoke_test(extracted / name, binary)
        print(archive)


if __name__ == "__main__":
    main()
