use cosmwasm_schema::{cw_serde, QueryResponses};

/// Message type for `instantiate` entry_point
#[cw_serde]
pub struct InstantiateMsg {
    pub aurand_address: String,
    pub owner: String,
}

/// Message type for `execute` entry_point
#[cw_serde]
pub enum ExecuteMsg {
    ShuffleDeck {
        request_id: String
    },

    ReceiveHexRandomness {
        request_id: String,
        randomness: Vec<String>
    },

    SetConfig{
        aurand_address: String,
        owner: String,
    },
}

#[cw_serde]
pub enum AurandExecuteMsg {
    RequestHexRandomness{
        request_id: String,
        num: u32,
    } ,
}

/// Message type for `migrate` entry_point
#[cw_serde]
pub enum MigrateMsg {}

/// Message type for `query` entry_point
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Option<GetDecksResponse>)]
    GetDecks {
        request_id: String,
        num: Option<u32>
    },
}

#[cw_serde]
pub struct GetDecksResponse {
    pub decks: Vec<Vec<u8>>,
}
