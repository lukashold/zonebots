"""Keep generated inputs/numeric FNV proofs; never publish raw record frames."""
import argparse
import json
import struct
from pathlib import Path


def fnv(data):
    value = 2166136261
    for byte in data:
        value = ((value ^ byte) * 16777619) & 0xFFFFFFFF
    return value


def compact(root):
    result = {}
    for name in ["v43-building-loader", "v43-trail-job-birth",
                 "v43-double-destruction", "v43-owner4-particle",
                 "v43-trail-playback", "v43-crane-work",
                 "v43-owner-blast-prefix"]:
        rows = []
        for line in (root / (name + ".jsonl")).read_text(encoding="utf-8").splitlines():
            source = json.loads(line)
            record = {}
            for key, value in source.items():
                if name == "v43-owner-blast-prefix" and key == "source":
                    continue
                if isinstance(value, str):
                    record[key + "Hash"] = fnv(bytes.fromhex(value))
                elif key == "draws":
                    record["drawsHash"] = fnv(b"".join(struct.pack("<q", n) for row in value for n in row))
                    record["drawsCount"] = len(value)
                elif key != "events":
                    record[key] = value
            if name == "v43-owner-blast-prefix":
                record["sourceHash"] = fnv(bytes.fromhex(source["source"]))
            rows.append(record)
        columns = list(rows[0])
        assert all(list(row) == columns for row in rows)
        result[name + ".jsonl"] = {"columns": columns, "rows": [[row[key] for key in columns] for row in rows]}
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("private_trace_directory", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps(compact(args.private_trace_directory), separators=(",", ":")), encoding="ascii")
