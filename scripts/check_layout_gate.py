#!/usr/bin/env python3
"""
Wave 0 Task L4: Layout Regression Gate Checker.
Validates canonical layout snapshot integrity, minimum panel dimension constraints,
and guards against unapproved layout geometry regressions.
"""

import sys
import os
import json
import subprocess
from pathlib import Path

SNAPSHOT_PATH = Path("docs/snapshots/layout-geometry.json")

PANEL_MIN = {
    "CurrentFolder": (220.0, 160.0),
    "Workspace": (220.0, 140.0),
    "Editor": (360.0, 240.0),
    "BlockDiagram": (400.0, 300.0),
    "CommandWindow": (360.0, 160.0),
    "Help": (280.0, 200.0),
    "Figures": (280.0, 200.0),
}


def validate_snapshot_data(data):
    """Validates structural and geometric integrity of snapshot data."""
    errors = []

    if not isinstance(data, dict):
        return ["Snapshot root must be a JSON object"]

    if data.get("version") != 1:
        errors.append(f"Expected schema version 1, got {data.get('version')}")

    resolutions = data.get("resolutions", [])
    if not resolutions:
        errors.append("Snapshot must contain at least one tested resolution")

    for res_idx, res in enumerate(resolutions):
        res_dims = res.get("resolution", [])
        if len(res_dims) != 2:
            errors.append(f"Resolution entry #{res_idx} missing 2D resolution [w, h]")
            continue

        dock_rect = res.get("dock_rect", [])
        if len(dock_rect) != 4:
            errors.append(f"Resolution {res_dims} missing dock_rect [x, y, w, h]")
            continue

        dock_min_x, dock_min_y, dock_w, dock_h = dock_rect
        dock_max_x = dock_min_x + dock_w
        dock_max_y = dock_min_y + dock_h

        panels = res.get("panels", [])
        if not panels:
            errors.append(f"Resolution {res_dims} contains zero panels")
            continue

        # Check per-panel minimum floors and boundary containment
        for p in panels:
            pid = p.get("panel_id", "Unknown")
            px = p.get("min_x", 0.0)
            py = p.get("min_y", 0.0)
            pw = p.get("width", 0.0)
            ph = p.get("height", 0.0)

            # Look up required minimums
            req_w, req_h = PANEL_MIN.get(pid, (p.get("min_width_required", 0.0), p.get("min_height_required", 0.0)))

            if pw < req_w - 0.5:
                errors.append(
                    f"Resolution {res_dims}: Panel {pid} width {pw:.1f} is below minimum {req_w:.1f}"
                )
            if ph < req_h - 0.5:
                errors.append(
                    f"Resolution {res_dims}: Panel {pid} height {ph:.1f} is below minimum {req_h:.1f}"
                )

            p_max_x = px + pw
            p_max_y = py + ph

            if (
                px < dock_min_x - 0.5
                or p_max_x > dock_max_x + 0.5
                or py < dock_min_y - 0.5
                or p_max_y > dock_max_y + 0.5
            ):
                errors.append(
                    f"Resolution {res_dims}: Panel {pid} bounds [{px:.1f}, {py:.1f}, {p_max_x:.1f}, {p_max_y:.1f}] "
                    f"clipped outside dock [{dock_min_x:.1f}, {dock_min_y:.1f}, {dock_max_x:.1f}, {dock_max_y:.1f}]"
                )

        # Check pairwise non-overlapping geometry
        for i in range(len(panels)):
            for j in range(i + 1, len(panels)):
                a = panels[i]
                b = panels[j]

                a_max_x = a.get("min_x", 0.0) + a.get("width", 0.0)
                a_max_y = a.get("min_y", 0.0) + a.get("height", 0.0)
                b_max_x = b.get("min_x", 0.0) + b.get("width", 0.0)
                b_max_y = b.get("min_y", 0.0) + b.get("height", 0.0)

                overlap_x = max(0.0, min(a_max_x, b_max_x) - max(a.get("min_x", 0.0), b.get("min_x", 0.0)))
                overlap_y = max(0.0, min(a_max_y, b_max_y) - max(a.get("min_y", 0.0), b.get("min_y", 0.0)))

                if overlap_x > 0.5 and overlap_y > 0.5:
                    errors.append(
                        f"Resolution {res_dims}: Panels {a.get('panel_id')} and {b.get('panel_id')} "
                        f"overlap by {overlap_x:.1f}x{overlap_y:.1f}"
                    )

    return errors


def check_git_diff_approval(snapshot_path):
    """
    Checks if the snapshot was modified relative to git base and verifies
    whether an explicit approval rationale is present.
    """
    try:
        # Check if modified against HEAD or origin/main
        diff_proc = subprocess.run(
            ["git", "diff", "--name-only", "HEAD", "--", str(snapshot_path)],
            capture_output=True,
            text=True,
            check=False,
        )
        if str(snapshot_path).replace("\\", "/") in diff_proc.stdout.replace("\\", "/"):
            # Check commit messages or environment override
            if os.environ.get("LAYOUT_APPROVED") == "1":
                return True, "Approved via LAYOUT_APPROVED environment override"

            log_proc = subprocess.run(
                ["git", "log", "-n", "5", "--pretty=%B"],
                capture_output=True,
                text=True,
                check=False,
            )
            commit_text = log_proc.stdout.lower()
            if "layout-approved" in commit_text or "layout approved" in commit_text:
                return True, "Approved via commit log tag [layout-approved]"

            return False, "Unapproved modification detected in working tree"

        return True, "Snapshot unchanged in working tree"
    except Exception as e:
        # If git is unavailable, proceed with file contents check
        return True, f"Git check skipped: {e}"


def run_self_test():
    """Runs internal self-tests asserting detector accuracy on positive and negative cases."""
    print("Running layout gate self-tests...")

    valid_data = {
        "version": 1,
        "schema": "matrust-layout-snapshot-v1",
        "approved_reason": "test",
        "resolutions": [
            {
                "resolution": [900.0, 600.0],
                "dock_rect": [0.0, 115.0, 900.0, 460.0],
                "panels": [
                    {
                        "panel_id": "CurrentFolder",
                        "min_x": 0.0,
                        "min_y": 115.0,
                        "width": 220.0,
                        "height": 460.0,
                        "min_width_required": 220.0,
                        "min_height_required": 160.0,
                    },
                    {
                        "panel_id": "Editor",
                        "min_x": 225.0,
                        "min_y": 115.0,
                        "width": 675.0,
                        "height": 460.0,
                        "min_width_required": 360.0,
                        "min_height_required": 240.0,
                    },
                ],
            }
        ],
    }

    # Case 1: Valid data passes
    errs = validate_snapshot_data(valid_data)
    assert not errs, f"Expected clean pass, got: {errs}"

    # Case 2: Shrinking panel below minimum fails
    bad_width_data = json.loads(json.dumps(valid_data))
    bad_width_data["resolutions"][0]["panels"][0]["width"] = 150.0  # < 220.0
    errs = validate_snapshot_data(bad_width_data)
    assert any("below minimum" in e for e in errs), "Failed to detect width below minimum"

    # Case 3: Overlapping panels fail
    overlap_data = json.loads(json.dumps(valid_data))
    overlap_data["resolutions"][0]["panels"][1]["min_x"] = 100.0  # Overlaps panel 0 [0..220]
    errs = validate_snapshot_data(overlap_data)
    assert any("overlap" in e for e in errs), "Failed to detect overlapping panels"

    # Case 4: Out-of-bounds panel fails
    oob_data = json.loads(json.dumps(valid_data))
    oob_data["resolutions"][0]["panels"][1]["width"] = 1000.0  # Extends past 900
    errs = validate_snapshot_data(oob_data)
    assert any("clipped outside dock" in e for e in errs), "Failed to detect out-of-bounds panel"

    print("All layout gate self-tests PASSED.")
    return 0


def main():
    if "--self-test" in sys.argv:
        return run_self_test()

    print("=== Wave 0 Task L4: Layout Regression Gate Checker ===")

    if not SNAPSHOT_PATH.exists():
        print(f"ERROR: Canonical layout snapshot file missing at {SNAPSHOT_PATH}")
        return 1

    try:
        with open(SNAPSHOT_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:
        print(f"ERROR: Failed to parse snapshot JSON at {SNAPSHOT_PATH}: {e}")
        return 1

    errors = validate_snapshot_data(data)
    if errors:
        print(f"ERROR: Layout snapshot integrity violations detected in {SNAPSHOT_PATH}:")
        for err in errors:
            print(f"  - {err}")
        return 1

    approved, reason = check_git_diff_approval(SNAPSHOT_PATH)
    if not approved:
        print(f"ERROR: {reason}")
        print("To update the layout snapshot, include [layout-approved] in your commit message or set LAYOUT_APPROVED=1.")
        return 1

    resolutions = data.get("resolutions", [])
    print(f"Verified canonical layout snapshot {SNAPSHOT_PATH} across {len(resolutions)} supported resolutions:")
    for res in resolutions:
        r = res.get("resolution", [])
        panels = res.get("panels", [])
        print(f"  - Resolution {r[0]:.0f}x{r[1]:.0f}: {len(panels)} active panels, all minimum bounds satisfied, zero overlaps.")

    print(f"Snapshot approval status: {reason}")
    print("Layout regression gate PASSED.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
