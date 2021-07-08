use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PersonData {
    pub name: String,
    pub age: i32,
}

pub const PEOPLE: Map<&[u8], PersonData> = Map::new("people");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct GroupData {
    pub name: String
}

pub const GROUPS: Map<&[u8], GroupData> = Map::new("groups");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum MembershipStatus {
    Regular,
    Admin,
    SuperAdmin
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MembershipData {
    pub person_id: String,
    pub group_id: String,
    pub membership_status: MembershipStatus
}

pub const MEMBERSHIPS: Map<&[u8], MembershipData> = Map::new("memberships");
