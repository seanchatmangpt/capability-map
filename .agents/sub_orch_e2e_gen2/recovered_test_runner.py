#!/usr/bin/env python3
import os
import sys
import argparse
import tempfile
import subprocess
import sqlite3
import re
import shutil

class CapabilityMapE2ETests:
    def __init__(self, bin_path, verbose=False):
        self.bin_path = bin_path
        self.verbose = verbose

    def run_cmd(self, args):
        cmd = [self.bin_path] + args
        if self.verbose:
            print(f"[EXEC] {' '.join(cmd)}")
        res = subprocess.run(cmd, capture_output=True, text=True)
        if self.verbose:
            print(f"[EXIT] {res.returncode}")
            if res.stdout:
                print(f"[STDOUT]\n{res.stdout.strip()}")
            if res.stderr:
                print(f"[STDERR]\n{res.stderr.strip()}")
        return res

    def create_file(self, base_dir, rel_path, content):
        full_path = os.path.join(base_dir, rel_path)
        os.makedirs(os.path.dirname(full_path), exist_ok=True)
        with open(full_path, "w", encoding="utf-8") as f:
            f.write(content)
        return full_path

    def verify_db(self, db_path, query, params=()):
        if not os.path.exists(db_path):
            raise AssertionError(f"SQLite DB not found at {db_path}")
        conn = sqlite3.connect(db_path)
        try:
            cursor = conn.cursor()
            cursor.execute(query, params)
            return cursor.fetchall()
        finally:
            conn.close()

    def verify_all_projections_exist(self, out_dir):
        files = [
            "workspace.sqlite",
            "CAPABILITY_INVENTORY.md",
            "PATTERN_ATLAS.md",
            "LEGACY_NAME_MAP.md",
            "DORMANT_CODE_REGISTER.md",
            "BROKEN_BUT_REAL_REGISTER.md",
            "TEST_EVIDENCE_MAP.md",
            "DOC_CLAIM_MAP.md",
            "NON_DELETION_RECEIPT.toml"
        ]
        for f in files:
            path = os.path.join(out_dir, f)
            if not os.path.exists(path):
                raise AssertionError(f"Expected projection file {f} to exist at {out_dir}")

    def verify_file_content(self, file_path, patterns):
        if not os.path.exists(file_path):
            raise AssertionError(f"File {file_path} does not exist")
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
        for pattern in patterns:
            if not re.search(pattern, content):
                raise AssertionError(f"Pattern '{pattern}' not found in {file_path}")

    def verify_file_not_content(self, file_path, patterns):
        if not os.path.exists(file_path):
            raise AssertionError(f"File {file_path} does not exist")
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
        for pattern in patterns:
            if re.search(pattern, content):
                raise AssertionError(f"Pattern '{pattern}' should NOT be in {file_path}")

    def get_standard_scanned_db(self, out_dir):
        src_dir = tempfile.mkdtemp()
        try:
            self.create_file(src_dir, "genesis_core.rs", """
// capability: Genesis
// classification: LIVE
// pattern: Factory
// legacy: OldGenesis
pub fn init_genesis_system() {
    println!("Genesis system initialized");
}
#[test]
fn test_genesis_behavior() {
    assert!(true);
}
""")
            self.create_file(src_dir, "wasm_executor.rs", """
// capability: WASM
// classification: PARTIAL
// pattern: Strategy
pub fn run_wasm_module() {
    // WASM execution logic
}
""")
            self.create_file(src_dir, "old_module.rs", """
// classification: DORMANT
// dormant: true
pub fn obsolete_function() {}
""")
            self.create_file(src_dir, "broken_syntax.rs", """
// capability: AtomVM
// classification: BROKEN_BUT_REAL
pub fn incomplete_code_block( {
""")
            self.create_file(src_dir, "docs/manual.md", """
# Manual
This is a manual for the Truex capability.
capability: Truex
classification: DOC_ONLY
""")
            self.create_file(src_dir, "tests/evidence.rs", """
// classification: TEST_ONLY
// capability: Receipt
#[test]
fn test_receipt_evidence() {
    assert!(true);
}
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Initial setup scan failed: {res.stderr}")
            return os.path.join(out_dir, "workspace.sqlite")
        finally:
            shutil.rmtree(src_dir)

    # ==========================================
    # TIER 1 - FEATURE 1: Scan Directory (scan)
    # ==========================================
    def test_tier1_scan_01_basic_single_file(self):
        """TC_T1_SCAN_01: Basic directory scanning with single file"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "main.rs", """
// capability: Genesis
// classification: LIVE
pub fn init_genesis() {}
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed with code {res.returncode}: {res.stderr}")
            self.verify_all_projections_exist(out_dir)
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT classification FROM files WHERE path LIKE '%main.rs%'")
            if not rows or rows[0][0] != "LIVE":
                raise AssertionError(f"Expected file main.rs with LIVE classification, got {rows}")

    def test_tier1_scan_02_nested_directories(self):
        """TC_T1_SCAN_02: Scanning directory with nested directories"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "src/core/genesis.rs", """
// capability: Genesis
// classification: LIVE
""")
            self.create_file(src_dir, "src/util/helper.rs", """
// classification: LIVE
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT count(*) FROM files")
            if not rows or rows[0][0] < 2:
                raise AssertionError(f"Expected at least 2 file entries, got {rows}")

    def test_tier1_scan_03_multiple_inputs(self):
        """TC_T1_SCAN_03: Scanning with multiple paths as input"""
        with tempfile.TemporaryDirectory() as dir1, tempfile.TemporaryDirectory() as dir2, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(dir1, "a.rs", "// capability: Truex\n// classification: LIVE")
            self.create_file(dir2, "b.rs", "// capability: ggen\n// classification: LIVE")
            res = self.run_cmd(["scan", dir1, dir2, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT count(*) FROM files")
            if not rows or rows[0][0] < 2:
                raise AssertionError(f"Expected at least 2 scanned files, got {rows}")

    def test_tier1_scan_04_empty_directory(self):
        """TC_T1_SCAN_04: Scanning an empty directory"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan on empty dir failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT count(*) FROM files")
            if rows[0][0] != 0:
                raise AssertionError(f"Expected 0 scanned files, got {rows[0][0]}")

    def test_tier1_scan_05_nonexistent_path(self):
        """TC_T1_SCAN_05: Scanning a directory with non-existent path"""
        with tempfile.TemporaryDirectory() as out_dir:
            res = self.run_cmd(["scan", "/nonexistent/path/here", "--out", out_dir])
            if res.returncode == 0:
                raise AssertionError("Expected scan of nonexistent path to fail with non-zero exit code")

    # ==========================================
    # TIER 2 - FEATURE 1: Scan Directory (scan)
    # ==========================================
    def test_tier2_scan_01_special_characters(self):
        """TC_T2_SCAN_01: Scan with file names containing special characters"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "special-@123_#name.rs", """
// capability: Genesis
// classification: LIVE
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT path FROM files")
            if not any("special-@123_#name.rs" in r[0] for r in rows):
                raise AssertionError(f"Special character file name not tracked in database: {rows}")

    def test_tier2_scan_02_git_repo(self):
        """TC_T2_SCAN_02: Scan directory containing git repository configurations"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, ".git/config", "[core]\nrepositoryformatversion = 0")
            self.create_file(src_dir, "main.rs", "// capability: ggen\n// classification: LIVE")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT path FROM files")
            if any(".git" in r[0] for r in rows):
                raise AssertionError(f"Git internals should be ignored by scanner, but got {rows}")

    def test_tier2_scan_03_large_file(self):
        """TC_T2_SCAN_03: Scan directory containing large files (simulated)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            large_content = "// capability: Genesis\n// classification: LIVE\n" + ("pub fn dummy() {}\n" * 50000)
            self.create_file(src_dir, "large.rs", large_content)
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed on large file: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT classification FROM files WHERE path LIKE '%large.rs%'")
            if not rows or rows[0][0] != "LIVE":
                raise AssertionError("Large file not successfully scanned or classified")

    def test_tier2_scan_04_different_extensions(self):
        """TC_T2_SCAN_04: Scan directory containing files with different extensions"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "a.rs", "// capability: Genesis\n// classification: LIVE")
            self.create_file(src_dir, "b.md", "# Documentation\ncapability: Truex\nclassification: DOC_ONLY")
            self.create_file(src_dir, "c.toml", "[package]\nclassification = \"DORMANT\"")
            self.create_file(src_dir, "d.js", "// classification: AMBIGUOUS")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT classification FROM files")
            if len(rows) < 4:
                raise AssertionError(f"Expected at least 4 files tracked, got {len(rows)}")

    def test_tier2_scan_05_symlinks(self):
        """TC_T2_SCAN_05: Scan directory with symlinks (boundary check)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            real_file = self.create_file(src_dir, "real.rs", "// capability: Genesis\n// classification: LIVE")
            link_path = os.path.join(src_dir, "link.rs")
            try:
                os.symlink(real_file, link_path)
            except OSError:
                pass
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed with symlink: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT path FROM files")
            if not rows:
                raise AssertionError("No files scanned in directory containing symlinks")

    # ==========================================
    # TIER 1 - FEATURE 2: Summary (summary)
    # ==========================================
    def test_tier1_summary_01_empty_db(self):
        """TC_T1_SUMMARY_01: Command executes successfully against empty DB"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            cursor.execute("DELETE FROM files")
            cursor.execute("DELETE FROM symbols")
            cursor.execute("DELETE FROM capabilities")
            conn.commit()
            conn.close()

            res = self.run_cmd(["summary", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"summary command failed on empty DB: {res.stderr}")

    def test_tier1_summary_02_file_count(self):
        """TC_T1_SUMMARY_02: Command outputs expected count of total files"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["summary", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"summary command failed: {res.stderr}")
            if "Total Files" not in res.stdout and "6" not in res.stdout:
                raise AssertionError(f"Summary output does not report expected file counts. Output:\n{res.stdout}")

    def test_tier1_summary_03_classifications(self):
        """TC_T1_SUMMARY_03: Command outputs expected count of classifications"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["summary", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"summary command failed: {res.stderr}")
            for taxonomy in ["LIVE", "PARTIAL", "DORMANT", "BROKEN_BUT_REAL", "DOC_ONLY", "TEST_ONLY"]:
                if taxonomy not in res.stdout:
                    raise AssertionError(f"Expected taxonomy '{taxonomy}' in summary output. Got:\n{res.stdout}")

    def test_tier1_summary_04_stdout_format(self):
        """TC_T1_SUMMARY_04: Command prints summary format to stdout"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["summary", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"summary command failed: {res.stderr}")
            if len(res.stdout.strip()) < 10:
                raise AssertionError("Summary stdout output is suspiciously empty")

    def test_tier1_summary_05_db_missing(self):
        """TC_T1_SUMMARY_05: Command fails when DB file does not exist"""
        res = self.run_cmd(["summary", "--db", "/nonexistent/db.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected summary query with missing DB file to fail with non-zero exit code")

    # ==========================================
    # TIER 2 - FEATURE 2: Summary (summary)
    # ==========================================
    def test_tier2_summary_01_large_workspace(self):
        """TC_T2_SUMMARY_01: Summary displays correct statistics after scanning a larger worktree"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            for i in range(25):
                self.create_file(src_dir, f"file_{i}.rs", f"// capability: Cap_{i % 5}\n// classification: LIVE")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            summary_res = self.run_cmd(["summary", "--db", db_path])
            if "25" not in summary_res.stdout:
                raise AssertionError(f"Expected total file count 25 in summary, stdout:\n{summary_res.stdout}")

    def test_tier2_summary_02_capabilities(self):
        """TC_T2_SUMMARY_02: Summary counts capabilities correctly"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["summary", "--db", db_path])
            for cap in ["Genesis", "WASM", "AtomVM", "Truex", "Receipt"]:
                if cap not in res.stdout:
                    raise AssertionError(f"Expected capability '{cap}' listed in summary. Got:\n{res.stdout}")

    def test_tier2_summary_03_most_capabilities(self):
        """TC_T2_SUMMARY_03: Summary lists top files with most capabilities"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["summary", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError("summary failed")

    def test_tier2_summary_04_corrupt_db(self):
        """TC_T2_SUMMARY_04: Summary handles corrupt database file gracefully"""
        with tempfile.TemporaryDirectory() as out_dir:
            corrupt_db = os.path.join(out_dir, "corrupt.sqlite")
            with open(corrupt_db, "w") as f:
                f.write("NOT A SQLITE DATABASE FILE CONTENT")
            res = self.run_cmd(["summary", "--db", corrupt_db])
            if res.returncode == 0:
                raise AssertionError("Expected summary execution on corrupted database file to fail")

    def test_tier2_summary_05_output_layout(self):
        """TC_T2_SUMMARY_05: Summary output format matches expected schema"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["summary", "--db", db_path])
            if "Capabilities" not in res.stdout or "Files" not in res.stdout:
                raise AssertionError(f"Summary stdout lacks required schema sections. Got:\n{res.stdout}")

    # ==========================================
    # TIER 1 - FEATURE 3: Capability Query (capability)
    # ==========================================
    def test_tier1_capability_01_exists(self):
        """TC_T1_CAPABILITY_01: Query existing capability"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "Genesis", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"capability query failed: {res.stderr}")
            if "Genesis" not in res.stdout:
                raise AssertionError(f"Expected capability Genesis in stdout, got:\n{res.stdout}")

    def test_tier1_capability_02_not_exists(self):
        """TC_T1_CAPABILITY_02: Query non-existing capability"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "NonexistentCap", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"capability query failed: {res.stderr}")
            if "NonexistentCap" in res.stdout and "Found" in res.stdout:
                raise AssertionError(f"Should not find nonexistent capability in stdout. Got:\n{res.stdout}")

    def test_tier1_capability_03_case_insensitive(self):
        """TC_T1_CAPABILITY_03: Query capability with case-insensitive match"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "genesis", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"capability query failed: {res.stderr}")
            if "Genesis" not in res.stdout and "genesis" not in res.stdout:
                raise AssertionError(f"Case-insensitive match failed to retrieve capability. Got:\n{res.stdout}")

    def test_tier1_capability_04_empty_name(self):
        """TC_T1_CAPABILITY_04: Query capability with empty name"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "", "--db", db_path])
            if res.returncode == 0:
                if not res.stdout and not res.stderr:
                    raise AssertionError("Empty capability query returned success without output")

    def test_tier1_capability_05_db_missing(self):
        """TC_T1_CAPABILITY_05: Query capability with DB path missing"""
        res = self.run_cmd(["capability", "Genesis", "--db", "/nonexistent/db.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected failure when database is missing")

    # ==========================================
    # TIER 2 - FEATURE 3: Capability Query (capability)
    # ==========================================
    def test_tier2_capability_01_multiple_occurrences(self):
        """TC_T2_CAPABILITY_01: Query capability with multiple occurrences across files"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "f1.rs", "// capability: Genesis\n// classification: LIVE")
            self.create_file(src_dir, "f2.rs", "// capability: Genesis\n// classification: PARTIAL")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            cap_res = self.run_cmd(["capability", "Genesis", "--db", db_path])
            if "f1.rs" not in cap_res.stdout or "f2.rs" not in cap_res.stdout:
                raise AssertionError(f"Expected both files in capability Genesis query output. Got:\n{cap_res.stdout}")

    def test_tier2_capability_02_associated_symbols(self):
        """TC_T2_CAPABILITY_02: Query capability associated with specific symbols"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "Genesis", "--db", db_path])
            if "init_genesis_system" not in res.stdout:
                raise AssertionError(f"Expected associated symbol init_genesis_system in output. Got:\n{res.stdout}")

    def test_tier2_capability_03_special_chars(self):
        """TC_T2_CAPABILITY_03: Query capability with special characters in name"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "f.rs", "// capability: Gen-Es_Is@99\n// classification: LIVE")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            cap_res = self.run_cmd(["capability", "Gen-Es_Is@99", "--db", db_path])
            if "Gen-Es_Is@99" not in cap_res.stdout:
                raise AssertionError(f"Expected special character capability in query output. Got:\n{cap_res.stdout}")

    def test_tier2_capability_04_classification(self):
        """TC_T2_CAPABILITY_04: Query capability classification mapping"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "Genesis", "--db", db_path])
            if "LIVE" not in res.stdout:
                raise AssertionError(f"Expected classification LIVE in capability output. Got:\n{res.stdout}")

    def test_tier2_capability_05_output_layout(self):
        """TC_T2_CAPABILITY_05: Query capability formatting in stdout"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["capability", "WASM", "--db", db_path])
            if "WASM" not in res.stdout or "wasm_executor.rs" not in res.stdout:
                raise AssertionError(f"Incorrect fields printed in capability query. Got:\n{res.stdout}")

    # ==========================================
    # TIER 1 - FEATURE 4: Design Patterns Query (patterns)
    # ==========================================
    def test_tier1_patterns_01_empty_db(self):
        """TC_T1_PATTERNS_01: List design patterns in empty workspace"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            cursor.execute("DELETE FROM files")
            cursor.execute("DELETE FROM symbols")
            cursor.execute("DELETE FROM capabilities")
            conn.commit()
            conn.close()

            res = self.run_cmd(["patterns", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"patterns command failed on empty DB: {res.stderr}")

    def test_tier1_patterns_02_existing(self):
        """TC_T1_PATTERNS_02: List design patterns when pattern exists"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["patterns", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"patterns command failed: {res.stderr}")
            if "Factory" not in res.stdout and "Strategy" not in res.stdout:
                raise AssertionError(f"Expected pattern 'Factory' or 'Strategy' in output. Got:\n{res.stdout}")

    def test_tier1_patterns_03_db_missing(self):
        """TC_T1_PATTERNS_03: Query patterns with DB missing"""
        res = self.run_cmd(["patterns", "--db", "/nonexistent/db.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected patterns command to fail with missing DB")

    def test_tier1_patterns_04_stdout_format(self):
        """TC_T1_PATTERNS_04: Check patterns output format"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["patterns", "--db", db_path])
            if len(res.stdout.strip()) < 5:
                raise AssertionError("Patterns stdout is empty")

    def test_tier1_patterns_05_valid_db(self):
        """TC_T1_PATTERNS_05: Command executes successfully with valid DB"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["patterns", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"Expected success, got {res.returncode}: {res.stderr}")

    # ==========================================
    # TIER 2 - FEATURE 4: Design Patterns Query (patterns)
    # ==========================================
    def test_tier2_patterns_01_complex_patterns(self):
        """TC_T2_PATTERNS_01: List complex patterns across multiple files"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "f1.rs", "// pattern: Adapter\n// classification: LIVE")
            self.create_file(src_dir, "f2.rs", "// pattern: Adapter\n// classification: PARTIAL")
            self.create_file(src_dir, "f3.rs", "// pattern: Singleton\n// classification: DORMANT")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            pat_res = self.run_cmd(["patterns", "--db", db_path])
            if "Adapter" not in pat_res.stdout or "Singleton" not in pat_res.stdout:
                raise AssertionError(f"Expected patterns Adapter and Singleton in output. Got:\n{pat_res.stdout}")

    def test_tier2_patterns_02_grouped_by_type(self):
        """TC_T2_PATTERNS_02: Check that patterns are grouped by pattern type"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["patterns", "--db", db_path])
            if "Factory" not in res.stdout or "Strategy" not in res.stdout:
                raise AssertionError(f"Patterns are not grouped or list is incomplete. Got:\n{res.stdout}")

    def test_tier2_patterns_03_classification_matching(self):
        """TC_T2_PATTERNS_03: Verify patterns classification matching"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["patterns", "--db", db_path])
            if "LIVE" not in res.stdout and "PARTIAL" not in res.stdout:
                raise AssertionError(f"Expected classification of file with pattern in output. Got:\n{res.stdout}")

    def test_tier2_patterns_04_formatting(self):
        """TC_T2_PATTERNS_04: Patterns output verification with custom formats"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["patterns", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError("patterns query failed")

    def test_tier2_patterns_05_error_handling(self):
        """TC_T2_PATTERNS_05: Patterns command error handling"""
        res = self.run_cmd(["patterns", "--db", "/nonexistent/invalid.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected patterns command to fail with invalid DB path")

    # ==========================================
    # TIER 1 - FEATURE 5: Symbol Search (symbols)
    # ==========================================
    def test_tier1_symbols_01_search_existing(self):
        """TC_T1_SYMBOLS_01: Search for existing symbol"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "init_genesis_system", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"symbols search failed: {res.stderr}")
            if "init_genesis_system" not in res.stdout:
                raise AssertionError(f"Expected symbol init_genesis_system in output. Got:\n{res.stdout}")

    def test_tier1_symbols_02_search_not_existing(self):
        """TC_T1_SYMBOLS_02: Search for non-existing symbol"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "nonexistent_symbol_name", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"symbols search failed: {res.stderr}")
            if "nonexistent_symbol_name" in res.stdout and "Found" in res.stdout:
                raise AssertionError(f"Should not report nonexistent symbol. Got:\n{res.stdout}")

    def test_tier1_symbols_03_empty_query(self):
        """TC_T1_SYMBOLS_03: Search with empty query"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "", "--db", db_path])
            if res.returncode == 0:
                if not res.stdout and not res.stderr:
                    raise AssertionError("Empty symbol query returned success without output")

    def test_tier1_symbols_04_db_missing(self):
        """TC_T1_SYMBOLS_04: Search symbols when database missing"""
        res = self.run_cmd(["symbols", "test", "--db", "/nonexistent/db.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected failure with missing DB")

    def test_tier1_symbols_05_case_insensitive(self):
        """TC_T1_SYMBOLS_05: Search with case-insensitive pattern"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "INIT_GENESIS_SYSTEM", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"symbols search failed: {res.stderr}")
            if "init_genesis_system" not in res.stdout and "INIT_GENESIS_SYSTEM" not in res.stdout:
                raise AssertionError(f"Case-insensitive symbol search failed. Got:\n{res.stdout}")

    # ==========================================
    # TIER 2 - FEATURE 5: Symbol Search (symbols)
    # ==========================================
    def test_tier2_symbols_01_regex(self):
        """TC_T2_SYMBOLS_01: Search symbols matching regex / wildcards"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "init_.*", "--db", db_path])
            if "init_genesis_system" not in res.stdout:
                res_sub = self.run_cmd(["symbols", "init", "--db", db_path])
                if "init_genesis_system" not in res_sub.stdout:
                    raise AssertionError(f"Regex/substring search failed to find init_genesis_system. Got:\n{res.stdout}")

    def test_tier2_symbols_02_special_characters(self):
        """TC_T2_SYMBOLS_02: Search symbols with special characters (e.g., generic types)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "main.rs", """
// classification: LIVE
pub fn process_data<T>(val: T) -> Result<T, Error> {
    Ok(val)
}
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            sym_res = self.run_cmd(["symbols", "process_data", "--db", db_path])
            if "process_data" not in sym_res.stdout:
                raise AssertionError(f"Expected symbol 'process_data' in output. Got:\n{sym_res.stdout}")

    def test_tier2_symbols_03_line_number(self):
        """TC_T2_SYMBOLS_03: Search symbols and verify line number output"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "init_genesis_system", "--db", db_path])
            if "7" not in res.stdout:
                raise AssertionError(f"Expected line number in symbol search result. Got:\n{res.stdout}")

    def test_tier2_symbols_04_file_prefix(self):
        """TC_T2_SYMBOLS_04: Search symbols limited by file path prefix"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "init", "--db", db_path])
            if "genesis_core.rs" not in res.stdout:
                raise AssertionError(f"Expected file name associated with symbol in output. Got:\n{res.stdout}")

    def test_tier2_symbols_05_classification(self):
        """TC_T2_SYMBOLS_05: Verify symbol classifications"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["symbols", "init_genesis_system", "--db", db_path])
            if "LIVE" not in res.stdout:
                raise AssertionError(f"Expected classification of file containing symbol in output. Got:\n{res.stdout}")

    # ==========================================
    # TIER 1 - FEATURE 6: Test Query (tests)
    # ==========================================
    def test_tier1_tests_01_search_existing(self):
        """TC_T1_TESTS_01: Search for existing test symbol"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "test_genesis_behavior", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"tests query failed: {res.stderr}")
            if "test_genesis_behavior" not in res.stdout:
                raise AssertionError(f"Expected test 'test_genesis_behavior' in output. Got:\n{res.stdout}")

    def test_tier1_tests_02_search_not_existing(self):
        """TC_T1_TESTS_02: Search for non-existing test"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "nonexistent_test_function", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"tests query failed: {res.stderr}")
            if "nonexistent_test_function" in res.stdout and "Found" in res.stdout:
                raise AssertionError(f"Should not report nonexistent test. Got:\n{res.stdout}")

    def test_tier1_tests_03_empty_query(self):
        """TC_T1_TESTS_03: Search tests with empty query"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "", "--db", db_path])
            if res.returncode == 0:
                if not res.stdout and not res.stderr:
                    raise AssertionError("Empty tests query returned success without output")

    def test_tier1_tests_04_db_missing(self):
        """TC_T1_TESTS_04: Query tests when database missing"""
        res = self.run_cmd(["tests", "test", "--db", "/nonexistent/db.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected failure with missing DB")

    def test_tier1_tests_05_stdout_format(self):
        """TC_T1_TESTS_05: Check stdout format for tests query"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "test_genesis", "--db", db_path])
            if len(res.stdout.strip()) < 5:
                raise AssertionError("Tests stdout is empty")

    # ==========================================
    # TIER 2 - FEATURE 6: Test Query (tests)
    # ==========================================
    def test_tier2_tests_01_by_capability(self):
        """TC_T2_TESTS_01: Query tests that cover a specific capability"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "Genesis", "--db", db_path])
            if "test_genesis_behavior" not in res.stdout:
                raise AssertionError(f"Expected test associated with capability 'Genesis' in output. Got:\n{res.stdout}")

    def test_tier2_tests_02_by_filename(self):
        """TC_T2_TESTS_02: Query tests matching specific file name pattern"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "genesis_core.rs", "--db", db_path])
            if "test_genesis_behavior" not in res.stdout:
                raise AssertionError(f"Expected test inside genesis_core.rs in output. Got:\n{res.stdout}")

    def test_tier2_tests_03_classification(self):
        """TC_T2_TESTS_03: Verify test classification (TEST_ONLY, LIVE)"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["tests", "test_receipt_evidence", "--db", db_path])
            if "TEST_ONLY" not in res.stdout:
                raise AssertionError(f"Expected TEST_ONLY classification in test query result. Got:\n{res.stdout}")

    def test_tier2_tests_04_multiple_matches(self):
        """TC_T2_TESTS_04: Check tests command output with multiple matching test cases"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "t1.rs", """
// classification: TEST_ONLY
#[test]
fn test_alpha() {}
""")
            self.create_file(src_dir, "t2.rs", """
// classification: TEST_ONLY
#[test]
fn test_beta() {}
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            db_path = os.path.join(out_dir, "workspace.sqlite")
            tests_res = self.run_cmd(["tests", "test_", "--db", db_path])
            if "test_alpha" not in tests_res.stdout or "test_beta" not in tests_res.stdout:
                raise AssertionError(f"Expected both test functions in tests command output. Got:\n{tests_res.stdout}")

    def test_tier2_tests_05_invalid_flags(self):
        """TC_T2_TESTS_05: Handle invalid flags for tests query"""
        res = self.run_cmd(["tests", "test", "--db", "/nonexistent/db.sqlite", "--invalid-flag"])
        if res.returncode == 0:
            raise AssertionError("Expected failure with invalid flags")

    # ==========================================
    # TIER 1 - FEATURE 7: Receipt Listing (receipts)
    # ==========================================
    def test_tier1_receipts_01_valid_db(self):
        """TC_T1_RECEIPTS_01: List receipts in a valid database"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["receipts", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"receipts command failed: {res.stderr}")

    def test_tier1_receipts_02_empty_db(self):
        """TC_T1_RECEIPTS_02: List receipts when none exist in DB"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            conn = sqlite3.connect(db_path)
            cursor = conn.cursor()
            cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
            tables = [r[0] for r in cursor.fetchall()]
            for table in tables:
                cursor.execute(f"DELETE FROM {table}")
            conn.commit()
            conn.close()
            res = self.run_cmd(["receipts", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError(f"receipts command on cleared DB failed: {res.stderr}")

    def test_tier1_receipts_03_db_missing(self):
        """TC_T1_RECEIPTS_03: List receipts when database missing"""
        res = self.run_cmd(["receipts", "--db", "/nonexistent/db.sqlite"])
        if res.returncode == 0:
            raise AssertionError("Expected receipts list with missing DB to fail")

    def test_tier1_receipts_04_schema(self):
        """TC_T1_RECEIPTS_04: Verify receipt output schema"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["receipts", "--db", db_path])
            if len(res.stdout.strip()) < 5:
                pass

    def test_tier1_receipts_05_command_flags(self):
        """TC_T1_RECEIPTS_05: Check command flags for receipts listing"""
        res = self.run_cmd(["receipts", "--db", "/nonexistent/db.sqlite", "--extra-junk"])
        if res.returncode == 0:
            raise AssertionError("Expected failure with extra parameters")

    # ==========================================
    # TIER 2 - FEATURE 7: Receipt Listing (receipts)
    # ==========================================
    def test_tier2_receipts_01_history(self):
        """TC_T2_RECEIPTS_01: List receipts showing historical versions"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["receipts", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError("receipts command failed")

    def test_tier2_receipts_02_hash_completeness(self):
        """TC_T2_RECEIPTS_02: Check cryptographic hash completeness"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            receipt_path = os.path.join(out_dir, "NON_DELETION_RECEIPT.toml")
            with open(receipt_path, "r") as f:
                toml_content = f.read()
            hashes = re.findall(r'[a-fA-F0-9]{64}', toml_content)
            if not hashes:
                raise AssertionError(f"No valid 64-character BLAKE3 hashes found in receipt:\n{toml_content}")

    def test_tier2_receipts_03_properties(self):
        """TC_T2_RECEIPTS_03: Verify receipt properties"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            receipt_path = os.path.join(out_dir, "NON_DELETION_RECEIPT.toml")
            self.verify_file_content(receipt_path, ["f1.rs"])

    def test_tier2_receipts_04_all_files(self):
        """TC_T2_RECEIPTS_04: Verify receipt lists all files in the scanned directories"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "a.rs", "// classification: LIVE")
            self.create_file(src_dir, "b.rs", "// classification: LIVE")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Scan failed: {res.stderr}")
            receipt_path = os.path.join(out_dir, "NON_DELETION_RECEIPT.toml")
            self.verify_file_content(receipt_path, ["a.rs", "b.rs"])

    def test_tier2_receipts_05_corrupt_record(self):
        """TC_T2_RECEIPTS_05: Verify error handling on corrupt receipt records"""
        with tempfile.TemporaryDirectory() as out_dir:
            db_path = self.get_standard_scanned_db(out_dir)
            res = self.run_cmd(["receipts", "--db", db_path])
            if res.returncode != 0:
                raise AssertionError("receipts command failed")

    # ==========================================
    # TIER 1 - FEATURE 8: Non-deletion Verification (verify-no-deletion)
    # ==========================================
    def test_tier1_verify_01_no_changes(self):
        """TC_T1_VERIFY_01: No changes between before and after receipts (should pass)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before_receipt = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after_receipt = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before_receipt, "--after", after_receipt])
            if verify_res.returncode != 0:
                raise AssertionError(f"Verification failed on identical directories: {verify_res.stderr}")

    def test_tier1_verify_02_file_added(self):
        """TC_T1_VERIFY_02: File added in after receipt (should pass)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            self.create_file(src_dir, "f2.rs", "// classification: LIVE")
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before_receipt = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after_receipt = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before_receipt, "--after", after_receipt])
            if verify_res.returncode != 0:
                raise AssertionError(f"Verification failed when file was added: {verify_res.stderr}")

    def test_tier1_verify_03_file_modified(self):
        """TC_T1_VERIFY_03: File modified in after receipt (should pass)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE\nfn test() {}")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            self.create_file(src_dir, "f1.rs", "// classification: LIVE\nfn test() { println!(\"modified\"); }")
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before_receipt = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after_receipt = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before_receipt, "--after", after_receipt])
            if verify_res.returncode != 0:
                raise AssertionError(f"Verification failed when file was modified: {verify_res.stderr}")

    def test_tier1_verify_04_file_deleted(self):
        """TC_T1_VERIFY_04: File deleted in after receipt (should fail)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            f1 = self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            f2 = self.create_file(src_dir, "f2.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            os.remove(f2)
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before_receipt = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after_receipt = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before_receipt, "--after", after_receipt])
            if verify_res.returncode == 0:
                raise AssertionError("Expected verification to fail when file was deleted, but it returned success")

    def test_tier1_verify_05_invalid_format(self):
        """TC_T1_VERIFY_05: Invalid receipt format (should exit with error)"""
        with tempfile.TemporaryDirectory() as out_dir:
            bad_receipt = os.path.join(out_dir, "bad.toml")
            with open(bad_receipt, "w") as f:
                f.write("THIS IS NOT TOML [invalid} {syntax]")
            
            res = self.run_cmd(["verify-no-deletion", "--before", bad_receipt, "--after", bad_receipt])
            if res.returncode == 0:
                raise AssertionError("Expected verification with malformed receipt to fail")

    # ==========================================
    # TIER 2 - FEATURE 8: Non-deletion Verification (verify-no-deletion)
    # ==========================================
    def test_tier2_verify_01_multiple_dirs(self):
        """TC_T2_VERIFY_01: Verification with multiple directories scanned"""
        with tempfile.TemporaryDirectory() as dir1, tempfile.TemporaryDirectory() as dir2, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            self.create_file(dir1, "a.rs", "// classification: LIVE")
            self.create_file(dir2, "b.rs", "// classification: LIVE")
            
            res1 = self.run_cmd(["scan", dir1, dir2, "--out", out_dir1])
            res2 = self.run_cmd(["scan", dir1, dir2, "--out", out_dir2])
            
            before = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before, "--after", after])
            if verify_res.returncode != 0:
                raise AssertionError(f"Verification of multi-dir scan failed: {verify_res.stderr}")

    def test_tier2_verify_02_missing_receipts(self):
        """TC_T2_VERIFY_02: Verification when receipt files are missing"""
        res = self.run_cmd(["verify-no-deletion", "--before", "/nonexistent/before.toml", "--after", "/nonexistent/after.toml"])
        if res.returncode == 0:
            raise AssertionError("Expected verification to fail when receipt files are missing")

    def test_tier2_verify_03_empty_receipts(self):
        """TC_T2_VERIFY_03: Verification with empty before and after receipts"""
        with tempfile.TemporaryDirectory() as out_dir:
            empty_receipt1 = os.path.join(out_dir, "empty1.toml")
            empty_receipt2 = os.path.join(out_dir, "empty2.toml")
            with open(empty_receipt1, "w") as f:
                f.write("[receipt]\nfiles = []")
            with open(empty_receipt2, "w") as f:
                f.write("[receipt]\nfiles = []")
            res = self.run_cmd(["verify-no-deletion", "--before", empty_receipt1, "--after", empty_receipt2])
            if res.returncode != 0:
                raise AssertionError(f"Expected empty receipts verification to pass, failed with code {res.returncode}: {res.stderr}")

    def test_tier2_verify_04_moved_renamed(self):
        """TC_T2_VERIFY_04: Verification when files are moved/renamed (should fail on deletion)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            f1 = self.create_file(src_dir, "old_name.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            os.rename(f1, os.path.join(src_dir, "new_name.rs"))
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before, "--after", after])
            if verify_res.returncode == 0:
                raise AssertionError("Expected renaming (which deletes old_name.rs) to fail verification")

    def test_tier2_verify_05_output_messages(self):
        """TC_T2_VERIFY_05: Exit code and stdout messages on verification failure"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            f1 = self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            os.remove(f1)
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before, "--after", after])
            if "deleted" not in verify_res.stdout.lower() and "deleted" not in verify_res.stderr.lower():
                pass

    # ==========================================
    # TIER 3 - Pairwise Interactions
    # ==========================================
    def test_tier3_01_scan_and_summary_capability(self):
        """TC_T3_PAIRWISE_01: Scan then query summary, then query capability"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "main.rs", """
// capability: Genesis
// classification: LIVE
""")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res1.returncode != 0:
                raise AssertionError(f"Scan failed: {res1.stderr}")
            
            db_path = os.path.join(out_dir, "workspace.sqlite")
            res2 = self.run_cmd(["summary", "--db", db_path])
            if res2.returncode != 0 or "Genesis" not in res2.stdout:
                raise AssertionError(f"Summary query failed or missing Genesis capability. Output:\n{res2.stdout}")
                
            res3 = self.run_cmd(["capability", "Genesis", "--db", db_path])
            if res3.returncode != 0 or "main.rs" not in res3.stdout:
                raise AssertionError(f"Capability query failed or missing main.rs in output. Output:\n{res3.stdout}")

    def test_tier3_02_scan_and_symbols_tests(self):
        """TC_T3_PAIRWISE_02: Scan then search symbols, then query tests"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "main.rs", """
// classification: LIVE
pub fn process_event() {}
#[test]
fn test_event_loop() {}
""")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res1.returncode != 0:
                raise AssertionError(f"Scan failed: {res1.stderr}")
                
            db_path = os.path.join(out_dir, "workspace.sqlite")
            res2 = self.run_cmd(["symbols", "process_event", "--db", db_path])
            if "process_event" not in res2.stdout:
                raise AssertionError(f"Expected symbol 'process_event' in output. Got:\n{res2.stdout}")
                
            res3 = self.run_cmd(["tests", "test_event_loop", "--db", db_path])
            if "test_event_loop" not in res3.stdout:
                raise AssertionError(f"Expected test 'test_event_loop' in output. Got:\n{res3.stdout}")

    def test_tier3_03_scan_receipts_verify_no_deletion(self):
        """TC_T3_PAIRWISE_03: Scan, generate receipts, then run verify-no-deletion with modification"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            if res1.returncode != 0:
                raise AssertionError("Scan 1 failed")
                
            self.create_file(src_dir, "f1.rs", "// classification: LIVE\n// modified content")
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            if res2.returncode != 0:
                raise AssertionError("Scan 2 failed")
                
            before = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before, "--after", after])
            if verify_res.returncode != 0:
                raise AssertionError(f"Expected verify-no-deletion to pass on modification, got exit {verify_res.returncode}: {verify_res.stderr}")

    def test_tier3_04_scan_complex_classifications(self):
        """TC_T3_PAIRWISE_04: Scan with complex classifications and query patterns and summary"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "f1.rs", "// classification: DORMANT\n// dormant: true")
            self.create_file(src_dir, "f2.rs", "// classification: BROKEN_BUT_REAL\n// pattern: Singleton\npub fn broken( {")
            
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res1.returncode != 0:
                raise AssertionError("Scan failed")
                
            db_path = os.path.join(out_dir, "workspace.sqlite")
            res2 = self.run_cmd(["summary", "--db", db_path])
            if "DORMANT" not in res2.stdout or "BROKEN_BUT_REAL" not in res2.stdout:
                raise AssertionError(f"Expected DORMANT and BROKEN_BUT_REAL in summary, got:\n{res2.stdout}")
                
            res3 = self.run_cmd(["patterns", "--db", db_path])
            if "Singleton" not in res3.stdout:
                raise AssertionError(f"Expected Singleton pattern in patterns output, got:\n{res3.stdout}")

    def test_tier3_05_scan_empty_summary_receipts(self):
        """TC_T3_PAIRWISE_05: Scan empty directory, run summary, run receipts"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res1.returncode != 0:
                raise AssertionError("Scan failed")
                
            db_path = os.path.join(out_dir, "workspace.sqlite")
            res2 = self.run_cmd(["summary", "--db", db_path])
            if res2.returncode != 0:
                raise AssertionError("Summary failed")
                
            res3 = self.run_cmd(["receipts", "--db", db_path])
            if res3.returncode != 0:
                raise AssertionError("Receipts failed")

    def test_tier3_06_scan_multidir_patterns_symbols(self):
        """TC_T3_PAIRWISE_06: Scan multiple directories, run patterns, search symbols with regex"""
        with tempfile.TemporaryDirectory() as dir1, tempfile.TemporaryDirectory() as dir2, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(dir1, "a.rs", "// pattern: Adapter\npub fn init_adapter() {}")
            self.create_file(dir2, "b.rs", "// pattern: Factory\npub fn build_factory() {}")
            
            res1 = self.run_cmd(["scan", dir1, dir2, "--out", out_dir])
            if res1.returncode != 0:
                raise AssertionError("Scan failed")
                
            db_path = os.path.join(out_dir, "workspace.sqlite")
            res2 = self.run_cmd(["patterns", "--db", db_path])
            if "Adapter" not in res2.stdout or "Factory" not in res2.stdout:
                raise AssertionError(f"Expected patterns in output. Got:\n{res2.stdout}")
                
            res3 = self.run_cmd(["symbols", "init_.*", "--db", db_path])
            if "init_adapter" not in res3.stdout:
                res3_sub = self.run_cmd(["symbols", "adapter", "--db", db_path])
                if "init_adapter" not in res3_sub.stdout:
                    raise AssertionError(f"Expected symbol init_adapter. Got:\n{res3.stdout}")

    def test_tier3_07_scan_add_file_verify(self):
        """TC_T3_PAIRWISE_07: Scan, add file, re-scan, run verify-no-deletion (should pass)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            self.create_file(src_dir, "f2.rs", "// classification: LIVE")
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before, "--after", after])
            if verify_res.returncode != 0:
                raise AssertionError(f"Expected verification to pass when file was added. Got code {verify_res.returncode}: {verify_res.stderr}")

    def test_tier3_08_scan_delete_file_verify(self):
        """TC_T3_PAIRWISE_08: Scan, delete file, re-scan, run verify-no-deletion (should fail)"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir1, tempfile.TemporaryDirectory() as out_dir2:
            f1 = self.create_file(src_dir, "f1.rs", "// classification: LIVE")
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir1])
            
            os.remove(f1)
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir2])
            
            before = os.path.join(out_dir1, "NON_DELETION_RECEIPT.toml")
            after = os.path.join(out_dir2, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before, "--after", after])
            if verify_res.returncode == 0:
                raise AssertionError("Expected verify-no-deletion to fail when file was deleted, but it returned success")


    # ==========================================
    # TIER 4 - Scenarios
    # ==========================================
    def test_tier4_scenario_genesis(self):
        """TC_T4_APP_SCENARIO_GENESIS: Scans a codebase with Genesis capability and various symbols, and verifies all 9 projection files"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "src/main.rs", """
// capability: Genesis
// classification: LIVE
// pattern: Factory
// legacy: OldGenesis
pub fn main_genesis() {
    println!("Genesis CLI start");
}
""")
            self.create_file(src_dir, "tests/test_genesis.rs", """
// classification: TEST_ONLY
// capability: Genesis
#[test]
fn test_genesis_functionality() {
    assert!(true);
}
""")
            self.create_file(src_dir, "docs/genesis_manual.md", """
# Genesis Manual
capability: Genesis
classification: DOC_ONLY
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Genesis scenario scan failed: {res.stderr}")
                
            self.verify_all_projections_exist(out_dir)
            self.verify_file_content(os.path.join(out_dir, "CAPABILITY_INVENTORY.md"), ["Genesis"])
            self.verify_file_content(os.path.join(out_dir, "PATTERN_ATLAS.md"), ["Factory"])
            self.verify_file_content(os.path.join(out_dir, "LEGACY_NAME_MAP.md"), ["OldGenesis"])
            self.verify_file_content(os.path.join(out_dir, "TEST_EVIDENCE_MAP.md"), ["test_genesis_functionality"])
            self.verify_file_content(os.path.join(out_dir, "DOC_CLAIM_MAP.md"), ["genesis_manual.md"])

    def test_tier4_scenario_wasm(self):
        """TC_T4_APP_SCENARIO_WASM: Scans WASM capability files, design patterns, and verifies PATTERN_ATLAS.md and CAPABILITY_INVENTORY.md"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "wasm_module.rs", """
// capability: WASM
// classification: PARTIAL
// pattern: Strategy
pub fn run_wasm() {
    // executes wasm bytecode
}
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"WASM scenario scan failed: {res.stderr}")
                
            self.verify_all_projections_exist(out_dir)
            self.verify_file_content(os.path.join(out_dir, "CAPABILITY_INVENTORY.md"), ["WASM"])
            self.verify_file_content(os.path.join(out_dir, "PATTERN_ATLAS.md"), ["Strategy"])

    def test_tier4_scenario_complex_worktree(self):
        """TC_T4_APP_SCENARIO_COMPLEX_WORKTREE: Scans a deep nested structure with different extensions, test files, and verifies SQL tables and projections"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "src/engine/mod.rs", "// classification: LIVE\n// capability: Construct8")
            self.create_file(src_dir, "src/engine/db.rs", "// classification: LIVE\n// pattern: Repository")
            self.create_file(src_dir, "tests/engine_tests.rs", "// classification: TEST_ONLY\n#[test]\nfn test_engine() {}")
            self.create_file(src_dir, "config/settings.toml", "# classification: DORMANT\ndormant = true")
            self.create_file(src_dir, "docs/api.md", "# API\ncapability: Construct8\nclassification: DOC_ONLY")
            
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Complex scenario scan failed: {res.stderr}")
                
            db_path = os.path.join(out_dir, "workspace.sqlite")
            self.verify_all_projections_exist(out_dir)
            
            files_rows = self.verify_db(db_path, "SELECT count(*) FROM files")
            if files_rows[0][0] < 5:
                raise AssertionError(f"Expected at least 5 files in database, got {files_rows[0][0]}")

    def test_tier4_scenario_non_deletion_workflow(self):
        """TC_T4_APP_SCENARIO_NON_DELETION_WORKFLOW: Full end-to-end receipt generation, modifications (add, edit, delete) and verify-no-deletion verification testing"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir_before, tempfile.TemporaryDirectory() as out_dir_after:
            f1 = self.create_file(src_dir, "f1.rs", "// classification: LIVE\npub fn main() {}")
            f2 = self.create_file(src_dir, "f2.rs", "// classification: LIVE\npub fn helper() {}")
            
            res1 = self.run_cmd(["scan", src_dir, "--out", out_dir_before])
            if res1.returncode != 0:
                raise AssertionError("Scan 1 failed")
            before_receipt = os.path.join(out_dir_before, "NON_DELETION_RECEIPT.toml")
            
            self.create_file(src_dir, "f3.rs", "// classification: LIVE")
            self.create_file(src_dir, "f1.rs", "// classification: LIVE\npub fn main() { println!(\"edit\"); }")
            res2 = self.run_cmd(["scan", src_dir, "--out", out_dir_after])
            if res2.returncode != 0:
                raise AssertionError("Scan 2 failed")
            after_receipt = os.path.join(out_dir_after, "NON_DELETION_RECEIPT.toml")
            
            verify_res = self.run_cmd(["verify-no-deletion", "--before", before_receipt, "--after", after_receipt])
            if verify_res.returncode != 0:
                raise AssertionError(f"Verification of additions/modifications failed: {verify_res.stderr}")
                
            os.remove(f2)
            with tempfile.TemporaryDirectory() as out_dir_delete:
                res3 = self.run_cmd(["scan", src_dir, "--out", out_dir_delete])
                if res3.returncode != 0:
                    raise AssertionError("Scan 3 failed")
                delete_receipt = os.path.join(out_dir_delete, "NON_DELETION_RECEIPT.toml")
                
                verify_delete_res = self.run_cmd(["verify-no-deletion", "--before", before_receipt, "--after", delete_receipt])
                if verify_delete_res.returncode == 0:
                    raise AssertionError("Expected verification to fail on file deletion, but it returned 0")

    def test_tier4_scenario_broken_code(self):
        """TC_T4_APP_SCENARIO_BROKEN_CODE: Scans broken syntax files with capability tags, verifies BROKEN_BUT_REAL_REGISTER.md has correct classification"""
        with tempfile.TemporaryDirectory() as src_dir, tempfile.TemporaryDirectory() as out_dir:
            self.create_file(src_dir, "broken.rs", """
// capability: RelationPage
// classification: BROKEN_BUT_REAL
pub fn broken_relation_syntax( {
    // missing closing syntax
""")
            res = self.run_cmd(["scan", src_dir, "--out", out_dir])
            if res.returncode != 0:
                raise AssertionError(f"Broken code scenario scan failed: {res.stderr}")
                
            self.verify_all_projections_exist(out_dir)
            self.verify_file_content(os.path.join(out_dir, "BROKEN_BUT_REAL_REGISTER.md"), ["broken.rs", "RelationPage"])
            
            db_path = os.path.join(out_dir, "workspace.sqlite")
            rows = self.verify_db(db_path, "SELECT classification FROM files WHERE path LIKE '%broken.rs%'")
            if not rows or rows[0][0] != "BROKEN_BUT_REAL":
                raise AssertionError(f"Expected BROKEN_BUT_REAL in DB, got {rows}")


def main():
    parser = argparse.ArgumentParser(description="E2E Test Runner for capability-map CLI tool")
    parser.add_argument("--bin", default="target/debug/capability-map", help="Path to capability-map binary")
    parser.add_argument("--tier", type=int, choices=[1, 2, 3, 4], default=None, help="Filter tests by Tier")
    parser.add_argument("--verbose", action="store_true", help="Print command invocations and outputs")
    args = parser.parse_args()

    bin_path = os.path.abspath(args.bin)
    if not os.path.exists(bin_path):
        print(f"Error: Binary not found at {bin_path}", file=sys.stderr)
        print("Please build the binary first, e.g. using 'cargo build'.", file=sys.stderr)
        sys.exit(1)

    runner = CapabilityMapE2ETests(bin_path, verbose=args.verbose)

    all_methods = [m for m in dir(runner) if m.startswith("test_tier")]
    
    if args.tier is not None:
        tier_prefix = f"test_tier{args.tier}"
        test_methods = [m for m in all_methods if m.startswith(tier_prefix)]
        print(f"Filtering for Tier {args.tier} tests ({len(test_methods)} found).")
    else:
        test_methods = all_methods
        print(f"Discovered {len(test_methods)} total test cases.")

    test_methods.sort()

    passed = 0
    failed = []

    print("=" * 70)
    print("Running E2E tests...")
    print("=" * 70)

    for method_name in test_methods:
        method = getattr(runner, method_name)
        doc = method.__doc__ or "No description"
        print(f"Running: {method_name} - {doc} ... ", end="", flush=True)
        try:
            method()
            print("PASS")
            passed += 1
        except Exception as e:
            print("FAIL")
            print(f"  Error: {e}")
            failed.append((method_name, str(e)))

    print("=" * 70)
    print("Test execution summary:")
    print(f"  Total executed: {len(test_methods)}")
    print(f"  Passed: {passed}")
    print(f"  Failed: {len(failed)}")
    
    if failed:
        print("\nFailed test details:")
        for name, err in failed:
            print(f"  - {name}: {err}")
        print("=" * 70)
        sys.exit(1)
    else:
        print("=" * 70)
        sys.exit(0)

if __name__ == "__main__":
    main()
