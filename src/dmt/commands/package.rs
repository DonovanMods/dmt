use crate::dmt::SETTINGS;
use color_eyre::eyre::{eyre, Result};
use console::{style, Term};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
// use rand::random;
use modlet::Modlet;
use rayon::prelude::*;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

/// Reads a modlet's xml files
fn load(path: impl AsRef<Path>, padding: usize, pb: &ProgressBar) -> Result<Modlet> {
    let file_name = path.as_ref().file_name().unwrap_or(OsStr::new("")).to_str().unwrap();
    let verbose = SETTINGS.read().unwrap().verbosity > 0;
    if verbose {
        pb.set_prefix(format!("Loading {file_name:.<padding$} "));
    }

    let config_dir = path.as_ref().join("config");
    if !(config_dir.exists() && config_dir.is_dir()) {
        return Err(eyre!(
            "Invalid Modlet {}: Config directory does not exist",
            config_dir.display()
        ));
    }

    let modlet = Modlet::new(path.as_ref())?;

    dbg!(&modlet);

    Ok(modlet)
}

fn package(modlets: &[Modlet], output_modlet: &Path, padding: usize, pb: &ProgressBar) -> Result<()> {
    let verbose = SETTINGS.read().unwrap().verbosity > 0;
    let output_modlet_name = output_modlet.file_name().unwrap().to_str().unwrap();

    if verbose {
        pb.set_prefix(format!("Packaging {output_modlet_name:.<padding$} "));
    }

    for modlet in modlets {
        if verbose {
            pb.set_message(format!("Bundling {:.<padding$} ", &modlet.name()));
        }

        {
            for _ in 0..100 {
                if verbose {
                    pb.inc(1);
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
    }

    // todo!("Package modlets into a single modlet");
    Ok(())
}

/// Packages one or more modlets into a single modlet
///
/// # Arguments
///
/// * `modlets` - A list of modlet(s) to package
/// * `modlet` - The path to the modlet to package into
///
/// # Errors
///
/// * If the game directory is invalid
/// * If the modlet path is invalid
///
pub fn run(modlets: &[PathBuf], modlet: &Path) -> Result<()> {
    let verbose = SETTINGS.read().unwrap().verbosity > 0;
    let game_dir = SETTINGS.read().unwrap().game_directory.clone();
    let count = modlets.len() as u64;
    let mp = MultiProgress::new();
    let spinner_style = ProgressStyle::with_template("{prefix:.cyan.bright} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let mut padding = modlets
        .iter()
        .map(|p| p.as_path().file_name().unwrap().len())
        .max()
        .unwrap_or(0);
    let term = Term::stdout();

    let modlet_name = modlet.file_name().unwrap().to_str().unwrap();
    if padding < modlet_name.len() {
        padding = modlet_name.len();
    }

    if verbose {
        term.clear_screen()?;
        term.write_line(
            style(format!("Packaging {count} modlet(s) into {}...\n", modlet.display()))
                .yellow()
                .to_string()
                .as_ref(),
        )?;
    }

    // let gamexmls;
    if let Some(gamedir) = game_dir {
        if !gamedir.exists() {
            return Err(eyre!("Game directory does not exist: {}", gamedir.display()));
        }
        // gamexmls = gamexml::read(&gamedir)?;
    } else {
        return Err(eyre!("Game directory not set"));
    }

    // dbg!(gamexmls);
    // return Ok(());

    // Using `par_iter()` to parallelize the validation of each modlet.
    let loaded_modlets: Vec<Modlet> = modlets
        .par_iter()
        .fold(Vec::<Modlet>::new, |mut vf, path| {
            let pb = mp.add(ProgressBar::new(count));
            pb.set_style(spinner_style.clone());

            match load(path, padding + 3, &pb) {
                Ok(modlet) => {
                    if verbose {
                        pb.finish_with_message(style("OKAY").green().bold().to_string());
                    }
                    vf.push(modlet);
                }

                Err(err) => {
                    if verbose {
                        pb.finish_with_message(format!(
                            "{} {}",
                            style("FAIL").red().bold(),
                            style(format!("({err})")).red()
                        ));
                    }
                }
            }

            vf
        })
        .reduce(Vec::<Modlet>::new, |mut vf, mut v| {
            vf.append(&mut v);
            vf
        });

    if (loaded_modlets.len() as u64) == count {
        let pb = mp.add(ProgressBar::new(1));
        pb.set_style(spinner_style.clone());

        match package(&loaded_modlets, modlet, padding + 1, &pb) {
            Ok(_) => {
                if verbose {
                    pb.finish_with_message(style("OKAY").green().bold().to_string());
                }
            }
            Err(err) => {
                if verbose {
                    pb.finish_with_message(format!(
                        "{} {}",
                        style("FAIL").red().bold(),
                        style(format!("({err})")).red()
                    ));
                }
            }
        }
    } else {
        term.write_line(
            style(format!(
                "\n\n{count} modlet(s) failed to package!\n",
                count = count - (loaded_modlets.len() as u64)
            ))
            .red()
            .to_string()
            .as_ref(),
        )?;
    }

    Ok(())
}
