from ksim_ci.bump import main


def test_main_branch_bump_writes_version_and_prints(tmp_path, capsys):
    cargo_toml = tmp_path / "Cargo.toml"
    cargo_toml.write_text("""[workspace.package]
version = "0.1.0"
name = "ksim"
""")

    result = main(
        [
            "--cargo-toml",
            str(cargo_toml),
            "--branch",
            "main",
            "--date",
            "2026-09-29",
        ]
    )

    assert result == 0
    captured = capsys.readouterr()
    assert captured.out.strip() == "0.1.0-nightly.20260929"
    assert "0.1.0-nightly.20260929" in cargo_toml.read_text()


def test_release_branch_bump_increments_patch(tmp_path, capsys):
    cargo_toml = tmp_path / "Cargo.toml"
    cargo_toml.write_text("""[workspace.package]
version = "0.1.0"
name = "ksim"
""")

    result = main(
        [
            "--cargo-toml",
            str(cargo_toml),
            "--branch",
            "release/0.1",
            "--date",
            "2026-09-29",
        ]
    )

    assert result == 0
    captured = capsys.readouterr()
    assert captured.out.strip() == "0.1.1"
    assert 'version = "0.1.1"' in cargo_toml.read_text()


def test_invalid_branch_returns_nonzero_and_leaves_file_unchanged(tmp_path, capsys):
    cargo_toml = tmp_path / "Cargo.toml"
    original_content = """[workspace.package]
version = "0.1.0"
name = "ksim"
"""
    cargo_toml.write_text(original_content)
    original_bytes = cargo_toml.read_bytes()

    result = main(
        [
            "--cargo-toml",
            str(cargo_toml),
            "--branch",
            "feature/x",
            "--date",
            "2026-09-29",
        ]
    )

    assert result != 0
    captured = capsys.readouterr()
    assert captured.err
    assert cargo_toml.read_bytes() == original_bytes
