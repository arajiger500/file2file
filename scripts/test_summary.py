import json
import os

def generate_summary():
    report_path = "matrix_report.json"
    if not os.path.exists(report_path):
        print(f"Error: {report_path} not found.")
        return

    with open(report_path, "r") as f:
        results = json.load(f)

    total = len(results)
    success = sum(1 for r in results if r["success"])
    failed = total - success

    avg_duration = sum(r["duration_ms"] for r in results) / total if total > 0 else 0

    print("=== File2File Release Readiness Summary ===")
    print(f"Total Conversion Pairs: {total}")
    print(f"Success Rate: {success}/{total} ({(success/total)*100:.1f}%)")
    print(f"Average Duration: {avg_duration:.0f}ms")

    if failed > 0:
        print("\nFailed Pairs:")
        for r in results:
            if not r["success"]:
                print(f"- .{r['input']} -> .{r['output']} (Engine: {r['engine']}) | Error: {r['error']}")

    # Generate an artifact-like markdown summary
    with open("release_readiness.md", "w") as f:
        f.write("# Release Readiness Dashboard\n\n")
        f.write(f"- **Total Tests:** {total}\n")
        f.write(f"- **Passing:** {success}\n")
        f.write(f"- **Failing:** {failed}\n")
        f.write(f"- **Success Rate:** {(success/total)*100:.1f}%\n")
        f.write(f"- **Avg Latency:** {avg_duration:.0f}ms\n\n")

        if failed > 0:
            f.write("## Critical Failures\n\n")
            f.write("| Pair | Engine | Error |\n")
            f.write("|------|--------|-------|\n")
            for r in results:
                if not r["success"]:
                    f.write(f"| .{r['input']} -> .{r['output']} | {r['engine']} | {r['error']} |\n")

if __name__ == "__main__":
    generate_summary()
