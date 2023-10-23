use predicates::str::is_match;
use std::fs::File;
use std::io::Write;
use std::{env, fs};
use tempfile::TempDir;

#[test]
fn init() {
    let folder = tempfile::tempdir().unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "init"])
        .assert()
        .success()
        .stdout(is_match(
            format!("Initialized new shrine with UUID [a-f0-9]{{8}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{12}} in `{}/shrine`\\n", folder.path().display()),
        ).unwrap());

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "init"])
        .assert()
        .failure()
        .stderr(format!(
            "Shrine file `{}/shrine` already exists\n",
            folder.path().display()
        ));
}

#[test]
fn init_other_folder() {
    let folder = tempfile::tempdir().unwrap();

    fs::create_dir(folder.path().join("other")).unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--path", "other", "--password", "p", "init"])
        .assert()
        .success()
        .stdout(is_match(
            format!("Initialized new shrine with UUID [a-f0-9]{{8}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{12}} in `{}/other/shrine`\\n", folder.path().display()),
        ).unwrap());

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--path", "other", "--password", "p", "init"])
        .assert()
        .failure()
        .stderr(format!(
            "Shrine file `{}/other/shrine` already exists\n",
            folder.path().display()
        ));
}

#[test]
fn set() {
    let folder = create_shrine("p");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn get() {
    let folder = create_shrine("p");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "get", "key"])
        .assert()
        .success()
        .stdout("val");
}

#[test]
fn delete() {
    let folder = create_shrine("p");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "rm", "key"])
        .unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "get", "key"])
        .assert()
        .failure()
        .stderr("Key `key` does not exist\n");
}

#[test]
fn convert_change_pwd() {
    let folder = create_shrine("p");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "convert", "--new-password", "p1"])
        .assert()
        .success()
        .stderr("");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p1", "get", "key"])
        .assert()
        .success()
        .stdout("val");
}

#[test]
fn convert_no_pwd() {
    let folder = create_shrine("p");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "convert", "--encryption", "none"])
        .assert()
        .success()
        .stderr("");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["get", "key"])
        .assert()
        .success()
        .stdout("val");
}

#[test]
fn import() {
    let folder = create_shrine("p");

    let file_path = folder.path().join("env-file");
    let mut file = File::create(file_path.clone()).unwrap();
    writeln!(file, "key1=val1#comment\n#a comment\n\nkey2=val2==").unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec![
            "--password",
            "p",
            "import",
            file_path.display().to_string().as_str(),
        ])
        .assert()
        .success()
        .stdout("");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "get", "key1"])
        .assert()
        .success()
        .stdout("val1");
    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "get", "key2"])
        .assert()
        .success()
        .stdout("val2==");
}

#[test]
fn import_with_prefix() {
    let folder = create_shrine("p");

    let file_path = folder.path().join("env-file");
    let mut file = File::create(file_path.clone()).unwrap();
    writeln!(file, "key=val").unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec![
            "--password",
            "p",
            "import",
            file_path.display().to_string().as_str(),
            "--prefix",
            "env/",
        ])
        .assert()
        .success()
        .stdout("");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "get", "env/key"])
        .assert()
        .success()
        .stdout("val");
}

fn create_shrine(pwd: &str) -> TempDir {
    let folder = tempfile::tempdir().unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", pwd, "init"])
        .assert()
        .success()
        .stdout(is_match(
            format!("Initialized new shrine with UUID [a-f0-9]{{8}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{12}} in `{}/shrine`\\n", folder.path().display())
        ).unwrap());
    folder
}

#[test]
fn git() {
    let folder = tempfile::tempdir().unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "init", "--git"])
        .assert()
        .success()
        .stdout(is_match(
            format!("Initialized new shrine with UUID [a-f0-9]{{8}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{12}} in `{}/shrine`; git commit [a-f0-9]{{40}} in {}\\n", folder.path().display(), folder.path().display()),
        ).unwrap());

    assert_cmd::Command::new("git")
        .current_dir(&folder)
        .args(vec!["log", "-n", "1", "--format=format:%s"])
        .assert()
        .success()
        .stdout("Initialize shrine");
}

#[test]
fn git_disable_auto_commit() {
    let folder = tempfile::tempdir().unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "init", "--git"])
        .assert()
        .success()
        .stdout(is_match(
            format!("Initialized new shrine with UUID [a-f0-9]{{8}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{12}} in `{}/shrine`; git commit [a-f0-9]{{40}} in {}\\n", folder.path().display(),  folder.path().display()),
        ).unwrap());

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec![
            "--password",
            "p",
            "config",
            "set",
            "git.commit.auto",
            "false",
        ])
        .assert()
        .success();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .assert()
        .success()
        .stdout("");

    assert_cmd::Command::new("git")
        .current_dir(&folder)
        .args(vec!["rev-list", "HEAD", "--count"])
        .assert()
        .success()
        .stdout("1\n");
}

#[test]
fn git_then_disable_git() {
    let folder = tempfile::tempdir().unwrap();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "init", "--git"])
        .assert()
        .success()
        .stdout(is_match(
            format!("Initialized new shrine with UUID [a-f0-9]{{8}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{4}}-[a-f0-9]{{12}} in `{}/shrine`; git commit [a-f0-9]{{40}} in {}\\n", folder.path().display(), folder.path().display()),
        ).unwrap());

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec![
            "--password",
            "p",
            "config",
            "set",
            "git.enabled",
            "false",
        ])
        .assert()
        .success();

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .current_dir(&folder)
        .args(vec!["--password", "p", "set", "key", "val"])
        .assert()
        .stdout("");

    assert_cmd::Command::new("git")
        .current_dir(&folder)
        .args(vec!["rev-list", "HEAD", "--count"])
        .assert()
        .success()
        .stdout("1\n");
}

#[test]
fn info_format_v0() {
    let folder = env::current_dir().unwrap();
    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .args(vec!["--path", "test-data/v0-bson-aes", "info"])
        .assert()
        .success()
        .stdout(format!(
            r#"File:          {}/test-data/v0-bson-aes/shrine
Version:       0
UUID:          70f61568-eaae-a085-cd47-49650e58df08
Serialization: BSON
Encryption:    AES-GCM-SIV with 256-bits key
"#,
            folder.display()
        ));

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .args(vec!["--path", "test-data/v0-bson-clear", "info"])
        .assert()
        .success()
        .stdout(format!(
            r#"File:          {}/test-data/v0-bson-clear/shrine
Version:       0
UUID:          920e25c6-eced-53bd-da44-914201a8fba7
Serialization: BSON
Encryption:    Not encrypted
"#,
            folder.display()
        ));
}

#[test]
fn get_format_v0() {
    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .args(vec![
            "--path",
            "test-data/v0-bson-aes",
            "--password",
            "pwd",
            "get",
            "key",
        ])
        .assert()
        .success()
        .stdout("value");

    assert_cmd::Command::cargo_bin("shrine")
        .unwrap()
        .args(vec!["--path", "test-data/v0-bson-clear", "get", "key"])
        .assert()
        .success()
        .stdout("value");
}
