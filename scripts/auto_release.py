"""Check crates.io and prepare/publish releases. Requires Python 3.11+, git and gh."""

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib
import urllib.request


def run(*args):
    return subprocess.check_output(args, text=True, encoding="utf-8").strip()


def version(value):
    if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", value):
        raise ValueError(f"Expected a stable x.y.z version: {value}")
    return tuple(map(int, value.split(".")))


def latest_version(data):
    candidates = [
        item["num"] for item in data["versions"]
        if not item["yanked"] and re.fullmatch(r"\d+\.\d+\.\d+", item["num"])
    ]
    return max(candidates, key=version)


def fetch_latest():
    request = urllib.request.Request(
        "https://crates.io/api/v1/crates/anydoc",
        headers={"User-Agent": "winmerge-anydoc-updater (https://github.com/namulab/winmerge-anydoc)"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return latest_version(json.load(response))


def read_toml(path):
    return tomllib.loads(Path(path).read_text(encoding="utf-8"))


def locked_anydoc():
    return anydoc_version(read_toml("Cargo.lock"))


def anydoc_version(lock):
    packages = [p for p in lock["package"] if p["name"] == "anydoc"]
    if len(packages) != 1:
        raise ValueError("Expected exactly one anydoc package in Cargo.lock")
    return packages[0]["version"]


def replace_field(text, section, key, value):
    # Preserve formatting and change only the requested TOML section.
    pattern = rf'(?ms)(^\[{re.escape(section)}\]\s*\n)(.*?)(?=^\[|\Z)'

    def replace_section(match):
        body, count = re.subn(
            rf'(?m)^({re.escape(key)}\s*=\s*)"[^"]*"',
            lambda field: f'{field[1]}"{value}"', match[2],
        )
        if count != 1:
            raise ValueError(f"Expected one {section}.{key} string")
        return match[1] + body

    result, count = re.subn(pattern, replace_section, text)
    if count != 1:
        raise ValueError(f"Expected one [{section}] section")
    return result


def update_dependency(latest):
    manifest = Path("Cargo.toml")
    original = manifest.read_text(encoding="utf-8")
    major, minor, patch = version(tomllib.loads(original)["package"]["version"])
    next_version = f"{major}.{minor}.{patch + 1}"
    if run("git", "tag", "--list", f"v{next_version}"):
        raise ValueError(f"Tag v{next_version} already exists")
    updated = replace_field(original, "package", "version", next_version)
    updated = replace_field(updated, "dependencies", "anydoc", f"={latest}")
    lock = Path("Cargo.lock")
    original_lock = lock.read_bytes()
    try:
        manifest.write_text(updated, encoding="utf-8", newline="\n")
        run("cargo", "update", "--package", "anydoc", "--precise", latest)
        if locked_anydoc() != latest:
            raise ValueError("Cargo.lock did not resolve the requested anydoc version")
    except Exception:
        manifest.write_text(original, encoding="utf-8", newline="\n")
        lock.write_bytes(original_lock)
        raise
    return f"v{next_version}"


def releases():
    # --paginate includes old releases and drafts; API/auth errors must fail the run.
    pages = json.loads(run("gh", "api", "--paginate", "--slurp", "repos/{owner}/{repo}/releases"))
    return {item["tag_name"]: item for page in pages for item in page}


def output(**values):
    print(json.dumps(values))
    if os.environ.get("GITHUB_OUTPUT"):
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as stream:
            for key, value in values.items():
                stream.write(f"{key}={str(value).lower() if isinstance(value, bool) else value}\n")


def prepare(tag=None):
    current = read_toml("Cargo.toml")["package"]["version"]
    version(current)
    current_tag = f"v{current}"
    known = releases()
    if tag:
        if tag != current_tag:
            raise ValueError(f"Tag {tag} does not match Cargo.toml version {current}")
        output(tag=tag, release=tag not in known or known[tag]["draft"], changed=False)
        return
    # Resume an interrupted publication before considering another dependency update.
    if run("git", "tag", "--list", current_tag) and (
        current_tag not in known or known[current_tag]["draft"]
    ):
        run("git", "checkout", "--detach", current_tag)
        output(tag=current_tag, release=True, changed=False)
        return
    latest = fetch_latest()
    installed = locked_anydoc()
    if version(latest) <= version(installed):
        output(release=False, changed=False)
        return
    tag = update_dependency(latest)
    print(f"anydoc {installed} -> {latest}; release {tag}")
    output(tag=tag, release=True, changed=True)


def commit(tag):
    version(tag.removeprefix("v"))
    run("git", "config", "user.name", "github-actions[bot]")
    run("git", "config", "user.email", "41898282+github-actions[bot]@users.noreply.github.com")
    run("git", "add", "Cargo.toml", "Cargo.lock", "THIRD_PARTY_NOTICES.md")
    run("git", "commit", "-m", f"Release {tag}: update anydoc to {locked_anydoc()}")
    run("git", "tag", tag)
    # Both refs move together; a concurrent main change fails without overwriting it.
    run("git", "push", "--atomic", "origin", "HEAD:refs/heads/main", f"refs/tags/{tag}")


def release_notes(tag):
    # Read committed history so retries produce the same old/new versions.
    current = anydoc_version(tomllib.loads(run("git", "show", f"{tag}:Cargo.lock")))
    parents = run("git", "rev-list", "--parents", "-n", "1", tag).split()[1:]
    previous = None
    if parents:
        previous = anydoc_version(tomllib.loads(run("git", "show", f"{parents[0]}:Cargo.lock")))
    if previous and previous != current:
        summary = f"ドキュメント変換ライブラリ anydoc を {previous} から {current} に更新しました。"
    else:
        summary = f"ドキュメント変換ライブラリ anydoc {current} を使用しています。"
    return (
        f"{summary}\n\n"
        "詳細は [anydoc の更新履歴](https://github.com/firecrawl/anydoc/releases)をご覧ください。\n"
    )


def publish(tag):
    known = releases()
    if tag in known and not known[tag]["draft"]:
        print(f"{tag} is already published")
        return
    notes = release_notes(tag)
    if tag not in known:
        run("gh", "release", "create", tag, "--draft", "--verify-tag", "--title", tag,
            "--notes", notes)
    run("gh", "release", "upload", tag, f"winmerge-anydoc-{tag}.zip", "--clobber")
    run("gh", "release", "edit", tag, "--notes", notes, "--draft=false")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["check", "prepare", "commit", "publish"])
    parser.add_argument("--tag")
    args = parser.parse_args()
    if args.command == "check":
        print(f"Locked anydoc: {locked_anydoc()}; latest stable: {fetch_latest()}")
    elif args.command == "prepare":
        prepare(args.tag)
    else:
        if not args.tag:
            parser.error("--tag is required for commit/publish")
        {"commit": commit, "publish": publish}[args.command](args.tag)
