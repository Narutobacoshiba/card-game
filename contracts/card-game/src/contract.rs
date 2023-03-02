#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult};
use cosmwasm_std::{Api, Addr, WasmMsg};
use cw2::set_contract_version;
use std::collections::VecDeque;
use nois::shuffle;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    AurandExecuteMsg, GetDecksResponse,
};
use crate::state::{
    CONFIG, Config, JOB, RANDOM
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:card-game";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_REQUEST_ID_LENGTH: usize = 64;
const MAX_REQUEST_DECK_LENGTH: u32 = 103;

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let aurand_addr = addr_validate(deps.api, msg.aurand_address)?;
    let owner = addr_validate(deps.api, msg.owner)?;

    CONFIG.save(deps.storage, &Config{
        aurand_address: aurand_addr.clone(),
        owner: owner.clone()
    })?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("method", aurand_addr)
        .add_attribute("owner", owner))
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ShuffleDeck {
            request_id
        } => execute_shuffle_deck(deps, info, request_id),

        ExecuteMsg::ReceiveHexRandomness {
            request_id,
            randomness
        } => execute_receive_hex_randomness(deps, info, request_id, randomness),

        ExecuteMsg::SetConfig {
            aurand_address,
            owner
        } => execute_set_config(deps, info, aurand_address, owner),
    }
}

fn addr_validate(api: &dyn Api, addr: String) -> Result<Addr, ContractError> {
    let addr = api.addr_validate(&addr)
                        .map_err(|_| ContractError::InvalidAddress{})?;
    Ok(addr)
}

fn execute_shuffle_deck(
    deps: DepsMut, 
    info: MessageInfo,
    request_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized{});
    }

    if request_id.len() > MAX_REQUEST_ID_LENGTH {
        return Err(ContractError::RequestIdToLong{});
    }

    if JOB.has(deps.storage, request_id.clone()) {
        return Err(ContractError::CustomError{val:"Request with same id exist!".to_string()});
    }

    let msg = WasmMsg::Execute{
        contract_addr: config.aurand_address.to_string(),
        msg: to_binary(&AurandExecuteMsg::RequestHexRandomness { 
                        request_id: request_id.clone(),
                        num: 1,
                    })?,
        funds: info.funds,
    };

    JOB.save(deps.storage, request_id.clone(), &info.sender.clone())?;

    Ok(Response::new().add_message(msg)
            .add_attribute("action", "shuffle_deck")
            .add_attribute("sender", info.sender)
            .add_attribute("request_id", request_id))
}

fn execute_receive_hex_randomness(
    deps: DepsMut, 
    info: MessageInfo,
    request_id: String,
    randomness: Vec<String>,
) -> Result<Response, ContractError> {

    let config = CONFIG.load(deps.storage)?;

    if config.aurand_address != info.sender {
        return Err(ContractError::Unauthorized{});
    }

    if !JOB.has(deps.storage, request_id.clone()) {
        return Err(ContractError::CustomError{val:"Request with id does't exist!".to_string()});
    }

    RANDOM.save(deps.storage, request_id.clone(), &randomness)?;

    JOB.remove(deps.storage, request_id);

    return Ok(Response::new().add_attribute("action","receive_hex_randomness"));
}


fn execute_set_config(
    deps: DepsMut, 
    info: MessageInfo,
    aurand_address: String,
    owner: String
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.owner {
        return Err(ContractError::Unauthorized{});
    }

    let aurand_addr = addr_validate(deps.api, aurand_address)?;
    let owner = addr_validate(deps.api, owner)?;

    CONFIG.save(deps.storage, &Config{
        aurand_address: aurand_addr.clone(),
        owner: owner.clone()
    })?;

    return Ok(Response::new().add_attribute("action","set_config")
                    .add_attribute("method", aurand_addr)
                    .add_attribute("owner", owner));
}


/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDecks {request_id, num} => to_binary(&query_get_decks(deps, request_id, num)?),
    }
}

pub fn query_get_decks(deps: Deps, request_id: String, num: Option<u32>) -> StdResult<Option<GetDecksResponse>> {
    if !RANDOM.has(deps.storage, request_id.clone()) {
        return Ok(None);
    }

    let random = RANDOM.load(deps.storage, request_id)?;
    let random: [u8; 32] = hex::decode(random[0].clone()).unwrap().try_into().unwrap();
    if random.len() == 0 {
        return Ok(None);
    }

    if num.is_none() {
        return Ok(None);
    }

    let num = num.unwrap();

    if num > MAX_REQUEST_DECK_LENGTH {
        return Ok(None);
    }

    let mut init_deck: Vec<u8> = (1..=52).collect::<Vec<_>>();
    init_deck = shuffle(random, init_deck);

    let mut card_queue: VecDeque<u8> = VecDeque::from(init_deck); 

    let mut vecs: Vec<Vec<u8>> = Vec::new();

    for _ in 0..num {
        let card = card_queue.pop_back();
        card_queue.push_front(card.unwrap());

        vecs.push(card_queue.clone().into_iter().collect::<Vec<u8>>());  
    }

    Ok(Some(GetDecksResponse{
        decks: vecs
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    todo!()
}
