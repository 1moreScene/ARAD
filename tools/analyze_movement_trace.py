"""Compact read-only official movement samples into reviewable state events."""
import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path


def main():
    parser=argparse.ArgumentParser(__doc__)
    parser.add_argument("trace",type=Path)
    parser.add_argument("--out",type=Path,required=True)
    args=parser.parse_args()
    rows=[json.loads(line) for line in args.trace.read_text(encoding="utf-8").splitlines()]
    events=[]
    previous=None
    for row in rows:
        if "state" not in row:
            continue
        signature=(row["state"],row["jump_phase"],row.get("frame"),row.get("facing"),
                   row.get("pending_state"),tuple(row["directions"]),tuple(row["speeds"]),
                   tuple(row["rates"]),tuple(row["dash_commands"]),
                   tuple((record[0],record[1],record[3]) for record in row["records"]))
        if signature!=previous:
            # Omit heap addresses; retain the actual observations and timestamps.
            events.append({key:value for key,value in row.items() if key not in ("actor","vtable")})
            previous=signature
    counts=Counter(str(row.get("state")) for row in rows)
    jump_rows=[row for row in rows if row.get("state")==6]
    report={
        "source_trace":str(args.trace),
        "source_sha256":hashlib.sha256(args.trace.read_bytes()).hexdigest(),
        "samples":len(rows),"states":dict(counts),
        "observations":{
            "run_seen":any(row.get("state")==11 for row in rows),
            "walk_jump_y_halves_seen":any(row.get("speeds")==[143,57,119,47] for row in jump_rows),
            "jump_phases":sorted({row["jump_phase"] for row in jump_rows}),
            "jump_frame_eight_seen":any(row.get("frame")==8 for row in jump_rows),
            "landing_frame_fifteen_seen":any(row["jump_phase"]==2 and row.get("frame")==15 for row in jump_rows),
            "candidate_without_run_seen":any(row.get("state")==0 and row["dash_commands"] not in ([0,0],) for row in rows),
            "pending_run_without_effective_run_seen":any(row.get("pending_state")==11 and row.get("state")==0 for row in rows),
        },
        "events":events,
    }
    args.out.parent.mkdir(parents=True,exist_ok=True)
    args.out.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding="utf-8")
    print(json.dumps({k:v for k,v in report.items() if k!="events"},ensure_ascii=False))
    print(f"events={len(events)} -> {args.out}")


if __name__=="__main__":
    main()
