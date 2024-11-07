use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    process::{Command, Child},
    thread,
    fs::File,
    io::{Write, BufReader},
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
struct Cue {
    id: u32,
    actions: CueAction
}

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
enum CueAction {
    Play {
        file: String,
        fade_in: Option<u32>,
        fade_out: Option<u32>,
    },
    Stop {
        cue_id: u32,
        fade_out: Option<u32>,
    },
}

#[derive(Serialize, Deserialize)]
struct CueManager {
    cues: HashMap<u32, Cue>,
}

impl CueManager {
    fn new() -> Self {
        CueManager {
            cues: HashMap::new(),
        }
    }

    fn define_cue(&mut self, cue: Cue) {
    self.cues.insert(cue.id, cue);
    }

    fn remove_cue(&mut self, cue_id: u32) {
        self.cues.remove(&cue_id);
    }
    fn get_cue(&self, cue_id: u32) -> Option<&Cue> {
        self.cues.get(&cue_id)
    }
    fn stop_cue() {
        // Find cue process and kill it
    }

    fn save_to_file(&self, path: &str) -> std::io::Result<()> {
        let serialized = serde_json::to_string(&self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
    fn load_from_file(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let manager = serde_json::from_reader(reader)?;
        Ok(manager)
    }
}

struct AppState {
    cue_manager: Arc<Mutex<CueManager>>,
    running_cues: Arc<Mutex<HashMap<u32, Child>>>,
}

fn execute_cue(cue:Cue) {
    match cue.actions {
        CueAction::Play { file, fade_in, fade_out } => {
            let mut cmd = Command::new("play");
            cmd.arg(file);

            if let Some(fade_in_ms) = fade_in {
                cmd.arg("--fade-in").arg(format!("{}ms", fade_in_ms));
            }
            if let Some(fade_out_ms) = fade_out {
                cmd.arg("--fade-out").arg(format!("{}ms", fade_out_ms));
            }
            if let Err(e) = cmd.spawn() {
                eprintln!("Failed to execute cue {}: {}", cue.id, e);
            }
        },
    }
}

fn execute_cue_async(cue: Cue, state: Arc<AppState>) {
    let cue_id = cue.id;

    let child = thread::spawn(move || {
        match cue.actions {
            CueAction::Play { file, fade_in, fade_out } => {
                let mut cmd = Command::new("play");
                cmd.arg(file);

                if let Some(fade_in_ms) = fade_in {
                    cmd.arg("--fade-in").arg(format!("{}ms", fade_in_ms));
                }
                if let Some(fade_out_ms) = fade_out {
                    cmd.arg("--fade-out").arg(format!("{}ms", fade_out_ms));
                }
                if let Err(e) = cmd.spawn() {
                    eprintln!("Failed to execute cue {}: {}", cue.id, e);
                }
                cmd.spawn()
            },
            CueAction::Stop {cue_id, fade_out} => {
            },
        }
    });
}

fn handle_command(command: String, state: &AppState) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    let mut cue_manager = state.cue_manager.lock().expect("Failed to acquire lock for Cue Manager");
    match parts[0] {
        "define" => {
            handle_define_command(&parts, &mut cue_manager);
        },
        "remove" => {
            let cue_id = parts[1].parse::<u32>().unwrap();
            cue_manager.remove_cue(cue_id);
        },
        "go" => {
        },
        "goto" => {
        },
        "stop" => {
            let cue_id = parts[1].parse::<u32>().unwrap();
        },
        "exit" | "quit" => {
            std::process::exit(0);
        },
        _ => {
            if let Ok(cue_id) = parts[0].parse::<u32>() {
                ///
            } else {
                ///
            } println!("Unknown command: {}", parts[0]);
        }
    }
}

fn handle_define_command(args: &[&str], cue_manager: &mut CueManager) {
    let mut cue_id = None;
    let mut action = None;

    let mut i = 0;
    if args.len() < 5 {
        println!("Incorrect usage: define --cue <num> --play <file>");
    }
    while i < args.len() {
        match args[i] {
            "--cue" => {
                i+=1;
                if i < args.len() {
                    cue_id = args[i].parse::<u32>().ok();
                }
            },
            "--play" => {
                i+=1;
                if i < args.len() {
                    let file = args[i].to_string();
                    action = Some(CueAction::Play { file, fade_in: None, fade_out: None })
                }
            },
            _ => {},
        }
        i+=1;
    }
    if let (Some(id), Some(act)) = (cue_id, action) {
        let cue = Cue { id, actions: act };
        cue_manager.define_cue(cue);
        println!("Defined cue {}", id);
    } else {
        println!("Invalid define command");
    }
}

fn handle_go_command(cue_manager: &CueManager) {
    let mut cue_ids: Vec<u32> = cue_manager.cues.keys().cloned().collect();
    cue_ids.sort();

    for cue_id in cue_ids {
        if let Some(cue) = cue_manager.get_cue(cue_id) {
            execute_cue_async(cue.clone());
        }
    }
}

fn handle_goto_command(cue_id: u32, cue_manager: &CueManager) {
    if let Some(cue) = cue_manager.get_cue(cue_id) {
        execute_cue_async(cue.clone());
    } else {
        println!("Cue {} not found", cue_id);
    }
}


fn main() -> rustyline::Result<()> {
    let state = AppState {
        cue_manager: Arc::new(Mutex::new(CueManager::new())),
    };

    let mut rl = Editor::<(), FileHistory>::new()?;
    println!("cueplayer started. Type 'help' for a list of commands");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                handle_command(line, &state);
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted!");
            }
            Err(ReadlineError::Eof) => {
                println!("EOF!");
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }

        }
    }

}

