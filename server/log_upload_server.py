#!/usr/bin/env python3
"""
Tiny HTTP server that accepts POST /app/upload_log and writes the body
to a timestamped file under UPLOAD_DIR.

The directory is capped at MAX_DIR_BYTES (default 50 MB) — oldest files
are deleted when the limit is exceeded.

Run:  python3 log_upload_server.py
      (listens on 127.0.0.1:9977 by default, only reachable from nginx)
"""

import os, time, glob
from http.server import HTTPServer, BaseHTTPRequestHandler

UPLOAD_DIR = os.environ.get("UPLOAD_DIR", "/var/www/html/app/uploads")
MAX_DIR_BYTES = int(os.environ.get("MAX_DIR_MB", "50")) * 1024 * 1024
MAX_BODY = 1 * 1024 * 1024  # 1 MB per request (matches nginx client_max_body_size)
LISTEN_PORT = int(os.environ.get("LISTEN_PORT", "9977"))


def dir_size(path):
    total = 0
    for f in glob.glob(os.path.join(path, "*")):
        if os.path.isfile(f):
            total += os.path.getsize(f)
    return total


def enforce_quota(path, max_bytes):
    """Delete oldest files until total size is under max_bytes."""
    files = sorted(glob.glob(os.path.join(path, "*")), key=os.path.getmtime)
    total = sum(os.path.getsize(f) for f in files if os.path.isfile(f))
    while total > max_bytes and files:
        victim = files.pop(0)
        sz = os.path.getsize(victim)
        os.remove(victim)
        total -= sz


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        if length > MAX_BODY:
            self.send_response(413)
            self.end_headers()
            self.wfile.write(b"TOO_LARGE")
            return

        body = self.rfile.read(length)
        os.makedirs(UPLOAD_DIR, exist_ok=True)

        filename = time.strftime("%Y%m%d-%H%M%S") + f"-{int(time.time()*1000)%10000}.log"
        filepath = os.path.join(UPLOAD_DIR, filename)
        with open(filepath, "wb") as f:
            f.write(body)

        # Enforce directory quota
        enforce_quota(UPLOAD_DIR, MAX_DIR_BYTES)

        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(b"OK")

    def do_OPTIONS(self):
        self.send_response(204)
        self.end_headers()

    # Suppress noisy request logging
    def log_message(self, fmt, *args):
        pass


if __name__ == "__main__":
    print(f"[log_upload_server] Listening on 127.0.0.1:{LISTEN_PORT}, dir={UPLOAD_DIR}, quota={MAX_DIR_BYTES//1024//1024}MB")
    server = HTTPServer(("127.0.0.1", LISTEN_PORT), Handler)
    server.serve_forever()
