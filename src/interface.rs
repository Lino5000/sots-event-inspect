use inquire::{Select, InquireError};
use strum::{ IntoEnumIterator, EnumIter };

use crate::{
    data::Event,
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

#[derive(PartialEq, Eq, Debug, Clone, EnumIter)]
enum Command {
    ViewEvent,
    Quit,
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Command::*;
        write!(f, "{}", match self {
            ViewEvent => "view event",
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
            "quit" => Quit,
            _ => { return Err(format!("Unknown command `{}`", s).into()); }
        })
    }
}

#[derive(Debug)]
pub struct App {
    event_map: BTreeMap<String, Event>,
    running: bool,
}

impl App {
    pub fn new(args: Args) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            event_map: parse_event_data(args.path)?,
            running: true,
        })
    }

    fn run_command(&mut self, cmd: Command) -> Result<(), CommandError> {
        match cmd {
            Command::ViewEvent => {
                let id: &str = Select::new("Event id: ", self.event_map.keys().collect())
                    .prompt()?;
                let Some(event) = self.event_map.get(id) else {
                    return Err("Select somehow returned an invalid event id.".into());
                };
                println!("{}", event);
            }
            Command::Quit => { self.running = false; }
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
