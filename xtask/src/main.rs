use std::*;

struct Helper {
    metadata: cargo_metadata::Metadata,
}

impl Helper {
    fn new() -> cargo_metadata::Result<Helper> {
        let metadata = cargo_metadata::MetadataCommand::new().exec()?;
        let bundled_dir = metadata.target_directory.join("bundled");
        fs::create_dir(bundled_dir).ok();
        Ok(Helper { metadata: metadata })
    }

    fn bundle_exe(&self, bin: &str) -> io::Result<()> {
        self.cargo(&["build", "--release", "--bin", bin])?;
        self.copy(&format!("{}{}", bin, env::consts::EXE_SUFFIX))?;
        Ok(())
    }

    fn cargo(&self, args: &[&str]) -> io::Result<()> {
        match process::Command::new("cargo").args(args).spawn()?.wait()?.success() {
            true => Ok(()),
            false => Err(io::ErrorKind::Other.into()),
        }
    }

    fn copy(&self, path: &str) -> io::Result<()> {
        let release_dir = self.metadata.target_directory.join("release");
        let bundled_dir = self.metadata.target_directory.join("bundled");
        fs::copy(release_dir.join(path), bundled_dir.join(path))?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    match env::args().skip(1).next().ok_or("")?.as_str() {
        "deploy" => {
            let helper = Helper::new()?;
            helper.bundle_exe("memol_cli")?;
            helper.bundle_exe("memol_gui")?;
            nih_plug_xtask::main_with_args("", ["bundle", "memol_nih", "--release"].map(str::to_string))?;
        }
        "nih-plug" => {
            nih_plug_xtask::main_with_args("", env::args().skip(2))?;
        }
        _ => return Err("".into()),
    }

    Ok(())
}
