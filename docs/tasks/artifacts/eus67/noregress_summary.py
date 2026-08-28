#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.67(b) — summarise the RSS + wall-clock no-regression arm.

Reports, per input, the OLD->NEW change in median peak RSS and median wall time with the
instrument OFF. ⛔ MEDIAN of the repeats, not the minimum: taking the best run of each arm
compares two different tails and flatters whichever arm got the quieter machine.
"""
import statistics
import sys


def main(path):
    rows = [l.rstrip("\n").split("\t") for l in open(path)][1:]
    data = {}
    for arm, p, _rep, rss, wall in rows:
        if rss == "NA" or wall == "NA":
            continue
        data.setdefault(p, {}).setdefault(arm, []).append((int(rss), float(wall)))
    print(f"{'input':<32}{'n':>3}{'RSS old MB':>12}{'RSS new MB':>12}{'dRSS':>9}"
          f"{'wall old':>10}{'wall new':>10}{'dwall':>9}")
    worst_rss = worst_wall = 0.0
    for p, arms in data.items():
        if {"OLD", "NEW"} - arms.keys():
            print(f"{p.split('/')[-1]:<32} INCOMPLETE — one arm is missing, no delta derivable")
            continue
        o_rss = statistics.median(v[0] for v in arms["OLD"]) / 1048576
        n_rss = statistics.median(v[0] for v in arms["NEW"]) / 1048576
        o_w = statistics.median(v[1] for v in arms["OLD"])
        n_w = statistics.median(v[1] for v in arms["NEW"])
        d_rss = (n_rss - o_rss) / o_rss * 100
        d_w = (n_w - o_w) / o_w * 100 if o_w else 0.0
        worst_rss = max(worst_rss, abs(d_rss))
        worst_wall = max(worst_wall, abs(d_w))
        print(f"{p.split('/')[-1]:<32}{len(arms['NEW']):>3}{o_rss:>12.1f}{n_rss:>12.1f}"
              f"{d_rss:>8.2f}%{o_w:>10.2f}{n_w:>10.2f}{d_w:>8.2f}%")
    print(f"\nworst |dRSS| {worst_rss:.2f}% · worst |dwall| {worst_wall:.2f}%")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else
         "docs/tasks/artifacts/eus67/noregress_owned.tsv")
