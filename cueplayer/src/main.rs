use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use serde::{Serialize, Deserialize};
use std::{
    collections::HashMap,
    process::Command,
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

#[derive(Serialize, Deserialize, Debug)]
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

    fn _save_to_file(&self, path: &str) -> std::io::Result<()> {
        let serialized = serde_json::to_string(&self)?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
    fn _load_from_file(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let manager = serde_json::from_reader(reader)?;
        Ok(manager)
    }
}

struct AppState {
    cue_manager: Arc<Mutex<CueManager>>,
    running_cues: Arc<Mutex<HashMap<u32, Arc<Mutex<std::process::Child>>>>>,
}

fn execute_cue_process(cue: Cue, state: &AppState) {
    match cue.actions {
        CueAction::Play{ file, fade_in, fade_out } => {
            let mut cmd = Command::new("play");
            println!("Arrived here: {:?}", cmd);
            cmd.arg(file);
            if let Some(fade_in_ms) = fade_in {
                cmd.arg("--fade-in").arg(format!("{}ms", fade_in_ms));
            };
            if let Some(fade_out_ms) = fade_out {
                cmd.arg("--fade-out").arg(format!("{}ms", fade_out_ms));
            };
            match cmd.spawn() {
                Ok(child) => {
                    let cue_id = cue.id;
                    let child_arc = Arc::new(Mutex::new(child));
                    {
                        let mut exec_cues = state.running_cues.lock().unwrap();
                        exec_cues.insert(cue_id, child_arc.clone());
                    }
                    println!("Cue {} is playing", cue_id);

                    let exec_cues = Arc::clone(&state.running_cues);
                    thread::spawn(move || {
                        {
                            let mut child = child_arc.lock().unwrap();
                            let _ = child.wait();
                        }
                        let mut exec_cues = exec_cues.lock().unwrap();
                        exec_cues.remove(&cue_id);
                        println!("Cue {} has finished", cue_id);
                     });
                },
                Err(e) => {
                    eprintln!("Failed to execute cue {}: {}", cue.id, e);
                }
            }
        },
        CueAction::Stop{ cue_id: _, fade_out: _ } => {
            todo!();
        },
    }
}

fn handle_command(command: String, state: &AppState) {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    match parts[0] {
        "define" => {
            let mut cue_manager = state.cue_manager.lock().expect("Failed to acquire lock for Cue Manager");
            handle_define_command(&parts, &mut cue_manager);
        },
        "show" => {
            let cue_manager = state.cue_manager.lock().expect("Failed to acquire lock for Cue Manager");
            let cue_list = cue_manager.cues.clone();
            println!("{:?}", cue_list);

        }
        "remove" => {
            if parts.len() < 2 {
                println!("Usage: remove <cue_id>");
            } else if let Ok(cue_id) = parts[1].parse::<u32>() {
                let mut cue_manager = state.cue_manager.lock().expect("Failed to acquire lock for Cue Manager");
                cue_manager.remove_cue(cue_id);
            } else {
                println!("Invalid cue ID");
            }
        },
        "go" => {
            println!("going!");
            handle_go_command(&state);
        },
        "goto" => {
            if parts.len() < 2 {
                println!("Usage: goto <cue_id>");
            } else if let Ok(cue_id) = parts[1].parse::<u32>() {
                handle_goto_command(cue_id, &state);
            } else {
                println!("Invalid cue ID");
            }
        },
        "stop" => {
            if parts.len() < 2 {
                println!("Usage: stop <cue_id>");
            } else if let Ok(cue_id) = parts[1].parse::<u32>() {
                handle_stop_cue(cue_id, &state);
            } else {
                println!("Invalid cue ID");
            }
        },
        "exit" | "quit" => {
            std::process::exit(0);
        },
        _ => {
            if let Ok(cue_id) = parts[0].parse::<u32>() {
                handle_goto_command(cue_id, &state);
            } else {
             println!("Unknown command: {}", parts[0]);
            }
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
            "cue" => {
                i+=1;
                if i < args.len() {
                    cue_id = args[i].parse::<u32>().ok();
                }
            },
            "play" => {
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

fn handle_go_command(state: &AppState) {
    let cue_manager = state.cue_manager.lock().unwrap();
    println!("Successful lock of cue_manager: {:?}", cue_manager);
    let mut cue_ids: Vec<u32> = cue_manager.cues.keys().cloned().collect();
    println!("Cue ids: {:?}", cue_ids);
    cue_ids.sort();
    println!("Cue ids sort: {:?}", cue_ids);
    for cue_id in cue_ids {
        println!("Starting cue {}", cue_id);
        if let Some(cue) = cue_manager.get_cue(cue_id) {
            execute_cue_process(cue.clone(), state);
        }
    }
}

fn handle_goto_command(cue_id: u32, state: &AppState) {
    let cue_manager = state.cue_manager.lock().unwrap();
    if let Some(cue) = cue_manager.get_cue(cue_id) {
        execute_cue_process(cue.clone(), state);
    } else {
        println!("Cue {} not found", cue_id);
    }
}

fn handle_stop_cue(cue_id: u32, state: &AppState) {
    let mut exec_cues = state.running_cues.lock().unwrap();
    if let Some(child_arc) = exec_cues.clone().get(&cue_id) {
        let mut child = child_arc.lock().unwrap();
        match child.kill() {
            Ok(_) => {
                println!("Cue {} stopped", cue_id);
                exec_cues.remove(&cue_id);
            },
            Err(e) => {
                eprintln!("Failed to stop cue {}: {}", cue_id, e);
            }
        }
    } else {
        println!("Cue {} is not currently playing", cue_id);
    }

}


fn main() -> rustyline::Result<()> {
    let state = AppState {
        cue_manager: Arc::new(Mutex::new(CueManager::new())),
        running_cues: Arc::new(Mutex::new(HashMap::new())),
    };

    let mut rl = Editor::<(), FileHistory>::new()?;
    println!("cueplayer started. Type 'help' for a list of commands");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
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

