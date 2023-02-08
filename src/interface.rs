use inquire::{Select, InquireError};
use strum::{ IntoEnumIterator, EnumIter };
use walkdir::{ WalkDir, DirEntry };

use crate::{
    data::{Event, NPC},
    yaml::{ constrain_field_get_body, Field, YamlError },
    field_get, field_get_body, field_value_type, Args, 
};

use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    path::PathBuf, fmt::Display, str::FromStr,
};

#[derive(Debug, Clone)]
struct CommandError(String);

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <String as Display>::fmt(&self.0, f)
    }
}

impl Error for CommandError {}

impl From<String> for CommandError {
    fn from(value: String) -> Self {
        CommandError(value)
    }
}

impl From<&str> for CommandError {
    fn from(value: &str) -> Self {
        CommandError(value.to_owned())
    }
}

impl From<InquireError> for CommandError {
    fn from(value: InquireError) -> Self {
        value.to_string().into()
    }
}

fn parse_event_data(mut folder_path: PathBuf) -> Result<BTreeMap<String, Event>, Box<dyn Error>> {
    folder_path.push("event_data.asset");
    let file = File::open(folder_path)?;
    let yaml: Field = serde_yaml::from_reader(file)?;
    
    let Field::Struct(data_map) = yaml else { return Err("Root isn't a map".into()); };
    let ref_data_map = &data_map;
    field_get!(let monobehaviour: Struct = ref_data_map.MonoBehaviour);
    field_get!(let events: List = monobehaviour.data);

    let vec = events.iter()
        .map(|field| Event::try_from(field))
        .collect::<Result<Vec<Event>, YamlError>>()?;

    Ok(vec.into_iter()
       .map(|e| (e.id.clone(), e))
       .collect())
}

fn build_npc_map(folder_path: &PathBuf) -> Result<BTreeMap<String, NPC>, Box<dyn Error>> {
    let mut map = BTreeMap::new();

    let meta_files =
        WalkDir::new(folder_path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|entry| {
            entry.file_name().to_str().map_or(false, |name| name.ends_with(".meta"))
        })
        .flatten(); // Silently skip permission errors

    for meta_file in meta_files {
        let meta_path = meta_file.into_path();
        let asset_path = meta_path.with_extension("");

        let meta_yaml: Field = serde_yaml::from_reader(File::open(meta_path)?)?;
        let Field::Struct(meta_map) = meta_yaml else { return Err("Root isn't a map".into()); };
        let ref_meta_map = &meta_map;
        field_get!(let guid: Str = ref_meta_map.guid);

        if let Some(npc) = NPC::load_asset(asset_path)? {
            map.insert(guid.clone(), npc);
        }
    }

    Ok(map)
}

#[derive(PartialEq, Eq, Debug, Clone, EnumIter)]
enum Command {
    ViewEvent,
    ViewNPCGuid,
    Quit,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        write!(f, "{}", match self {
            ViewEvent => "view event",
            ViewNPCGuid => "view npc for guid",
            Quit => "quit",
        })
    }
}

impl FromStr for Command {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Command::*;
        Ok(match s.to_lowercase().as_str() {
            "view event" => ViewEvent,
            "view npc for guid" => ViewNPCGuid,
            "quit" => Quit,
            _ => { return Err(format!("Unknown command `{}`", s).into()); }
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone, EnumIter)]
enum NPCSubCommand {
    Deck0,
    Deck1,
    Deck2,
    Deck3,
    Deck4,
    Deck5,
    AllDecks,
}

impl NPCSubCommand {
    fn cycle(&self) -> Option<usize> {
        match self {
            Deck0 => Some(0),
            Deck1 => Some(1),
            Deck2 => Some(2),
            Deck3 => Some(3),
            Deck4 => Some(4),
            Deck5 => Some(5),
            AllDecks => None,
        }
    }
}

impl Display for NPCSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NPCSubCommand::*;
        write!(f, "{}", match *self {
            Deck0 => "0",
            Deck1 => "1",
            Deck2 => "2",
            Deck3 => "3",
            Deck4 => "4",
            Deck5 => "5",
            AllDecks => "all",
        })
    }
}

impl FromStr for NPCSubCommand {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use NPCSubCommand::*;
        Ok(
            match s.to_lowercase().as_str() {
                "0" => Deck0,
                "1" => Deck1,
                "2" => Deck2,
                "3" => Deck3,
                "4" => Deck4,
                "5" => Deck5,
                "all" => AllDecks,
                _ => { return Err("Unknown command".into()); }
            }
        )
    }
}

#[derive(Debug)]
pub struct App {
    event_map: BTreeMap<String, Event>,
    npc_map: BTreeMap<String, NPC>,
    running: bool,
}

impl App {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            event_map: parse_event_data(args.path.clone())?,
            npc_map: build_npc_map(&args.path)?,
            running: true,
        })
    }

    fn run_command(&mut self, cmd: Command) -> Result<(), CommandError> {
        use Command::*;
        match cmd {
            ViewEvent => {
                let id: &str = Select::new("Event id: ", self.event_map.keys().collect())
                    .prompt()?;
                let Some(event) = self.event_map.get(id) else {
                    return Err("Select somehow returned an invalid event id.".into());
                };
                println!("{}", event);
            }
            ViewNPCGuid => {
                let guid: &str = Select::new("NPC Guid: ", self.npc_map.keys().collect())
                    .prompt()?;
                let Some(npc) = self.npc_map.get(guid) else {
                    return Err("Select somehow returned an invalid npc guid.".into());
                };
                npc.print_details();
                let sub_cmd = Select::new("Which cycle do you want? ", NPCSubCommand::iter().collect())
                    .prompt()?;
                use NPCSubCommand::*;
                if sub_cmd == AllDecks {
                    npc.print_all_decks();
                } else {
                    npc.print_deck(sub_cmd.cycle().expect("Only AllDecks variant should return None"));
                }
            }
            Quit => { self.running = false; }
        };
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while self.is_running() {
            let cmd: Command = Select::new("", Command::iter().collect())
                .prompt()?;
            self.run_command(cmd)?;
        }
        Ok(())
    }
}
