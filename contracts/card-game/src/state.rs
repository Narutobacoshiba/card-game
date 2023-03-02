use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub aurand_address: Addr,
    pub owner: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const JOB: Map<String, Addr> = Map::new("job");

pub const RANDOM: Map< String, Vec<String>> = Map::new("random");
