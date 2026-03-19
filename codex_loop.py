#!/usr/bin/env python3
"""Loop Codex executions using the local command file as the prompt source."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Repeatedly run `codex exec` with the contents of the current "
            "working directory's command file."
        )
    )
    parser.add_argument(
        "--command-file",
        default="Boot.md",
        help="Prompt file to read before every Codex run. Defaults to `command`.",
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=0.0,
        help="Seconds to wait between runs. Defaults to 0.",
    )
    parser.add_argument(
        "--max-runs",
        type=int,
        default=None,
        help="Stop after this many runs. Defaults to infinite looping.",
    )
    parser.add_argument(
        "--codex-bin",
        default="codex",
        help="Codex executable name or absolute path. Defaults to `codex`.",
    )
    parser.add_argument(
        "--model",
        default="gpt-5.4",
        help="Model passed to Codex. Defaults to `gpt-5.4`.",
    )
    parser.add_argument(
        "--reasoning-effort",
        default="high",
        help=(
            "Reasoning effort passed through `-c model_reasoning_effort=...`. "
            "Defaults to `high`."
        ),
    )
    parser.add_argument(
        "--dangerously-bypass-approvals-and-sandbox",
        action=argparse.BooleanOptionalAction,
        default=True,
        help=(
            "Pass through Codex's bypass flag so child runs execute without "
            "approvals or sandboxing. Enabled by default; use "
            "`--no-dangerously-bypass-approvals-and-sandbox` to disable."
        ),
    )
    return parser.parse_args()


def read_prompt(command_path: Path) -> str:
    if not command_path.is_file():
        raise FileNotFoundError(
            f"Prompt file not found: {command_path}. Expected a file named `command` in the working directory."
        )

    prompt = command_path.read_text(encoding="utf-8").strip()
    if not prompt:
        raise ValueError(f"Prompt file is empty: {command_path}")

    return prompt


def ensure_codex_exists(codex_bin: str) -> None:
    if Path(codex_bin).is_absolute():
        if not Path(codex_bin).exists():
            raise FileNotFoundError(f"Codex executable not found: {codex_bin}")
        return

    if shutil.which(codex_bin) is None:
        raise FileNotFoundError(
            f"Codex executable `{codex_bin}` was not found in PATH."
        )


def run_loop(args: argparse.Namespace) -> int:
    ensure_codex_exists(args.codex_bin)

    command_path = Path.cwd() / args.command_file
    run_count = 0
    last_exit_code = 0

    try:
        while args.max_runs is None or run_count < args.max_runs:
            prompt = read_prompt(command_path)
            run_count += 1

            started_at = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            print(
                f"[{started_at}] Starting Codex run #{run_count} using {command_path}",
                flush=True,
            )

            codex_command = [
                args.codex_bin,
                "exec",
                "--model",
                args.model,
                "-c",
                f'model_reasoning_effort="{args.reasoning_effort}"',
            ]
            if args.dangerously_bypass_approvals_and_sandbox:
                codex_command.append("--dangerously-bypass-approvals-and-sandbox")
            codex_command.append(prompt)
            completed = subprocess.run(
                codex_command,
                check=False,
                cwd=Path.cwd(),
            )
            last_exit_code = completed.returncode

            finished_at = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            print(
                f"[{finished_at}] Codex run #{run_count} finished with exit code {last_exit_code}",
                flush=True,
            )

            if args.max_runs is not None and run_count >= args.max_runs:
                break

            if args.delay > 0:
                time.sleep(args.delay)
    except KeyboardInterrupt:
        print("\nInterrupted by user.", file=sys.stderr, flush=True)
        return 130

    return last_exit_code


def main() -> int:
    args = parse_args()
    try:
        return run_loop(args)
    except (FileNotFoundError, ValueError) as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
