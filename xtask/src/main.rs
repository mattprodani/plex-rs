use anyhow::{Context, Result};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    // 1. Build the bootloader
    println!("Building plex-boot...");
    let status = Command::new("cargo")
        .args([
            "build",
            "--package",
            "plex-boot",
            "--target",
            "x86_64-unknown-uefi",
        ])
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    // 2. Prepare ESP
    println!("Preparing ESP...");
    let target_dir = PathBuf::from("target");
    let esp_dir = target_dir.join("test-esp");
    let boot_dir = esp_dir.join("EFI").join("BOOT");

    if esp_dir.exists() {
        fs::remove_dir_all(&esp_dir).context("Failed to clean old ESP")?;
    }
    fs::create_dir_all(&boot_dir).context("Failed to create BOOT dir")?;

    let efi_src = target_dir
        .join("x86_64-unknown-uefi")
        .join("debug")
        .join("plex-boot.efi");
    let efi_dest = boot_dir.join("BOOTX64.EFI");
    fs::copy(&efi_src, &efi_dest).context("Failed to copy EFI binary")?;

    // Create dummy plex.toml
    let toml_content = r#"
theme = "default"

[[boot_targets]]
type = "generic"
label = "Dummy Linux"
executable = "\\kernel"
options = "root=/dev/sda1"
"#;
    let toml_path = esp_dir.join("plex.toml");
    fs::write(&toml_path, toml_content).context("Failed to write plex.toml")?;

    // 3. Fetch OVMF
    println!("Fetching OVMF...");
    let prebuilt = Prebuilt::fetch(Source::EDK2_STABLE202502_R2, target_dir.join("ovmf"))
        .context("Failed to fetch OVMF")?;
    let code_path = prebuilt.get_file(Arch::X64, FileType::Code);
    let vars_path = prebuilt.get_file(Arch::X64, FileType::Vars);

    // Make a copy of vars so we can use it read-write if necessary, or just use read-only
    let local_vars = target_dir.join("test_ovmf_vars.fd");
    fs::copy(&vars_path, &local_vars).context("Failed to copy vars")?;

    // 4. Run QEMU
    println!("Running QEMU...");
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.args([
        "-nodefaults",
        "-machine",
        "q35",
        "-m",
        "256M",
        "-display",
        "none", // headless
        "-vga",
        "std", // Still provide a VGA device so GOP is available!
    ]);

    // Setup firmware
    qemu.arg("-drive").arg(format!(
        "if=pflash,format=raw,readonly=on,file={}",
        code_path.display()
    ));
    qemu.arg("-drive").arg(format!(
        "if=pflash,format=raw,readonly=on,file={}",
        local_vars.display()
    ));

    // Setup ESP
    qemu.arg("-drive")
        .arg(format!("format=raw,file=fat:rw:{}", esp_dir.display()));

    // Setup Serial output to capture
    qemu.arg("-serial").arg("stdio");

    qemu.stdout(Stdio::piped());
    qemu.stderr(Stdio::piped());

    let mut child = qemu.spawn().context("Failed to spawn QEMU")?;

    let stdout = child.stdout.take().expect("Failed to grab stdout");

    let (tx, rx) = mpsc::channel();

    // Spawn a thread to read stdout
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for l in reader.lines().map_while(Result::ok) {
            println!("[QEMU] {}", l);
            if l.contains("Loaded 1 boot targets from config") {
                let _ = tx.send(true);
                break;
            }
        }
    });

    // Wait for success or timeout
    let timeout = Duration::from_secs(15);
    let start = Instant::now();
    let mut success = false;

    while start.elapsed() < timeout {
        if let Ok(res) = rx.recv_timeout(Duration::from_millis(100)) {
            success = res;
            break;
        }

        // Check if QEMU died unexpectedly
        if let Ok(Some(status)) = child.try_wait() {
            println!("QEMU exited unexpectedly with status: {}", status);
            break;
        }
    }

    // Terminate QEMU
    let _ = child.kill();
    let _ = child.wait();

    if success {
        println!("Integration test PASSED!");
        Ok(())
    } else {
        anyhow::bail!("Integration test FAILED! (Timeout or exit before success condition)");
    }
}
