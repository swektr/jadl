use std::{fs,
          env::var,
          io::{self, Read, Write},
          path::{Path, PathBuf},
          process::{Command, ExitCode, Stdio}}; // dont know why i did this
use clap::Parser;
use libmpv::Mpv;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use serde_derive::{Serialize, Deserialize};
use std::time::Duration;
use jadl::timer::Timer;


trait PathExt {
    fn file_exists(&self) -> bool;
}

impl PathExt for Path {
    fn file_exists(&self) -> bool {
        return self.is_file() && self.exists();
    }
}

#[derive(Debug, Clone)]
struct DownloadError;

#[derive(Parser, Debug)]
struct Cli {
    /// Word in kanji
    kanji: String,
    /// The word's reading
    kana: String,
    /// Force overwrite files
    #[clap(long,short,action)]
    force: bool,
    /// Use the anki directory
    #[clap(long,short,action)]
    anki: bool,
    /// Copies a formated "[sound:]" string to your clipboard.
    /// Use with --anki option. (or don't)
    #[clap(long,short,action,verbatim_doc_comment)]
    copy: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct JadlConfig {
    anki_dir: Option<String>,
    dest_dir: String,
}

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
    
    return ExitCode::SUCCESS;
}

fn load_config() -> JadlConfig {
    let config_path = var("XDG_CONFIG_HOME")
                    .or_else( |_| var("HOME")
                        .map(|home| format!("{}/.config/jadl/config.toml",home))
                    )
                    .expect("Error reading environment variables");        
    
    return match fs::read_to_string(&config_path) {
        Ok(contents) =>{ 
            toml::from_str(&contents).expect("Error parsing config file")
        }
        Err(_) => {
            JadlConfig {
                anki_dir: None,
                dest_dir: var("HOME").unwrap(),
            }
        }
    }
}


fn curl_download(url: &String, file: &Path) -> Result<(), DownloadError> {
    let file_string = file.as_os_str()
                          .to_str()
                          .expect("Error converting Path to &str");
    
    let curl_timer = Timer::new(Duration::from_millis(1000),
                             || println!("This may take a moment..."));
    curl_timer.start();

    println!("Running 'curl'");
    let curl_status = Command::new("curl")
                              .args([url, "--output", file_string])
                              .output()
                              .expect("Error running curl")
                              .status;
    curl_timer.cancel(); 

    return match curl_status.code() {
        Some(0) => {
            println!("Download Success");
            Ok(())
        },
        Some(x) => {
            println!("Curl exited with code: {x}");
            Err(DownloadError)
        }
        None => {
            println!("Curl terminated by signal");
            Err(DownloadError)
        },
    }
}

fn play_audio_and_prompt_loop(path: &Path) -> bool {
    let file_string = path.as_os_str().to_str()
                          .expect("Error converting Path to &str"); 
    // Create new mpv instance
    let mpv = Mpv::new().unwrap();
    mpv.set_property("keep-open", "yes").unwrap();
    mpv.set_property("keep-open-pause", "no").unwrap();
    mpv.command("loadfile", &[&file_string]).unwrap();

    let stdin_fd = 0; // stdin's file descriptor is 0
    let termios_default = Termios::from_fd(stdin_fd).unwrap();
    
    let mut termios_grab_key = termios_default.clone();
    // Recieve characters as soon as typed
    termios_grab_key.c_lflag &= !ICANON; 
    // Turn off echoing back typed characters
    termios_grab_key.c_lflag &= !ECHO; 
    
    let stdout = io::stdout();
    let mut stdin = io::stdin();
    let mut char_buf: [u8; 1] = [0];
    tcsetattr(stdin_fd, TCSANOW, &termios_grab_key).unwrap();
    
    print!("Save?(Y/S) Replay?(R) : ");
    
    let should_save = loop {
        stdout.lock().flush().unwrap(); // This makes sure the above gets printed 
        stdin.read_exact(&mut char_buf).unwrap();
        match char_buf.get(0)  {
            Some(b'r') | Some(b'R') => {
                mpv.command("seek", &["0", "absolute"]) // Play from start (time=0)
                   .expect("Something went wrong when trying to replay")
            },
            Some(b'y') | Some(b'Y') |
            Some(b's') | Some(b'S') => break true,
            _ => break false,
        }

    };
    tcsetattr(stdin_fd, TCSANOW, &termios_default).unwrap(); // Reset terminal mode
    println!();
     
    return should_save;
}

fn move_file(from: &Path, to: &Path) {
    // Rather than rename(), which cant move files across different mounts, I use copy() then remove() 
    fs::copy(from, to).expect("temp -> dest file copy failed!");
    if let Err(e) = fs::remove_file(from) {
        eprintln!("Error removing temp file: {:?}", e);
    }
    println!("File saved to \x1b[0;32m{:?}\x1b[0m", to.as_os_str());
}

fn set_clipboard(txt: &String) -> Result<(), Box<dyn std::error::Error>> {
    let mut xsel = Command::new("xsel")
                       .arg("-bi")
                       .stdin(Stdio::piped())
                       .spawn()?;
    let xsel_in = xsel.stdin.as_mut().unwrap();
    xsel_in.write_all(txt.as_bytes())?;
    xsel.wait()?;

    Ok(())
}