use inquire::{Select, InquireError, Confirm};
use strum::{ IntoEnumIterator, EnumIter };
use walkdir::WalkDir;
use bimap::BiBTreeMap;

use crate::{
    data::{ RawEvent, NPC, write_vec_sep },
    yaml::{ constrain_field_get_body, Field, YamlError },
    field_get, field_get_body, field_value_type, Args, 
};

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::Display, 
    fs::File,
    path::PathBuf, 
    str::FromStr, 
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

#[derive(PartialEq, Eq, Debug, Clone, EnumIter)]
enum Command {
    ViewNPC,
    ViewEvent,
    Quit,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        write!(f, "{}", match self {
            ViewEvent => "view event",
            ViewNPC => "view npc",
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
            "view npc" => ViewNPC,
            "quit" => Quit,
            _ => { return Err(format!("Unknown command `{}`", s).into()); }
        })
    }
}

impl Command {
    fn run(self, app: &mut App) -> Result<(), CommandError> {
        use Command::*;
        match self {
            ViewEvent => {
                let id: &str = Select::new("Event id:", app.event_map.keys().collect())
                    .prompt()?;
                app.state = AppState::Event { id: id.to_owned() };
            }
            ViewNPC => {
                let npc_id: &str = Select::new("NPC Id:", app.npc_guids.right_values().collect())
                    .prompt()?;
                app.state = AppState::NPC { id: npc_id.to_owned() };
            }
            Quit => { app.state = AppState::Quit; }
        };
        Ok(())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, EnumIter)]
enum DeckSubCommand {
    Deck1,
    Deck2,
    Deck3,
    Deck4,
    Deck5,
    AllDecks,
    FallbackDeck,
}

impl DeckSubCommand {
    fn cycle(&self) -> Option<usize> {
        use DeckSubCommand::*;
        match self {
            Deck1 => Some(1),
            Deck2 => Some(2),
            Deck3 => Some(3),
            Deck4 => Some(4),
            Deck5 => Some(5),
            FallbackDeck => None,
            AllDecks => None,
        }
    }

    fn run(self, npc: &NPC) -> Result<(), CommandError> {
        if self == DeckSubCommand::AllDecks {
            npc.print_all_decks();
        } else if self == DeckSubCommand::FallbackDeck {
            npc.print_fallback_deck();
        } else {
            npc.print_deck(self.cycle().expect("variant with specific cycle number"));
        }
        Ok(())
    }
}

impl Display for DeckSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use DeckSubCommand::*;
        write!(f, "{}", match *self {
            Deck1 => "1",
            Deck2 => "2",
            Deck3 => "3",
            Deck4 => "4",
            Deck5 => "5",
            FallbackDeck => "fallback",
            AllDecks => "all",
        })
    }
}

impl FromStr for DeckSubCommand {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DeckSubCommand::*;
        Ok(
            match s.to_lowercase().as_str() {
                "1" => Deck1,
                "2" => Deck2,
                "3" => Deck3,
                "4" => Deck4,
                "5" => Deck5,
                "fallback" => FallbackDeck,
                "all" => AllDecks,
                _ => { return Err("Unknown command".into()); }
            }
        )
    }
}

#[derive(PartialEq, Eq, Debug, Clone, EnumIter)]
enum NPCSubCommand {
    ViewEvents,
    ViewDecks,
    Back,
}

impl Display for NPCSubCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NPCSubCommand::*;
        write!(f, "{}", match self {
            ViewEvents => "events",
            ViewDecks => "decks",
            Back => "back",
        })
    }
}

impl FromStr for NPCSubCommand {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use NPCSubCommand::*;
        Ok(match s.to_lowercase().as_str() {
            "events" => ViewEvents,
            "decks" => ViewDecks,
            "back" => Back,
            _ => { return Err(format!("Unknown command `{}`", s).into()); }
        })
    }
}

impl NPCSubCommand {
    fn run(self, app_state: &mut AppState, npc: &NPC) -> Result<(), CommandError> {
        use NPCSubCommand::*;
        match self {
            ViewEvents => {
                *app_state = AppState::NPCEvents { npc_id: npc.id.clone() };
            },
            ViewDecks => { 
                let sub_cmd = Select::new("Which cycle do you want the deck for?", DeckSubCommand::iter().collect())
                    .prompt()?;
                sub_cmd.run(&npc)?;
            },
            Back => {
                *app_state = AppState::Root;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Event {
    npc_id: String,
    event: RawEvent,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.event.id)?;
        writeln!(f, "\tNPC: {}", self.npc_id)?;
        writeln!(f, "\tNum Concord: {}", self.event.sequence_count)?;
        writeln!(f, "\tNum Discord: {}", self.event.strike_count)?;
        write!(f, "\tSequence Lengths: ")?;
        write_vec_sep(&self.event.sequence_lengths, ", ", f)?;
        if let Some(deck) = &self.event.deck {
            writeln!(f, "\n\tOverrides NPC deck with:")?;
            writeln!(f, "{}", deck)
        } else {
            writeln!(f, "\n\tUses default deck for this cycle; see NPC data.")
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AppState {
    Root,
    Event { id: String },
    NPC { id: String },
    NPCEvents { npc_id: String },
    Quit,
}

#[derive(Debug)]
pub struct App {
    event_map: BTreeMap<String, Event>,
    npc_map: BTreeMap<String, NPC>,
    npc_guids: BiBTreeMap<String, String>, // (Guid, NPC id)
    npc_events: BTreeMap<String, BTreeSet<String>>, // (NPC id, Set of Event ids)
    state: AppState,
}

impl App {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        let mut out = Self {
            event_map: BTreeMap::new(),
            npc_map: BTreeMap::new(),
            npc_guids: BiBTreeMap::new(),
            npc_events: BTreeMap::new(),
            state: AppState::Root,
        };

        out.build_npc_maps(&args.path)?;
        out.parse_event_data(args.path)?;
        
        Ok(out)
    }

    fn is_running(&self) -> bool {
        self.state != AppState::Quit
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while self.is_running() {
            use AppState::*;
            match &self.state {
                Root => {
                    let cmd: Command = Select::new("What would you like to do?", Command::iter().collect())
                        .prompt()?;
                    cmd.run(self)?;
                },
                Event { id } => {
                    let Some(event) = self.event_map.get(id) else {
                        return Err("Select somehow returned an invalid event id.".into());
                    };
                    println!("Event - {}", event);
                    self.state = Root;
                },
                NPC { id } => {
                    let Some(guid) = self.npc_guids.get_by_right(id) else {
                        return Err("Select somehow returned an invalid NPC Id.".into());
                    };
                    let Some(npc) = self.npc_map.get(guid) else {
                        return Err("NPC Id was mapped to an invalid NPC GUID.".into());
                    };
                    npc.print_details();
                    let sub_cmd = Select::new(&format!("What would you like to know about {}?", npc.id), NPCSubCommand::iter().collect())
                        .prompt()?;
                    sub_cmd.run(&mut self.state, npc)?;
                },
                NPCEvents { npc_id } => {
                    let Some(event_ids) = self.npc_events.get(npc_id) else {
                        return Err("Somehow ended up with an invalid NPC Id.".into());
                    };
                    println!("{} has the following events:", npc_id);
                    event_ids.iter().for_each(|e| {
                        println!("\t{}", e);
                    });
                    let inspect = Confirm::new("Would you like to inspect one of these events?").prompt()?;
                    if inspect {
                        let mut options = event_ids.clone();
                        options.insert("cancel".to_owned());
                        let event_id = Select::new("Which event would you like to inspect?", options.iter().collect())
                            .prompt()?;
                        if let Some(event) = self.event_map.get(event_id) {
                            println!("Event - {}", event);
                        } else if event_id == "cancel" {
                            println!("Cancelled.");
                        } else {
                            return Err("Select somehow returned an invalid event id.".into());
                        };
                    }
                    self.state = NPC { id: npc_id.clone() };
                },
                Quit => { unreachable!("Loop should end as soon as we enter the AppState::Quit state"); }
            }
        }
        Ok(())
    }

    fn parse_event_data(&mut self, mut folder_path: PathBuf) -> Result<(), Box<dyn Error>> {
        folder_path.push("event_data.asset");
        let file = File::open(folder_path)?;
        let yaml: Field = serde_yaml::from_reader(file)?;
        
        let Field::Struct(data_map) = yaml else { return Err("Root isn't a map".into()); };
        let ref_data_map = &data_map;
        field_get!(let monobehaviour: Struct = ref_data_map.MonoBehaviour);
        field_get!(let events: List = monobehaviour.data);


        self.event_map = events.iter()
            .map(|field| {
                let raw = RawEvent::try_from(field)?;
                if let Some(npc_id) = self.npc_guids.get_by_left(&raw.npc_guid) {
                    // Insert to relevant npc_events set
                    let Some(event_set) = self.npc_events.get_mut(npc_id) else {
                        return Err(format!("NPC {} somehow wasn't added to the npc_events map", npc_id).into());
                    };
                    event_set.insert(raw.id.clone());

                    // Create the actual Event struct for the event_map
                    Ok((raw.id.clone(), Event {
                        npc_id: npc_id.clone(),
                        event: raw
                    }))
                } else {
                    Err(format!("Unknown NPC Guid `{}` in event `{}`", raw.npc_guid, raw.id).into())
                }
            })
            .collect::<Result<_, YamlError>>()?;

        Ok(())
    }

    fn build_npc_maps(&mut self, folder_path: &PathBuf) -> Result<(), Box<dyn Error>> {
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
                self.npc_guids.insert(guid.clone(), npc.id.clone());
                self.npc_events.insert(npc.id.clone(), BTreeSet::new());
                self.npc_map.insert(guid.clone(), npc);
            }
        }

        Ok(())
    }
}
