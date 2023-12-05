use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub wormhole_contract: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CHANNELS: Map<String, bool> = Map::new("channels");
