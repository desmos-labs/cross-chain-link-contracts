use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {
    pub wormhole_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SubmitVaa { data: Binary },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}


#[cw_serde]
pub struct Payload {
    pub channel_id: String,
    pub packet: Binary,
}

