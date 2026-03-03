#!/usr/bin/env python3
import argparse
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import termios
import time


def parse_keys(value: str) -> bytes:
    return value.encode("utf-8").decode("unicode_escape").encode("utf-8")


def set_winsize(fd: int, rows: int, cols: int) -> None:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def main() -> int:
    parser = argparse.ArgumentParser(description="Capture one real Pulsar TUI frame through a PTY.")
    parser.add_argument("--output", required=True, help="Path to the raw typescript output file")
    parser.add_argument("--rows", type=int, default=40)
    parser.add_argument("--cols", type=int, default=140)
    parser.add_argument("--start-delay", type=float, default=1.4)
    parser.add_argument("--linger", type=float, default=1.0)
    parser.add_argument("--key-delay", type=float, default=0.5)
    parser.add_argument(
        "--keys",
        action="append",
        default=[],
        help=r"Key sequence to send, for example 8 or \\/? . Can be repeated.",
    )
    parser.add_argument(
        "command",
        nargs=argparse.REMAINDER,
        help="Command to execute after --, for example -- target/debug/pulsar",
    )
    args = parser.parse_args()

    command = args.command
    if command and command[0] == "--":
        command = command[1:]
    if not command:
        parser.error("missing command after --")

    master_fd, slave_fd = pty.openpty()
    set_winsize(slave_fd, args.rows, args.cols)

    proc = subprocess.Popen(
        command,
        stdin=slave_fd,
        stdout=slave_fd,
        stderr=slave_fd,
        close_fds=True,
        start_new_session=True,
    )
    os.close(slave_fd)

    captured = bytearray()
    send_plan = []
    current_time = args.start_delay
    for item in args.keys:
        send_plan.append((current_time, parse_keys(item)))
        current_time += args.key_delay
    end_time = current_time + args.linger

    started = time.monotonic()
    send_index = 0

    try:
        while True:
            elapsed = time.monotonic() - started
            while send_index < len(send_plan) and elapsed >= send_plan[send_index][0]:
                os.write(master_fd, send_plan[send_index][1])
                send_index += 1

            ready, _, _ = select.select([master_fd], [], [], 0.05)
            if ready:
                try:
                    chunk = os.read(master_fd, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                captured.extend(chunk)

            if proc.poll() is not None:
                break
            if elapsed >= end_time:
                break
    finally:
        if proc.poll() is None:
            try:
                os.killpg(proc.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                proc.wait(timeout=1.0)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(proc.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                proc.wait(timeout=1.0)
        os.close(master_fd)

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, "wb") as handle:
        handle.write(captured)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
