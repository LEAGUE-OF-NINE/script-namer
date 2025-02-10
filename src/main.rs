use chrono::Local;
use rayon::prelude::*;
use regex::Regex;
use rfd::FileDialog;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::LazyLock;
use walkdir::WalkDir;

// useful site for making regex: https://rustexp.lpil.uk/
static SCRIPT_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""scriptName":\s*"([^"]+)""#).unwrap());

fn main() {
    // self explanatory
    let folder = match FileDialog::new().pick_folder() {
        Some(path) => path,
        None => {
            eprintln!("No folder selected.");
            return;
        }
    };

    // this code is slow btw, but idc
    let benchmrk = std::time::Instant::now();

    let script_names: BTreeSet<String> = WalkDir::new(&folder)
        .into_iter() // iterators = wife material
        .par_bridge() // I LOVE RAYON WE ALL RAYONING IN THE RAYONCORE CITY
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            // this is prob unnecessary but eh, i aint tryna parse no damn nonjsons
            path.is_file() && path.extension().map_or(false, |ext| ext == "json")
        })
        .filter_map(|entry| fs::read_to_string(entry.path()).ok())
        .flat_map(|content| {
            SCRIPT_NAME_RE
                .captures_iter(&content)
                .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
                .collect::<Vec<_>>() // baby... you let out so much...
        })
        .collect(); // mmm, oh dear... you still want more..?

    // ts (these) comments pmo me off

    if script_names.is_empty() {
        println!("No script names found.");
        return;
    }

    // timestamp so we know last update of script dump
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let output_filename = format!("script_names_{}.txt", timestamp);
    // save in cur dir
    let output_path = Path::new("./").join(output_filename);

    if let Ok(mut file) = File::create(&output_path) {
        // autism
        file.write_all(
            script_names
                .into_par_iter()
                .collect::<Vec<_>>()
                .join("\n")
                .as_bytes(),
        )
        .unwrap();
        println!("Script names saved to: {:?}", output_path);
    } else {
        eprintln!("Failed to create output file.");
    }

    println!("{} secs elapsed.", benchmrk.elapsed().as_secs_f32());
}
