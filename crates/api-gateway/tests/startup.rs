//! The gateway refuses to start without a JWT secret. These need no Docker: the secret is
//! checked before the database is touched, so the process exits straight away.

use std::process::{Command, Stdio};

fn run_gateway(secret: Option<&str>) -> (bool, String) {
    // an empty working directory, so dotenv doesn't find the repo's .env and fill in the secret
    let workdir = std::env::temp_dir().join(format!("api-gateway-startup-{}-{}", std::process::id(), secret.is_some()));
    std::fs::create_dir_all(&workdir).unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_api-gateway"));
    cmd.current_dir(&workdir)
        .env_clear()
        // a URL that cannot be parsed, so the process stops at once instead of retrying the
        // connection for 30 seconds; if the secret check were skipped the failure would be
        // about this URL instead, with a different message
        .env("DATABASE_URL", "postgres://nobody:hunter2@127.0.0.1:not-a-port/none")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(secret) = secret {
        cmd.env("JWT_SECRET", secret);
    }

    let out = cmd.output().expect("failed to run api-gateway");
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    (out.status.success(), text)
}

#[test]
fn it_refuses_to_start_without_a_jwt_secret() {
    let (ok, output) = run_gateway(None);

    assert!(!ok);
    assert!(output.contains("JWT_SECRET"), "{output}");
}

#[test]
fn a_blank_jwt_secret_counts_as_missing() {
    let (ok, output) = run_gateway(Some("   "));

    assert!(!ok);
    assert!(output.contains("JWT_SECRET"), "{output}");
}

#[test]
fn the_database_password_is_never_printed() {
    // with a secret set it gets as far as the database URL and fails there
    let (ok, output) = run_gateway(Some("startup-test-secret"));

    assert!(!ok);
    assert!(!output.contains("hunter2"), "the database password leaked into the output:\n{output}");
}
