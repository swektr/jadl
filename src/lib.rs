use std::{fs,
    env::var,
    io::{self, Read, Write},
    path::Path,
    process::{Command, Stdio}};
use clap::Parser;
use libmpv::Mpv;
use termios::{Termios, TCSANOW, ECHO, ICANON, tcsetattr};
use serde_derive::{Serialize, Deserialize};
use std::time::Duration;

mod timer;
use timer::Timer;

pub trait PathExt {
    fn file_exists(&self) -> bool;
}

impl PathExt for Path {
    fn file_exists(&self) -> bool {
        return self.is_file() && self.exists();
    }
}

#[derive(Debug, Clone)]
pub struct DownloadError;

#[derive(Parser, Debug)]
pub struct Cli {
    /// Word in kanji
    pub kanji: String,
    /// The word's reading
    pub kana: String,
    /// Force overwrite files
    #[clap(long,short,action)]
    pub force: bool,
    /// Use the anki directory
    #[clap(long,short,action)]
    pub anki: bool,
    /// Copies a formated "[sound:]" string to your clipboard.
    /// Use with --anki option. (or don't)
    #[clap(long,short,action,verbatim_doc_comment)]
    pub copy: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JadlConfig {
    pub anki_dir: Option<String>,
    pub dest_dir: String,
}

pub fn load_config() -> JadlConfig {
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


pub fn curl_download(url: &String, file: &Path) -> Result<(), DownloadError> {
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

    match curl_status.code() {
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

pub fn play_audio_and_prompt_loop(path: &Path) -> bool {
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
     
    should_save
}

pub fn move_file(from: &Path, to: &Path) {
    // Rather than rename(), which cant move files across different mounts, I use copy() then remove() 
    fs::copy(from, to).expect("temp -> dest file copy failed!");
    if let Err(e) = fs::remove_file(from) {
        eprintln!("Error removing temp file: {:?}", e);
    }
    println!("File saved to \x1b[0;32m{:?}\x1b[0m", to.as_os_str());
}

pub fn set_clipboard(txt: &String) -> Result<(), Box<dyn std::error::Error>> {
    let mut xsel = Command::new("xsel")
                       .arg("-bi")
                       .stdin(Stdio::piped())
                       .spawn()?;
    let xsel_in = xsel.stdin.as_mut().unwrap();
    xsel_in.write_all(txt.as_bytes())?;
    xsel.wait()?;

    Ok(())
}