use std::{
    collections::{ BTreeMap, BTreeSet },
    error::Error,
    fmt::Display,
    fs::File,
    ops::{ Deref, DerefMut }, path::PathBuf,
};

use crate::{
    yaml::{
        Field,
        YamlError,
        constrain_field_get_body
    },
    field_get, field_get_body, field_value_type, impl_tryfrom_field
};

#[derive(Debug, PartialEq, Eq)]
pub enum Effect {
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

impl_tryfrom_field!{Uint for Effect:
    |id| {
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
pub enum ConnectType {
    Circle,
    Triangle,
    Square,
    Diamond,
    Dog,
    Spiral,
}

impl Display for ConnectType {
    #[cfg(not(feature = "display_compat"))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //std::fmt::Debug::fmt(&self, f)
        use ConnectType::*;
        write!(f, "{}", 
            match self {
                Circle => "○",
                Triangle => "△",
                Square => "□",
                Diamond => "◊",
                Dog => "🐾",
                Spiral => "@",
            }
        )
    }

    #[cfg(feature = "display_compat")]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //std::fmt::Debug::fmt(&self, f)
        use ConnectType::*;
        write!(f, "{}", 
            match self {
                Circle => "C",
                Triangle => "T",
                Square => "S",
                Diamond => "D",
                Dog => "P",
                Spiral => "@",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connector(BTreeSet<ConnectType>);

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
        for c in self.iter() {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

impl_tryfrom_field!{Uint for Connector:
    |connect| {
        let mut set = BTreeSet::new();

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
pub struct Card {
    input: Connector,
    output: Connector,
    effect: Effect,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.input, self.output)?;
        if self.effect != Effect::None {
            write!(f, " + {}", self.effect)
        } else {
            Ok(())
        }
    }
}

impl_tryfrom_field!{Struct for Card:
    |value| {
        Ok(Self {
            input: Connector::try_from(
                       if let Some(input) = value.get("input") {
                           input
                       } else {
                           return Err("No `input` field in card".into());
                       }
                    )?,
            output: Connector::try_from(
                       if let Some(output) = value.get("output") {
                           output
                       } else {
                           return Err("No `output` field in card".into());
                       }
                    )?,
            effect: Effect::try_from(
                        if let Some(effect) = value.get("effect") {
                            effect
                        } else {
                            return Err("No `effect` field in card".into());
                        }
                    )?,
        })
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct RawEvent {
    pub id: String,
    pub npc_guid: String,
    pub sequence_count: u8,
    pub strike_count: u8,
    pub sequence_lengths: Vec<u8>,
    pub deck: Option<Deck>
}

impl PartialOrd for RawEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RawEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl_tryfrom_field!{Struct for RawEvent:
    |event| {
        field_get!(let id: Str = event.id);
        field_get!(let sequence: Str = event.sequence);
        field_get!(event id, let seq_count: Uint = event.sequenceCount);
        field_get!(event id, let strike_count: Uint = event.strikeCount);
        field_get!(event id, let override_deck: Uint = event.overrideDeck);
        field_get!(event id, let npc_data: Struct = event.npc);
        field_get!(event id, let npc_guid: Str = npc_data.guid);

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
                Some(event_deck.try_into()?)
            } else {
                None
            }
        })
    }
}

pub fn write_vec_sep<T: Display>(v: &Vec<T>, sep: &str, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut iter = v.iter();
    while let Some(el) = iter.next() {
        write!(f, "{}", el)?;
        if iter.len() != 0 {
            write!(f, "{}", sep)?;
        }
    }
    Ok(())
}

impl Display for RawEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.id)?;
        writeln!(f, "\tnpc_guid: {}", self.npc_guid)?;
        writeln!(f, "\tsequence_count: {}", self.sequence_count)?;
        writeln!(f, "\tstrike_count: {}", self.strike_count)?;
        write!(f, "\tsequence_lengths: ")?;
        write_vec_sep(&self.sequence_lengths, ", ", f)?;
        writeln!(f, "\n\tdeck:")?;
        if let Some(deck) = &self.deck {
            writeln!(f, "{}", deck)
        } else {
            writeln!(f, "\t\tDefault for cycle; see character with npc guid `{}`", self.npc_guid)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Deck {
    pub anchor: Card,
    pub cards: Vec<Card>
}

impl_tryfrom_field!{Struct for Deck:
    |value| {
        field_get!(let cards: List = value.cards);
        let mut deck = Vec::new();
        for card in cards {
            deck.push(card.try_into()?);
        }

        Ok(Self {
            anchor: if let Some(anchor) = value.get("anchor") {
                anchor.try_into()?
            } else {
                return Err("No `anchor` field for Deck.".into());
            },
            cards: deck,
        })
    }
}

impl Display for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\t\tanchor: {}\n\t\t", self.anchor)?;
        write_vec_sep(&self.cards, "\n\t\t", f)
    }
}

#[derive(Debug)]
pub struct NPC {
    pub id: String,
    pub hand_size: u8,
    pub prefers_doubles: bool,
    pub mad_threshold: u8,
    pub decks: [Deck; 6],
}

impl PartialEq for NPC {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}
impl Eq for NPC {}

impl PartialOrd for NPC {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NPC {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl_tryfrom_field!{Struct for NPC:
    |field| {
        field_get!(let id: Str = field.id);
        field_get!(let hand_size: Uint = field.handSize);
        field_get!(let doubles: Uint = field.prefersDoubles);
        field_get!(let mad: Uint = field.mad);
        field_get!(let deck0: Struct = field.deck0);
        field_get!(let deck1: Struct = field.deck1);
        field_get!(let deck2: Struct = field.deck2);
        field_get!(let deck3: Struct = field.deck3);
        field_get!(let deck4: Struct = field.deck4);
        field_get!(let deck5: Struct = field.deck5);

        Ok(Self {
            id: id.to_owned(),
            hand_size: *hand_size as u8,
            prefers_doubles: *doubles != 0,
            mad_threshold: *mad as u8,
            decks: [
                deck0.try_into()?,
                deck1.try_into()?,
                deck2.try_into()?,
                deck3.try_into()?,
                deck4.try_into()?,
                deck5.try_into()?,
            ]
        })
    }
}

impl NPC {
    fn is_npc(map: &BTreeMap<String, Field>) -> bool {
        map.contains_key("deck0")
    }

    pub fn load_asset(path: PathBuf) -> Result<Option<Self>, Box<dyn Error>> {
        let yaml: Field = serde_yaml::from_reader(File::open(path)?)?;
        let Field::Struct(data_map) = yaml else { return Err("Root isn't a map".into()); };
        let ref_data_map = &data_map;
        field_get!(let monobehaviour: Struct = ref_data_map.MonoBehaviour);
        if NPC::is_npc(monobehaviour) {
            let npc = monobehaviour.try_into()?;
            Ok(Some(npc))
        } else {
            Ok(None)
        }
    }

    pub fn print_details(&self) {
        println!("NPC - {}:", self.id);
        println!("\tHand Size: {}", self.hand_size);
        println!("\tPrefers Doubles: {}", self.prefers_doubles);
        println!("\tDiscordances to become mad: {}", self.mad_threshold);
    }

    pub fn print_deck(&self, cycle: usize) {
        println!("\tDeck for cycle {}:", cycle);
        println!("{}", self.decks[cycle]);
    }

    pub fn print_fallback_deck(&self) {
        println!("\tFallback deck (unexpected cycle value):");
        println!("{}", self.decks[0]);
    }

    pub fn print_all_decks(&self) {
        (1..=5).for_each(|i| NPC::print_deck(self, i));
        self.print_fallback_deck();
    }
}
