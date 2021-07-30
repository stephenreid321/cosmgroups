use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Storage, StdResult};
use cw_storage_plus::{Item, U64Key, MultiIndex, IndexedMap, UniqueIndex, index_string_tuple, index_string, PrimaryKey, IndexList, Index, U8Key};
use cw_storage_plus::Map;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Person {
    pub name: String,
    pub age: i32,
    pub membership_ids: Vec<String>
}

pub const PEOPLE: Map<&[u8], Person> = Map::new("people");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Group {
    pub name: String,
    pub membership_ids: Vec<String>
}

pub const GROUPS: Map<&[u8], Group> = Map::new("groups");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MembershipStatus {
    pub status: String,
    pub membership_ids: Vec<String>
}

pub const MEMBERSHIP_STATUSES: Map<&[u8], MembershipStatus> = Map::new("membership_statuses");

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Membership {
    pub person_id: String,
    pub group_id: String,
    pub membership_status_id: String
}

pub const MEMBERSHIPS: Map<&[u8], Membership> = Map::new("memberships");
