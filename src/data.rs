use std::{
    collections::BTreeSet,
    fmt::Display,
    ops::{ Deref, DerefMut },
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
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
        f.write_fmt(format_args!("{} | {} | {}", self.input, self.effect, self.output))
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
pub struct Event {
    pub id: String,
    pub npc_guid: String,
    pub sequence_count: u8,
    pub strike_count: u8,
    pub sequence_lengths: Vec<u8>,
    pub deck: Option<Deck>
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

impl_tryfrom_field!{Struct for Event:
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
        writeln!(f, "\n\tdeck:")?;
        if let Some(deck) = &self.deck {
            write!(f, "\t\tanchor: {}\n\t\t", deck.anchor)?;
            write_vec_sep(&deck.cards, "\n\t\t", f)?;
            writeln!(f, "")
        } else {
            writeln!(f, "Default for cycle; see character with npc guid `{}`", self.npc_guid)
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
