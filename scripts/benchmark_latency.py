"""Measure p50 and p99 latency of atl commands against a local mock server.

Each sample is one atl process: wall time from spawn to exit, plus its CPU
time and peak memory. With several binaries (`--bin base=... --bin new=...`)
the runs alternate, so both see the same machine load.

    python3 scripts/benchmark_latency.py --bin atl=target/release/atl
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import TypedDict

from pgo_common import MARKDOWN, OK, STORAGE, MockAtlassian, Workload, prepare_fixture

# Delay of the mock server in the bulk workload, like a remote Jira site.
BULK_DELAY_SECONDS = 0.1
BULK_KEYS = tuple(f'ABC-{number}' for number in range(1, 9))


def rpc(*messages: dict[str, object]) -> str:
    return ''.join(json.dumps(message) + '\n' for message in messages)


MCP_START = (
    {
        'jsonrpc': '2.0',
        'id': 1,
        'method': 'initialize',
        'params': {
            'protocolVersion': '2025-06-18',
            'capabilities': {},
            'clientInfo': {'name': 'benchmark', 'version': '1'},
        },
    },
    {'jsonrpc': '2.0', 'method': 'notifications/initialized'},
)

WORKLOADS = (
    Workload('atl --version', ('--version',), 'atl'),
    Workload('atl --help', ('--help',), 'Usage:'),
    Workload(
        'convert Markdown to ADF', ('convert', '--to', 'adf'), '"doc"', stdin=MARKDOWN
    ),
    Workload(
        'convert storage to Markdown',
        ('convert', '--from', 'storage', '--to', 'markdown'),
        '# Handbook',
        stdin=STORAGE,
    ),
    Workload('schema jira.getIssue', ('schema', 'jira.getIssue', '--agent'), OK),
    Workload(
        'api jira get-issue --dry-run',
        (
            'api',
            'jira',
            'issues',
            'get-issue',
            '--issue-id-or-key',
            'ABC-1',
            '--dry-run',
        ),
        'ABC-1',
    ),
    Workload(
        'mcp: start and list tools',
        ('mcp',),
        '"tools"',
        stdin=rpc(*MCP_START, {'jsonrpc': '2.0', 'id': 2, 'method': 'tools/list'}),
    ),
    Workload(
        'mcp: start and search',
        ('mcp',),
        'total_matches',
        stdin=rpc(
            *MCP_START,
            {
                'jsonrpc': '2.0',
                'id': 2,
                'method': 'tools/call',
                'params': {
                    'name': 'atlassian_search_operations',
                    'arguments': {'query': 'create issue'},
                },
            },
        ),
    ),
    Workload(
        'jira issue list',
        ('jira', 'issue', 'list', '-p', 'ABC', '--agent'),
        OK,
        network=True,
    ),
    Workload(
        'jira issue view',
        ('jira', 'issue', 'view', 'ABC-1', '--agent'),
        OK,
        network=True,
    ),
    Workload(
        'api jira get-issue',
        ('api', 'jira', 'issues', 'get-issue', '--issue-id-or-key', 'ABC-2', '--agent'),
        OK,
        network=True,
    ),
    Workload(
        'confluence page get',
        ('confluence', 'page', 'get', '123', '--agent'),
        OK,
        network=True,
    ),
)

BULK = Workload(
    f'jira issue move, {len(BULK_KEYS)} issues',
    ('jira', 'issue', 'move', *BULK_KEYS, 'Done', '--agent'),
    OK,
    network=True,
)


class Summary(TypedDict):
    p50_ms: float
    p99_ms: float
    cpu_p50_ms: float
    peak_memory_p50_mib: float
    samples: int


@dataclass(slots=True)
class Samples:
    wall_ms: list[float]
    cpu_ms: list[float]
    peak_mib: list[float]


def percentile(values: list[float], percent: int) -> float:
    """Nearest-rank percentile: the smallest sample with `percent`% at or below it."""

    ordered = sorted(values)
    rank = -(-len(ordered) * percent // 100)
    return ordered[max(rank, 1) - 1]


def run_once(
    binary: Path,
    workload: Workload,
    cwd: Path,
    environment: dict[str, str],
    scratch: Path,
) -> tuple[float, float, float, str]:
    stdin_path = scratch / 'stdin'
    stdin_path.write_text(workload.stdin or '', encoding='utf-8')
    output_path = scratch / 'output'
    with stdin_path.open('rb') as stdin, output_path.open('wb') as output:
        started = time.perf_counter_ns()
        process = subprocess.Popen(
            [str(binary), *workload.arguments],
            cwd=cwd,
            env=environment,
            stdin=stdin,
            stdout=output,
            stderr=subprocess.STDOUT,
        )
        # wait4 returns this child's own CPU time and peak memory.
        _, status, usage = os.wait4(process.pid, 0)
        elapsed_ns = time.perf_counter_ns() - started
        process.returncode = os.waitstatus_to_exitcode(status)
    text = output_path.read_text(encoding='utf-8', errors='replace')
    if process.returncode != 0 or workload.expected_output not in text:
        raise RuntimeError(
            f'{binary} {workload.name} exited {process.returncode}: {text[:2000]}'
        )
    # ru_maxrss is in bytes on macOS and in KiB on Linux.
    peak_bytes = usage.ru_maxrss if sys.platform == 'darwin' else usage.ru_maxrss * 1024
    return (
        elapsed_ns / 1e6,
        (usage.ru_utime + usage.ru_stime) * 1e3,
        peak_bytes / 1048576,
        text,
    )


def measure(
    binaries: dict[str, Path],
    workload: Workload,
    samples: int,
    warmups: int,
    cwd: Path,
    environment: dict[str, str],
    scratch: Path,
) -> dict[str, Summary]:
    collected = {label: Samples([], [], []) for label in binaries}
    for binary in binaries.values():
        for _ in range(warmups):
            run_once(binary, workload, cwd, environment, scratch)
    labels = list(binaries)
    for index in range(samples):
        for label in labels if index % 2 == 0 else reversed(labels):
            wall, cpu, peak, _ = run_once(
                binaries[label], workload, cwd, environment, scratch
            )
            collected[label].wall_ms.append(wall)
            collected[label].cpu_ms.append(cpu)
            collected[label].peak_mib.append(peak)
    return {
        label: Summary(
            p50_ms=percentile(values.wall_ms, 50),
            p99_ms=percentile(values.wall_ms, 99),
            cpu_p50_ms=percentile(values.cpu_ms, 50),
            peak_memory_p50_mib=percentile(values.peak_mib, 50),
            samples=samples,
        )
        for label, values in collected.items()
    }


def main() -> None:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        '--bin',
        action='append',
        required=True,
        metavar='LABEL=PATH',
        help='an atl binary to measure; repeat to compare',
    )
    parser.add_argument(
        '--samples',
        type=int,
        default=500,
        help='runs of each local workload (default 500)',
    )
    parser.add_argument(
        '--network-samples',
        type=int,
        default=300,
        help='runs of each workload that calls the mock server (default 300)',
    )
    parser.add_argument(
        '--bulk-samples',
        type=int,
        default=100,
        help='runs of the bulk move workload (default 100)',
    )
    parser.add_argument('--warmups', type=int, default=10)
    parser.add_argument('--json', type=Path, help='also write the results to this file')
    args = parser.parse_args()
    binaries: dict[str, Path] = {}
    for item in args.bin:
        label, separator, path = item.partition('=')
        if not separator or not Path(path).is_file():
            parser.error(
                f'--bin {item}: pass LABEL=PATH to an existing atl binary, for example --bin atl=target/release/atl'
            )
        binaries[label] = Path(path).resolve()
    if (
        min(args.samples, args.network_samples, args.bulk_samples) < 1
        or args.warmups < 0
    ):
        parser.error('sample counts must be at least 1 and --warmups at least 0')

    results: dict[str, dict[str, Summary]] = {}
    with tempfile.TemporaryDirectory(prefix='atl-latency-') as temporary:
        scratch = Path(temporary)
        for workload, delay, count in [
            *(
                (w, 0.0, args.network_samples if w.network else args.samples)
                for w in WORKLOADS
            ),
            (BULK, BULK_DELAY_SECONDS, args.bulk_samples),
        ]:
            with MockAtlassian(delay=delay) as server:
                cwd, environment = prepare_fixture(scratch / 'fixture', server.base_url)
                results[workload.name] = measure(
                    binaries, workload, count, args.warmups, cwd, environment, scratch
                )
            cells = '  '.join(
                f'{label}: p50 {row["p50_ms"]:7.1f} ms  p99 {row["p99_ms"]:7.1f} ms  CPU {row["cpu_p50_ms"]:5.1f} ms  {row["peak_memory_p50_mib"]:5.1f} MiB'
                for label, row in results[workload.name].items()
            )
            print(f'{workload.name:<32} {cells}', flush=True)
    print(
        f'{platform.system()} {platform.release()} {platform.machine()}, {os.cpu_count()} CPUs, load {os.getloadavg()[0]:.1f}'
    )
    if args.json:
        args.json.write_text(
            json.dumps(results, indent=2) + '\n', encoding='utf-8', newline='\n'
        )


if __name__ == '__main__':
    main()
