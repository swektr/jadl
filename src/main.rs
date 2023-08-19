use std::{path::PathBuf,
          process::ExitCode};
use clap::Parser;
use jadl::*;

fn main() -> ExitCode {
    let args = Cli::parse();
    
    let config = load_config();

    let url = "https://assets.languagepod101.com/dictionary/japanese/audiomp3.php?";
    let url = format!("{}kana={}&kanji={}",url, args.kana,args.kanji);
    
    // Create file path strings
    let filename = format!("{}({}).mp3", args.kanji, args.kana);
    let mut temp_path = PathBuf::from("/tmp/");
    temp_path.push(&filename);
    let mut dest_path = match args.anki {
        true => {
            if let Some(x) = &config.anki_dir {
                PathBuf::from(x)
            } else {
                eprintln!("Error: anki_dir not set in config.toml");
                return ExitCode::FAILURE;
            }  
        },
        false => PathBuf::from(config.dest_dir),
    };
    dest_path.push(&filename);

    if dest_path.file_exists() && ! args.force {
        println!("File \x1b[0;33m{:?}\x1b[0m already exists!\nRun with -f flag to force overwrite",
                  dest_path.as_os_str());
        return ExitCode::FAILURE;
    }

    if args.copy {
        let sound_str = format!("[sound:{}]", filename);
        if let Err(_) = set_clipboard(&sound_str) {
            eprintln!("Error setting clipboard!");
        }
    }

    // Download file
    if let Err(_) = curl_download(&url, &temp_path) {
        return ExitCode::FAILURE;
    }

    // Play
    if play_audio_and_prompt_loop(&temp_path) {
        move_file(&temp_path, &dest_path);
    }
    
    ExitCode::SUCCESS
}