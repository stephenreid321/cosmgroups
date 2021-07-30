use cw_storage_plus::{Map, Item, U64Key, MultiIndex, IndexList, Index, IndexedMap, PrimaryKey};
use cosmwasm_std::{Storage, StdResult, Addr};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NewPerson {
    pub name: String,
    pub age: u8,
}

pub const NEW_PEOPLE: Map<&[u8], NewPerson> = Map::new("new_people");

pub const GROUP_COUNTER: Item<u64> = Item::new("group_counter");

pub fn next_group_counter(store: &mut dyn Storage) -> StdResult<u64> {
    let id: u64 = GROUP_COUNTER.may_load(store)?.unwrap_or_default() + 1;
    GROUP_COUNTER.save(store, &id)?;
    Ok(id)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NewGroup {
    pub name: String,
}

pub const NEW_GROUPS: Map<U64Key, NewGroup> = Map::new("groups");

pub fn save_group(store: &mut dyn Storage, group: &NewGroup) -> StdResult<u64> {
    let id = next_group_counter(store)?;
    let key = U64Key::new(id);
    NEW_GROUPS.save(store, key, group)?;
    Ok(id)
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NewMembership {
    pub person: Addr,
    pub group_id: u64,
    pub role: Role
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Role {
    User{},
    Admin {},
    SuperAdmin {}
}

impl<'a> PrimaryKey<'a> for &'a Role{
    type Prefix = ();
    type SubPrefix = ();

    fn key(&self) -> Vec<&[u8]> {
        // this is simple, we don't add more prefixes
        match self {
            Role::User{ .. } => vec![&[0u8]],
            Role::Admin { .. } => vec![&[1u8]],
            Role::SuperAdmin { .. } => vec![&[2u8]],
        }
    }
}

pub struct MembershipIndexes<'a> {
    // indexed by person key
    pub person: MultiIndex<'a, (Vec<u8>, Vec<u8>), NewMembership>,
    pub group: MultiIndex<'a, (U64Key, Vec<u8>), NewMembership>,
    pub role: MultiIndex<'a, (Vec<u8>, Vec<u8>), NewMembership>,
}

// Future Note: this can likely be macro-derived
impl<'a> IndexList<NewMembership> for MembershipIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<NewMembership>> + '_> {
        let v: Vec<&dyn Index<NewMembership>> = vec![&self.person, &self.group];
        Box::new(v.into_iter())
    }
}

pub fn memberships<'a>() -> IndexedMap<'a, &'a [u8], NewMembership, MembershipIndexes<'a>> {
    let pk_namespace = "membership";
    let indexes = MembershipIndexes {
        person: MultiIndex::new(
            |d, k| (d.person.as_ref().joined_key(), k),
            pk_namespace,
            "membership__person",
        ),
        group: MultiIndex::new(
            |d,k| (U64Key::new(d.group_id), k),
            pk_namespace,
            "membership__group",
        ),
        role: MultiIndex::new(
            |d,k| (d.role.borrow().joined_key(), k),
        pk_namespace,
        "membership__role",
        ),
    };
    IndexedMap::new(pk_namespace, indexes)
}

#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use std::borrow::{BorrowMut, Borrow};
    use cosmwasm_std::Order;
    use cw_storage_plus::index_string;

    #[test]
    fn test_memberships() {
        let mut store = MockStorage::new();

        let addr1 = Addr::unchecked("addr1");
        let p1_name = "p1";
        let person1 = NewPerson {
            name: p1_name.into(),
            age: 12,
        };

        let addr2 = Addr::unchecked("addr2");
        let p2_name = "p2";
        let person2 = NewPerson {
            name: p2_name.into(),
            age: 12,
        };

        let addr3 = Addr::unchecked("addr3");
        let p3_name = "p3";
        let person3 = NewPerson {
            name: p3_name.into(),
            age: 12,
        };

        NEW_PEOPLE.save(store.borrow_mut(), &index_string(addr1.as_str()), &person1).unwrap();
        NEW_PEOPLE.save(store.borrow_mut(), &index_string(addr2.as_str()), &person2).unwrap();
        NEW_PEOPLE.save(store.borrow_mut(), &index_string(addr3.as_str()), &person3).unwrap();

        let g1_name = "g1";
        let group1 = NewGroup { name: g1_name.into() };
        let g2_name = "g2";
        let group2 = NewGroup { name: g2_name.into() };

        let g1_id = save_group(store.borrow_mut(), &group1).unwrap();
        let g2_id = save_group(store.borrow_mut(), &group2).unwrap();

        let membership1 = NewMembership{
            person: addr1,
            group_id: g1_id,
            role: Role::User {}
        };
        let key = U64Key::new(1);
        let ms_store = memberships();
        ms_store.save(store.borrow_mut(), key.joined_key().as_slice(), &membership1).unwrap();
        let person_memberships = ms_store
            .idx
            .person
            .prefix(membership1.person.as_ref().joined_key())
            .range(store.borrow(), None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>().unwrap();
        println!("{:?}", person_memberships);

        let key = U64Key::new(2).joined_key();
        let membership2 = NewMembership {
            person: addr2,
            group_id: g1_id,
            role: Role::Admin {}
        };
        ms_store.save(store.borrow_mut(), &key, &membership2).unwrap();
        let person_memberships = ms_store
            .idx
            .person
            .prefix(membership2.person.as_ref().joined_key())
            .range(store.borrow(), None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>().unwrap();
        println!("{:?}", person_memberships);

        let group_memberships = ms_store
            .idx
            .group
            .prefix(U64Key::new(g1_id))
            .range(store.borrow(), None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>().unwrap();
        println!("{:?}", group_memberships);

        let role = vec![1u8];
        let all_admins = ms_store
            .idx
            .role
            .prefix(role)
            .range(store.borrow(), None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>().unwrap();
        println!("{:?}", all_admins);

    }
}

/*
#[cfg(test)]
mod test {
    use super::*;
    use cosmwasm_std::testing::MockStorage;
    use std::borrow::{BorrowMut};
    use cosmwasm_std::Order;

    #[test]
    fn test_tokens() {
        let mut store = MockStorage::new();

        let admin1 = Addr::unchecked("addr1");
        let ticker1 = "TOKEN1".to_string();
        let token1 = Token {
            admin: admin1.clone(),
            ticker: ticker1,
            identifier: 0,
        };

        let token_id = increment_tokens(store.borrow_mut()).unwrap();
        tokens().save(store.borrow_mut(), &U64Key::from(token_id).joined_key(), &token1).unwrap();

        let ticker2 = "TOKEN2".to_string();
        let token2 = Token {
            admin: admin1.clone(),
            ticker: ticker2,
            identifier: 0,
        };

        let token_id = increment_tokens(store.borrow_mut()).unwrap();
        tokens().save(store.borrow_mut(), &U64Key::from(token_id).joined_key(), &token1).unwrap();

        // want to load token using admin1 and ticker1
        let list: Vec<_> = tokens()
            .idx.admin
            .prefix(index_string(admin1.as_str()))
            .range(&store, None, None, Order::Ascending)
            .collect::<StdResult<_>>().unwrap();
        let (_, t) = &list[0];
        assert_eq!(t, &token1);
        assert_eq!(2, list.len());


        let ticker3 = "TOKEN3".to_string();
        let token2 = Token {
            admin: admin1.clone(),
            ticker: ticker3,
            identifier: 0,
        };

        let token_id = increment_tokens(store.borrow_mut()).unwrap();
        tokens().save(store.borrow_mut(), &U64Key::from(token_id).joined_key(), &token1).unwrap();
    }
}

pub const TOKEN_COUNT: Item<u64> = Item::new("num_tokens");

pub fn num_tokens(storage: &dyn Storage) -> StdResult<u64> {
    Ok(TOKEN_COUNT.may_load(storage)?.unwrap_or_default())
}

pub fn increment_tokens(storage: &mut dyn Storage) -> StdResult<u64> {
    let val = num_tokens(storage)? + 1;
    TOKEN_COUNT.save(storage, &val)?;
    Ok(val)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub admin: Addr,
    pub ticker: String,
    pub identifier: u8,
}

pub struct TokenIndexes<'a> {
    // secondary indexed by admin address
    // last U64Key is the primary key which is auto incremented token counter
    pub admin: MultiIndex<'a, (Vec<u8>, Vec<u8>), Token>,
    pub identifier: UniqueIndex<'a, U8Key, Token>,
}

// this may become macro, not important just boilerplate, builds the list of indexes for later use
impl<'a> IndexList<Token> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Token>> + '_> {
        let v: Vec<&dyn Index<Token>> = vec![&self.admin, &self.identifier];
        Box::new(v.into_iter())
    }
}

const TOKEN_NAMESPACE: &str = "tokens";

pub fn tokens<'a>() -> IndexedMap<'a, &'a [u8], Token, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        admin: MultiIndex::new(
            |d, k| (index_string(d.admin.as_str()), k),
            TOKEN_NAMESPACE,
            "tokens__admin",
        ),
        identifier: UniqueIndex::new(|d| U8Key::new(d.identifier), "token_unique"),
    };
    IndexedMap::new(TOKEN_NAMESPACE, indexes)
}
*/
