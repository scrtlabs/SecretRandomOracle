// use bincode;
use cosmwasm_std::{
    Api, Binary, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, InitResult, MigrateResponse, Querier, QueryResponse, QueryResult, StdResult,
    Storage, WasmMsg,
};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json_wasm;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {}

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> InitResult {
    let init_seed = [0_u8; 32];
    deps.storage.set(b"seed", &init_seed);

    Ok(InitResponse::default())
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    AddEntropy {
        entropy: Binary,
    },
    GetRandom {
        bytes: u64,
        callback_address: HumanAddr,
        callback_input: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RandomAnswer {
    RandomBytes: Binary,
    Input: Binary,
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult {
    match msg {
        HandleMsg::AddEntropy { entropy } => {
            let mut seed = deps.storage.get(b"seed").unwrap();
            seed.extend(entropy.0);
            seed.extend(env.message.sender.as_slice().to_vec());
            seed.extend(env.block.chain_id.as_bytes().to_vec());
            seed.extend(&env.block.height.to_be_bytes());
            seed.extend(&env.block.time.to_be_bytes());
            let new_seed: [u8; 32] = Sha256::digest(&seed).into();
            deps.storage.set(b"seed", &new_seed);
        }
        HandleMsg::GetRandom {
            bytes,
            callback_address,
            callback_input,
        } => {
            let mut seed = deps.storage.get(b"seed").unwrap();
            let rng = ChaChaRng::from_seed(seed);

            let mut dest: Vec<u8> = vec![0; bytes as usize];
            for i in 0..dest.len() {
                dest[i] = rng.gen();
            }

            seed.extend(callback_address.0.as_bytes());
            seed.extend(callback_input.0);
            seed.extend(env.message.sender.as_slice().to_vec());
            seed.extend(env.block.chain_id.as_bytes().to_vec());
            seed.extend(&env.block.height.to_be_bytes());
            seed.extend(&env.block.time.to_be_bytes());
            let new_seed: [u8; 32] = Sha256::digest(&seed).into();
            deps.storage.set(b"seed", &new_seed);

            let answer: RandomAnswer = RandomAnswer {
                RandomBytes: Binary(dest),
                Input: callback_input,
            };

            return Ok(HandleResponse {
                messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    send: vec![],
                    contract_addr: callback_address.clone(),
                    msg: Binary(serde_json_wasm::to_vec(&answer).unwrap()),
                })],
                log: vec![],
                data: None,
            });
        }
    };

    Ok(HandleResponse::default())
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryMsg {}

pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _msg: QueryMsg,
) -> QueryResult {
    Ok(QueryResponse::default())
}

/////////////////////////////// Migrate ///////////////////////////////
// Isn't supported by the Secret Network, but we must declare this to
// comply with CosmWasm 0.9 API
///////////////////////////////////////////////////////////////////////

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> StdResult<MigrateResponse> {
    Ok(MigrateResponse::default())
}
