use std::env;
use anyhow::Result;
use std::process;
use serde_json;
use curl::easy::Easy;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

use reqwest;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    if args.len() != 1 { 
        println!("Expected 1 argument, got {}", args.len());
        process::exit(1)
    }
    let ver = &args[0];
    let url = get_installer_url().await?;
    let installer_path = download(url, format!("{}/.cache/fabric-installers/", env::var("HOME").unwrap())).await?;
    run(&installer_path, ver.clone()).expect("Failed to run installer! Aborting...");
    println!("Successfully installed Fabric {} with {}", ver, &installer_path);
    Ok(())
}

fn run(file: &String, version: String) -> Result<()> {
    println!("Running installer {}\n", file);
    let mut child = process::Command::new("java")
        .arg("-jar")
        .arg(file)
        .arg("client")
        .arg("-snapshot")
        .arg("-mcversion")
        .arg(version).spawn()?;
    child.wait()?;
    Ok(())
}

fn is_empty(path: &Path) -> bool {
    match fs::read_dir(path) {
        Ok(entries) => entries.count() == 0,
        Err(_) => true,
    }
}

fn clear_dir(path: &Path) {
    if !is_empty(path) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                fs::remove_file(entry.path()).unwrap();
            }
        }
    }
}

async fn download(url: String, folder: String) -> Result<String> {
    let mut easy = Easy::new();
    easy.url(&url).unwrap();
    easy.follow_location(true).unwrap();
    if !Path::new(&folder).exists() {
        fs::create_dir_all(&folder).unwrap();
    }
    let installer_name = url.split("/").last().unwrap();
    let full_path = format!("{}{}", folder, installer_name);
    if Path::new(&full_path).exists() {
        println!("You already have the newest installer, skipping download...\n");
    } else {
        if !is_empty(&Path::new(&folder)) {
            println!("Found an old installer, removing...\n");
            clear_dir(&Path::new(&folder));
        }

        println!("Downloading to {} from {}\n", full_path, url);
        let mut f = fs::File::create(&full_path)?;
        easy.write_function(move |data| {
            f.write_all(data).unwrap();
            Ok(data.len())
        }).unwrap();
        easy.perform().unwrap();
        println!("Downloaded!");
    }

    Ok(full_path)
}

async fn get_installer_url() -> Result<String> {
    Ok(reqwest::get("https://meta.fabricmc.net/v2/versions/installer").await?.json::<serde_json::Value>().await?[0]["url"].as_str().unwrap().to_string())
}
