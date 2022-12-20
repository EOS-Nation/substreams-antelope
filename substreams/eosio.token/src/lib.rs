// standard modules
use std::collections::HashSet;
use std::str::FromStr;

// substream modules
use substreams_antelope_core::pb::antelope::{Block};
use substreams::errors::Error;

// local modules
mod abi;
mod pb;
mod accounts;
use crate::pb::eosio_token::{Action, Actions};
use crate::accounts::{ACCOUNTS};

pub const ACTIONS: [&str; 6] = [
    "create",
    "issue",
    "retire",
    "transfer",
    "open",
    "close",
];

#[substreams::handlers::map]
fn map_actions(block: Block) -> Result<Actions, Error> {
    let mut actions = vec![];
    let filter_by = HashSet::from(ACTIONS);

    for trx in block.clone().all_transaction_traces() {
        for trace in &trx.action_traces {
            let action = trace.action.as_ref().unwrap().clone();
            if !filter_by.contains(action.name.as_str()) { continue; }
            if trace.receiver != action.account { continue; } // skip extra receivers

            actions.push(Action {
                block_num: block.number,
                timestamp: Some(block.header.as_ref().unwrap().timestamp.as_ref().unwrap().clone()),
                transaction_id: trace.transaction_id.clone(),
                account: action.account,
                name: action.name,
                json_data: action.json_data,
            })
        }
    }
    Ok(Actions { actions })
}

#[substreams::handlers::map]
pub fn map_transfers(actions: Actions) -> Result<Actions, Error> {
    let mut response = vec![];

    for action in actions.actions {
        if action.name != "transfer" { continue; }
        response.push(action);
    }

    Ok(Actions { actions: response })
}

#[substreams::handlers::map]
pub fn map_transfers_eosio_token(actions: Actions) -> Result<Actions, Error> {
    let mut response = vec![];

    for action in actions.actions {
        if action.account != "eosio.token" { continue; }
        response.push(action);
    }

    Ok(Actions { actions: response })
}

pub fn has_account(action: Action) -> bool {
    let params = abi::Transfer::from_str(&action.json_data);
    if params.is_err() { return false; }
    let data = params.unwrap();

    let filter_by = HashSet::from(ACCOUNTS);
    if filter_by.contains(data.from.as_str()) { return true; }
    if filter_by.contains(data.to.as_str()) { return true; }
    return false;
}

#[substreams::handlers::map]
pub fn map_transfers_accounts(actions: Actions) -> Result<Actions, Error> {
    let mut response = vec![];

    for action in actions.actions {
        if !has_account(action.clone()) { continue; }
        response.push(action);
    }

    Ok(Actions { actions: response })
}