#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, QueryRequest,
    Response, StdError, StdResult, WasmQuery,
};
use serde_json_wasm;

use crate::wormhole::{
    vaa_archive_add, vaa_archive_check, ParsedVAA, WormholeContractError, WormholeQueryMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, Payload, QueryMsg};
use crate::state::{Config, CHANNELS, CONFIG};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cosmwasm";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        wormhole_contract: msg.wormhole_contract,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitVaa { data } => Ok(submit_vaa(deps, env, info, &data)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

fn submit_vaa(deps: DepsMut, env: Env, _info: MessageInfo, data: &Binary) -> StdResult<Response> {
    // Parse and archive VAA
    let vaa = parse_vaa(deps.as_ref(), env.block.time.seconds(), data)?;
    if vaa_archive_check(deps.storage, vaa.hash.as_slice()) {
        return WormholeContractError::VaaAlreadyExecuted.std_err();
    }
    vaa_archive_add(deps.storage, vaa.hash.as_slice())?;

    let payload: Payload = serde_json_wasm::from_slice(&vaa.payload).unwrap();

    // Check channel is connected
    let is_channel_connected = !CHANNELS
        .may_load(deps.storage, payload.channel_id.clone())?
        .unwrap_or(false);
    if is_channel_connected {
        return Err(StdError::generic_err(format!(
            "unregistered channel id {}",
            payload.channel_id
        )));
    }

    Ok(Response::new().add_message(IbcMsg::SendPacket {
        channel_id: payload.channel_id,
        data: payload.packet,
        timeout: IbcTimeout::with_timestamp(env.block.time.plus_days(1)),
    }))
}

fn parse_vaa(deps: Deps, block_time: u64, data: &Binary) -> StdResult<ParsedVAA> {
    let cfg = CONFIG.load(deps.storage)?;
    let vaa: ParsedVAA = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cfg.wormhole_contract,
        msg: to_json_binary(&WormholeQueryMsg::VerifyVAA {
            vaa: data.clone(),
            block_time,
        })?,
    }))?;
    Ok(vaa)
}
