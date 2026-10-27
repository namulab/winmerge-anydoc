from pathlib import Path
import subprocess
import unittest
import uuid
from unittest.mock import patch

import auto_release as release


class ReleaseTests(unittest.TestCase):
    def test_latest_stable_not_yanked_numeric_order(self):
        data = {"versions": [
            {"num": "0.1.9", "yanked": False},
            {"num": "0.1.10", "yanked": False},
            {"num": "0.2.0-rc.1", "yanked": False},
            {"num": "1.0.0", "yanked": True},
        ]}
        self.assertEqual(release.latest_version(data), "0.1.10")

    def test_no_stable_release_fails(self):
        with self.assertRaises(ValueError):
            release.latest_version({"versions": []})

    def test_replace_only_intended_section(self):
        original = '[package]\nversion = "0.1.1"\n\n[dependencies]\nanydoc = "0.1.9"\n\n[dependencies.windows]\nversion = "0.62.2"\n'
        updated = release.replace_field(original, "package", "version", "0.1.2")
        self.assertIn('version = "0.1.2"', updated)
        self.assertIn('version = "0.62.2"', updated)
        self.assertEqual(release.replace_field(updated, "dependencies", "anydoc", "=0.2.0").count('anydoc = "=0.2.0"'), 1)

    def test_missing_field_fails(self):
        with self.assertRaises(ValueError):
            release.replace_field('[package]\nname = "example"\n', "package", "version", "0.1.2")

    @patch.object(release, "output")
    @patch.object(release, "fetch_latest", return_value="0.1.10")
    @patch.object(release, "locked_anydoc", return_value="0.1.10")
    @patch.object(release, "run", return_value="v0.1.1")
    @patch.object(release, "releases", return_value={"v0.1.1": {"draft": False}})
    @patch.object(release, "read_toml", return_value={"package": {"version": "0.1.1"}})
    def test_unchanged_or_older_does_not_release(self, manifest, known, run, locked, latest, output):
        for candidate in ("0.1.10", "0.1.9"):
            latest.return_value = candidate
            release.prepare()
            output.assert_called_with(release=False, changed=False)

    @patch.object(release, "output")
    @patch.object(release, "fetch_latest")
    @patch.object(release, "run", return_value="v0.1.2")
    @patch.object(release, "releases")
    @patch.object(release, "read_toml", return_value={"package": {"version": "0.1.2"}})
    def test_missing_or_draft_release_resumes_exact_tag(self, manifest, known, run, latest, output):
        for state in ({}, {"v0.1.2": {"draft": True}}):
            known.return_value = state
            release.prepare()
            run.assert_called_with("git", "checkout", "--detach", "v0.1.2")
            output.assert_called_with(tag="v0.1.2", release=True, changed=False)
        latest.assert_not_called()

    @patch.object(release, "run")
    @patch.object(release, "release_notes", return_value="release notes")
    @patch.object(release, "releases", return_value={})
    def test_upload_precedes_publication(self, known, notes, run):
        release.publish("v0.1.2")
        calls = [c.args for c in run.call_args_list]
        self.assertIn("--draft", calls[0])
        self.assertEqual(calls[1][1:3], ("release", "upload"))
        self.assertNotIn("--generate-notes", calls[0])
        self.assertEqual(calls[2], ("gh", "release", "edit", "v0.1.2", "--notes", "release notes", "--draft=false"))

    @patch.object(release, "run")
    @patch.object(release, "releases", return_value={"v0.1.2": {"draft": False}})
    def test_published_release_is_untouched(self, known, run):
        release.publish("v0.1.2")
        run.assert_not_called()

    @patch.object(release, "output")
    @patch.object(release, "update_dependency", return_value="v0.1.2")
    @patch.object(release, "fetch_latest", return_value="0.2.4")
    @patch.object(release, "locked_anydoc", return_value="0.1.9")
    @patch.object(release, "run", return_value="v0.1.1")
    @patch.object(release, "releases", return_value={"v0.1.1": {"draft": False}})
    @patch.object(release, "read_toml", return_value={"package": {"version": "0.1.1"}})
    def test_new_upstream_minor_prepares_one_patch_release(self, manifest, known, run, locked, latest, update, output):
        release.prepare()
        update.assert_called_once_with("0.2.4")
        output.assert_called_once_with(tag="v0.1.2", release=True, changed=True)

    @patch.object(release, "run", side_effect=subprocess.CalledProcessError(1, "gh"))
    @patch.object(release, "release_notes", return_value="release notes")
    @patch.object(release, "releases", return_value={"v0.1.2": {"draft": True}})
    def test_failed_upload_never_publishes(self, known, notes, run):
        with self.assertRaises(subprocess.CalledProcessError):
            release.publish("v0.1.2")
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args[1:3], ("release", "upload"))

    @patch.object(release, "run")
    def test_notes_use_tag_history_for_update_and_retry(self, run):
        current = '[[package]]\nname = "anydoc"\nversion = "0.2.4"\n'
        previous = current.replace("0.2.4", "0.1.9")
        run.side_effect = [current, "release-sha parent-sha", previous] * 2
        expected = (
            "ドキュメント変換ライブラリ anydoc を 0.1.9 から 0.2.4 に更新しました。\n\n"
            "詳細は [anydoc の更新履歴](https://github.com/firecrawl/anydoc/releases)をご覧ください。\n"
        )
        self.assertEqual(release.release_notes("v0.1.2"), expected)
        self.assertEqual(release.release_notes("v0.1.2"), expected)
        run.assert_called_with("git", "show", "parent-sha:Cargo.lock")

    @patch.object(release, "run")
    def test_notes_do_not_claim_update_when_dependency_unchanged(self, run):
        lock = '[[package]]\nname = "anydoc"\nversion = "0.1.9"\n'
        for history in ([lock, "root-sha"], [lock, "release-sha parent-sha", lock]):
            run.side_effect = history
            notes = release.release_notes("v0.1.1")
            self.assertIn("anydoc 0.1.9 を使用しています。", notes)
            self.assertNotIn("更新しました", notes)

    @patch.object(release, "run")
    @patch.object(release, "release_notes", return_value="updated Japanese notes")
    @patch.object(release, "releases", return_value={"v0.1.2": {"draft": True}})
    def test_resuming_draft_updates_notes_before_publishing(self, known, notes, run):
        release.publish("v0.1.2")
        self.assertEqual(run.call_count, 2)
        run.assert_called_with("gh", "release", "edit", "v0.1.2", "--notes", "updated Japanese notes", "--draft=false")

    def test_patch_bump_and_rollback_on_cargo_failure(self):
        original = '[package]\nversion = "0.1.1"\n[dependencies]\nanydoc = "0.1.9"\n'
        temporary_root = Path(__file__).resolve().parents[1] / "target"
        temporary_root.mkdir(exist_ok=True)
        directory = temporary_root / f"release-test-{uuid.uuid4()}"
        directory.mkdir()
        self.addCleanup(directory.rmdir)
        self.addCleanup((directory / "Cargo.lock").unlink, missing_ok=True)
        self.addCleanup((directory / "Cargo.toml").unlink, missing_ok=True)
        (directory / "Cargo.toml").write_text(original, encoding="utf-8")
        (directory / "Cargo.lock").write_bytes(b"original lock")
        with (
            patch.object(release, "Path", side_effect=lambda p: directory / p),
            patch.object(release, "run") as run,
            patch.object(release, "locked_anydoc", return_value="0.1.10"),
        ):
            run.return_value = ""
            self.assertEqual(release.update_dependency("0.1.10"), "v0.1.2")
            self.assertIn('anydoc = "=0.1.10"', (directory / "Cargo.toml").read_text())
            (directory / "Cargo.toml").write_text(original, encoding="utf-8")
            run.side_effect = ["", subprocess.CalledProcessError(1, "cargo")]
            with self.assertRaises(subprocess.CalledProcessError):
                release.update_dependency("0.1.10")
            self.assertEqual((directory / "Cargo.toml").read_text(), original)
            self.assertEqual((directory / "Cargo.lock").read_bytes(), b"original lock")


if __name__ == "__main__":
    unittest.main()
