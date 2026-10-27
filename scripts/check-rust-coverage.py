#!/usr/bin/env python3
"""Measure unique production code regions from LLVM's exported function data."""

import json
import sys
from pathlib import Path


def summarize(report, source_root):
    source_root = source_root.resolve()
    regions = {}
    for unit in report["data"]:
        for function in unit["functions"]:
            # Rust v0 mangling represents this project's cfg(test) module name
            # as the length-prefixed component 5tests. Test execution must not
            # increase the production coverage denominator or numerator.
            if "5tests" in function["name"]:
                continue
            for (
                start_line,
                start_column,
                end_line,
                end_column,
                count,
                file_id,
                _expanded,
                kind,
            ) in function["regions"]:
                if kind != 0:  # CodeRegion; expansion/gap/skipped regions are not executable code.
                    continue
                path = Path(function["filenames"][file_id]).resolve()
                if not path.is_relative_to(source_root):
                    continue
                relative = path.relative_to(source_root).as_posix()
                if relative == "tools/tests.rs":
                    continue
                key = (relative, start_line, start_column, end_line, end_column)
                # Generic instantiations and unit/process binaries may report
                # the same source region more than once. Count its union once.
                regions[key] = max(count, regions.get(key, 0))
    if not regions:
        raise ValueError("No production code regions were found")
    covered = sum(count > 0 for count in regions.values())
    per_file = {}
    for (filename, *_span), count in regions.items():
        entry = per_file.setdefault(filename, {"regions": 0, "covered": 0})
        entry["regions"] += 1
        entry["covered"] += count > 0
    return {
        "metric": "LLVM production code regions",
        "regions": len(regions),
        "covered": covered,
        "percent": covered * 100 / len(regions),
        "files": per_file,
    }


if __name__ == "__main__":
    result = summarize(
        json.loads(Path(sys.argv[1]).read_text()), Path(__file__).resolve().parents[1] / "src"
    )
    print(json.dumps(result, indent=2, sort_keys=True))
    if result["percent"] < 80:
        sys.exit("Production Rust region coverage is below 80%")
