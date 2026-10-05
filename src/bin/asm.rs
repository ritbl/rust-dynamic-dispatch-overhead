//! `cargo asm`: compile optimized library assembly and display both dispatch loops.
use std::{env, error::Error, fs, io, path::Path, process::Command};

fn show_function(assembly: &str, name: &str) -> Result<(), io::Error> {
    let mut lines = assembly.lines();
    let label = lines
        .find(|line| line.trim_end().ends_with(':') && line.contains(name))
        .ok_or_else(|| io::Error::other(format!("Assembly function {name} not found")))?;

    println!("\n=== {name} ===");
    println!("{label}");
    // Keep instructions and loop labels; omit unwind and alignment directives.
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with(".cfi_endproc")
            || trimmed.starts_with(".size")
            || trimmed.starts_with(".globl")
            || trimmed.starts_with(".section")
        {
            break;
        }
        if !trimmed.is_empty() && (!trimmed.starts_with('.') || trimmed.ends_with(':')) {
            println!("{line}");
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    fs::create_dir_all(root.join("target"))?;
    let status = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .current_dir(root)
        .args([
            "rustc",
            "--lib",
            "--release",
            "--",
            "--emit=asm=target/dispatch.s",
        ])
        .status()?;
    if !status.success() {
        return Err(io::Error::other("Release assembly compilation failed").into());
    }

    let assembly = fs::read_to_string(root.join("target/dispatch.s"))?;
    show_function(&assembly, "static_dispatch")?;
    show_function(&assembly, "vtable_dispatch")?;
    println!(
        "\nFull assembly: {}",
        root.join("target/dispatch.s").display()
    );
    Ok(())
}
