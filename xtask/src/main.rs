use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err("expected command: update-palettes".into());
    };

    if args.next().is_some() {
        return Err("unexpected extra arguments".into());
    }

    match command.as_str() {
        "update-palettes" => update_palettes(),
        _ => Err(format!("unknown command `{command}`; expected `update-palettes`").into()),
    }
}

fn update_palettes() -> Result<(), Box<dyn Error>> {
    let root = workspace_root()?;
    let source =
        env::var_os("GGSCI_SOURCE").map_or_else(|| root.join("vendor/ggsci"), PathBuf::from);
    let output = root.join("crates/ggsci/src/generated/palettes.rs");
    let fixture_output = root.join("crates/ggsci/tests/generated/continuous_fixtures.rs");
    let iterm_output = root.join("crates/ggsci/src/generated/iterm.rs");
    let gephi_output = root.join("crates/ggsci/src/generated/gephi.rs");

    run_command(
        Command::new("Rscript")
            .current_dir(&root)
            .arg("tools/generate-palettes.R")
            .arg(&source)
            .arg(&output),
    )?;
    run_command(
        Command::new("Rscript")
            .current_dir(&root)
            .arg("tools/generate-continuous-fixtures.R")
            .arg(&source)
            .arg(&fixture_output),
    )?;
    run_command(
        Command::new("Rscript")
            .current_dir(&root)
            .arg("tools/generate-iterm-palettes.R")
            .arg(&source)
            .arg(&iterm_output),
    )?;
    run_command(
        Command::new("Rscript")
            .current_dir(&root)
            .arg("tools/generate-gephi-palettes.R")
            .arg(&source)
            .arg(&gephi_output),
    )?;
    run_command(
        Command::new("cargo")
            .current_dir(&root)
            .arg("fmt")
            .arg("--all"),
    )?;

    Ok(())
}

fn workspace_root() -> Result<PathBuf, Box<dyn Error>> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "xtask manifest directory has no parent".into())
}

fn run_command(command: &mut Command) -> Result<(), Box<dyn Error>> {
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("command exited with status {status}").into())
    }
}
