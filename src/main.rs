use std::collections::{ BTreeMap, BTreeSet };
use std::fmt::Display;
use std::fs::File;
use std::num::TryFromIntError;
use std::ops::{ Deref, DerefMut };
use std::{path::PathBuf, error::Error};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File Path to extract from
    path: PathBuf,
}

#[derive(Debug)]
enum Field {
    Struct(BTreeMap<String, Field>),
    List(Vec<Field>),
    Bool(bool),
    Uint(u64),
    Int(i64),
    Float(f64),
    Null,
    Str(String),
}

struct FieldVisitor;

impl<'de> serde::de::Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid Unity yaml field")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut map_struct = BTreeMap::new();
        while let Some((key, value)) = map.next_entry()? {
            map_struct.insert(key, value);
        }
        Ok(Field::Struct(map_struct))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, 
    {
        let mut vec_list = Vec::new();
        while let Some(value) = seq.next_element::<Field>()? {
            vec_list.push(value);
        }
        Ok(Field::List(vec_list))
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Bool(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Float(v))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Null)
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Uint(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Int(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Str(v.to_owned()))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
    {
        Ok(Field::Str(v.to_owned()))
    }
}

impl<'de> serde::Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>
    {
        deserializer.deserialize_any(FieldVisitor)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Effect {
    None,
    Chain,
    Inherit,
    Duplicate,
    Insert,
    Collapse,
    Redraw,
    ViewHand,
    Choose,
    Listen,
}

impl TryFrom<&Field> for Effect {
    type Error = YamlError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let Field::Uint(id) = value else { return Err("Field is not a Uint".into()); };
        Ok(match id {
            1_u64 => Effect::Chain,
            2_u64 => Effect::Inherit,
            3_u64 => Effect::Duplicate,
            4_u64 => Effect::Insert,
            5_u64 => Effect::Collapse,
            6_u64 => Effect::Redraw,
            7_u64 => Effect::ViewHand,
            8_u64 => Effect::Choose,
            9_u64 => Effect::Listen,
            _ => Effect::None,
        })
    }
}

impl Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Effect::None => "",
            Effect::Chain => "Chatter", 
            Effect::Inherit => "Elaborate",
            Effect::Duplicate => "Accommodate",
            Effect::Insert => "Clarify",
            Effect::Collapse => "Backtrack",
            Effect::Redraw => "Reconsider",
            Effect::ViewHand => "Observe",
            Effect::Choose => "Prepare",
            Effect::Listen => "Listen",
        })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
enum ConnectType {
    Circle,
    Triangle,
    Square,
    Diamond,
    Dog,
    Spiral,
}

impl Display for ConnectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Connector(BTreeSet<ConnectType>);

impl Deref for Connector {
    type Target = BTreeSet<ConnectType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Connector {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Connector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "")
        } else {
            let mut iter = self.iter();
            while let Some(c) = iter.next() {
                write!(f, "{}", c)?;
                if iter.len() != 0 {
                    write!(f, ", ")?;
                }
            }
            Ok(())
        }
    }
}

impl TryFrom<&Field> for Connector {
    type Error = YamlError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let mut set = BTreeSet::new();
        
        let Field::Uint(connect) = value else { return Err("Connector is not a Uint field".into()); };

        if (connect & 0x1) > 0 {
            set.insert(ConnectType::Circle);
        }
        if (connect & 0x2) > 0 {
            set.insert(ConnectType::Triangle);
        }
        if (connect & 0x4) > 0 {
            set.insert(ConnectType::Square);
        }
        if (connect & 0x8) > 0 {
            set.insert(ConnectType::Diamond);
        }
        if (connect & 0x10) > 0 {
            set.insert(ConnectType::Spiral);
        }
        if (connect & 0x20) > 0 {
            set.insert(ConnectType::Dog);
        }

        Ok(Self(set))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Card {
    input: Connector,
    output: Connector,
    effect: Effect,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} | {} | {}", self.input, self.effect, self.output))
    }
}

impl TryFrom<&Field> for Card {
    type Error = YamlError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let Field::Struct(card_field) = value else { return Err("Not a Struct field".into()); };
        
        Ok(Self {
            input: Connector::try_from(
                       if let Some(input) = card_field.get("input") {
                           input
                       } else {
                           return Err("No `input` field in card".into());
                       }
                    )?,
            output: Connector::try_from(
                       if let Some(output) = card_field.get("output") {
                           output
                       } else {
                           return Err("No `output` field in card".into());
                       }
                    )?,
            effect: Effect::try_from(
                        if let Some(effect) = card_field.get("effect") {
                            effect
                        } else {
                            return Err("No `effect` field in card".into());
                        }
                    )?,
        })
    }
}


#[derive(Debug, PartialEq, Eq)]
struct Event {
    id: String,
    npc_guid: String,
    sequence_count: u8,
    strike_count: u8,
    sequence_lengths: Vec<u8>,
    deck: Option<Vec<Card>>
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

macro_rules! field_value_type {
    (Uint) => { u64 };
    (Struct) => { std::collections::BTreeMap<String, Field> };
    (List) => { Vec<Field> };
    (Bool) => { bool };
    (Uint) => { u64 };
    (Int) => { i64 };
    (Float) => { f64 };
    (Str) => { String };
}

fn constrain_field_get_body<F, R>(f: F) -> F
    where
        F: for<'a> Fn(Option<&String>, &'a std::collections::BTreeMap<String, Field>) -> Result<&'a R, YamlError>
{
    f
}

macro_rules! field_get_body {
    ($var:ident, $t:tt, $key:tt) => {
            constrain_field_get_body::<_, field_value_type!($t)>(|event_id, map| {
                let key: &str = stringify!($key);
                let Some(field) = map.get(key.clone()) else {
                    println!("{:?}", map);
                    return Err(format!("{}Field didn't contain `{}` key.",
                               if let Some(id) = event_id {
                                   format!("event {}: ", id)
                               } else { "".to_owned() },
                               key).into());
                };
                let Field::$t($var) = field else {
                    println!("{:?}", field);
                    return Err(format!("{}Field entry `{}` is not of type {}.",
                               if let Some(id) = event_id {
                                   format!("event {}: ", id)
                               } else { "".to_owned() },
                               key,
                               stringify!($t)).into());
                };
                Ok($var)
            })
    };
}

macro_rules! field_get {
    (let $var:ident: $t:tt = $map:ident.$key:tt) => {
        let $var = {
            let get = field_get_body!($var, $t, $key);
            get(None, $map)
        }?;
    };
    (event $id:expr, let $var:ident: $t:tt = $map:ident.$key:tt) => {
        let $var = {
            let get = field_get_body!($var, $t, $key);
            get(Some($id), $map)
        }?;
    };
}

impl TryFrom<&Field> for Event {
    type Error = YamlError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let Field::Struct(event) = value else { return Err("Field is not a Struct".into()); };
            //let Some(Field::Str(id)) = event.get("id") else { return Err("Couldn't retrieve `id` for an event".into()); };
            //let Some(Field::Uint(sequence)) = event.get("sequence") else { return Err(format!("Couldn't access the `sequence` field of event `{}`", id).into()); };
            //let Some(Field::Uint(seq_count)) = event.get("seq_count") else { return Err(format!("Couldn't access the `seq_count` field of event `{}`", id).into()); };
            //let Some(Field::Uint(strike_count)) = event.get("strike_count") else { return Err(format!("Couldn't access the `strike_count` field of event `{}`", id).into()); };
            //let Some(Field::Uint(override_deck)) = event.get("override_deck") else { return Err(format!("Couldn't access the `override_deck` field of event `{}`", id).into()); };
            //let Some(Field::Struct(npc_data)) = event.get("npc") else { return Err(format!("Couldn't access the `npc` field of event `{}`", id).into()); };
            //let Some(Field::Str(npc_guid)) = npc_data.get("guid") else { return Err(format!("Couldn't access the `guid` field of event `{}`", id).into()); };
        field_get!(let id: Str = event.id);
        field_get!(let sequence: Str = event.sequence);
        field_get!(event id, let seq_count: Uint = event.sequenceCount);
        field_get!(event id, let strike_count: Uint = event.strikeCount);
        field_get!(event id, let override_deck: Uint = event.overrideDeck);
        field_get!(event id, let npc_data: Struct = event.npc);
        field_get!(event id, let npc_guid: Str = npc_data.guid);
        //println!("{}", sequence);

        let sequence_lengths: Vec<u8> = sequence.split("")
                    .skip(2)
                    .step_by(8)
                    .flat_map(|s| {
                        u8::from_str_radix(s, 10)
                    })
                    .collect();

        if sequence_lengths.len() as u64 != *seq_count {
            return Err(format!("{}: Failed to parse `sequence` field.", id).into());
        }

        Ok(Self {
            id: id.clone(),
            npc_guid: npc_guid.clone(),
            sequence_count: u8::try_from(*seq_count)?,
            strike_count: u8::try_from(*strike_count)?,
            sequence_lengths,
            deck: if *override_deck == 1u64 {
                field_get!(event id, let event_deck: Struct = event.deck);
                field_get!(event id, let cards: List = event_deck.cards);

                let mut deck = Vec::new();
                for card in cards {
                    deck.push(card.try_into()?);
                }

                Some(deck)
            } else {
                None
            }
        })
    }
}

fn write_vec_sep<T: Display>(v: &Vec<T>, sep: &str, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut iter = v.iter();
    while let Some(el) = iter.next() {
        write!(f, "{}", el)?;
        if iter.len() != 0 {
            write!(f, "{}", sep)?;
        }
    }
    Ok(())
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.id)?;
        writeln!(f, "\tnpc_guid: {}", self.npc_guid)?;
        writeln!(f, "\tsequence_count: {}", self.sequence_count)?;
        writeln!(f, "\tstrike_count: {}", self.strike_count)?;
        write!(f, "\tsequence_lengths: ")?;
        write_vec_sep(&self.sequence_lengths, ", ", f)?;
        write!(f, "\n\tdeck:\n\t\t")?;
        if let Some(deck) = &self.deck {
            write_vec_sep(&deck, "\n\t\t", f)?;
            writeln!(f, "")
        } else {
            writeln!(f, "Default for cycle; see character with guid `{}`", self.npc_guid)
        }
    }
}

#[derive(Debug)]
struct YamlError(String);

impl YamlError {
    fn new(msg: &str) -> Self {
        Self(msg.to_owned())
    }
}

impl Display for YamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for YamlError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl From<&str> for YamlError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for YamlError {
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

macro_rules! impl_yamlerror_from_error {
    ($t:ty) => {
        impl From<$t> for YamlError {
            fn from(value: $t) -> Self {
                Self::new(&value.to_string())
            }
        }
    };
}
impl_yamlerror_from_error!(TryFromIntError);
//impl_yamlerror_from_error!();

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.path.is_file() {
        let file = File::open(args.path)?;
        let yaml: Field = serde_yaml::from_reader(file)?;
        
        let Field::Struct(data_map) = yaml else { return Err("Root isn't a map".into()); };
        let ref_data_map = &data_map;
        field_get!(let monobehaviour: Struct = ref_data_map.MonoBehaviour);
        field_get!(let events: List = monobehaviour.data);
        for event_field in events {
            println!("{}", Event::try_from(event_field)?);
        }
    } else if args.path.is_dir() {
        return Err("Directories are not yet supported".into());
    } else if !args.path.try_exists()? {
        return Err(format!("The file `{}` does not exist.", args.path.display()).into());
    } else {
        return Err("An unknown error occured".into());
    }

    Ok(())
}
