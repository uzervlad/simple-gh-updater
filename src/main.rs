use std::{fs::{File, self}, io::{Write, self}};

use indicatif::{ProgressBar, ProgressStyle, ProgressState};

const REPO: &'static str = "uzervlad/kool-gosu";
const ZIP_NAME: &'static str = "kool.zip";

#[tokio::main]
async fn main() {
  {
    let url = format!("https://github.com/{}/releases/latest/download/{}", REPO, ZIP_NAME);
    let mut source = reqwest::get(url).await.expect("Unable to fetch zip");
    let content_length = str::parse::<u64>(source.headers().get("content-length").unwrap().to_str().unwrap()).unwrap();
    
    println!("Downloading zip");
    let bar = ProgressBar::new(content_length);
    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.green}] {bytes}/{total_bytes} ({eta})")
      .unwrap()
      .with_key("eta", |state: &ProgressState, w: &mut dyn std::fmt::Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
      .progress_chars("#>-"));

    let mut size: u64 = 0;
  
    let mut dest = File::create(ZIP_NAME).unwrap();
    while let Some(chunk) = source.chunk().await.unwrap() {
      dest.write_all(&chunk).unwrap();
      let new = size + chunk.len() as u64;
      size = new;
      bar.set_position(new);
    }

    if content_length != size {
      bar.finish_and_clear();
      println!("Failed to download zip");
      fs::remove_file(ZIP_NAME).unwrap();
      return;
    }
  }

  println!("Deleting previous files");

  {
    let exe_path = std::env::current_exe().unwrap();
    let exe = exe_path.file_name().unwrap();

    for entry in fs::read_dir(".").unwrap() {
      let entry = entry.unwrap();
      let path = entry.path();

      if entry.file_type().unwrap().is_dir() {
        fs::remove_dir_all(path).unwrap();
      } else if !path.file_name().unwrap().eq(exe) 
        && !path.file_name().unwrap().to_str().unwrap().eq(ZIP_NAME) 
      {
        fs::remove_file(path).unwrap();
      }
    }
  }

  let archive = File::open(ZIP_NAME).unwrap();
  let mut archive = zip::ZipArchive::new(archive).unwrap();

  for i in 0..archive.len() {
    let mut file = archive.by_index(i).unwrap();
    let path = match file.enclosed_name() {
      Some(path) => path.to_owned(),
      None => continue,
    };

    if path.file_name().unwrap().to_str().unwrap().eq(ZIP_NAME) {
      continue;
    }

    if (*file.name()).ends_with('/') {
      fs::create_dir_all(&path).unwrap();
    } else {
      if let Some(p) = path.parent() {
        if !p.exists() {
          fs::create_dir_all(p).unwrap();
        }
      }
      let mut outfile = fs::File::create(&path).unwrap();
      io::copy(&mut file, &mut outfile).unwrap();
      println!("Extracted {}", path.display());
    }
  }

  println!("Deleting temporary file");

  fs::remove_file(ZIP_NAME).unwrap();

  println!("Update successful");
}
