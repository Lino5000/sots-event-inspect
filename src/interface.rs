use crate::{
    data::Event,
    yaml::{ constrain_field_get_body, Field, YamlError },
    field_get, field_get_body, field_value_type, 
};

use std::{
    collections::BTreeMap,
    error::Error,
    fs::File,
    path::PathBuf,
};

pub fn parse_event_data(mut folder_path: PathBuf) -> Result<BTreeMap<String, Event>, Box<dyn Error>> {
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
