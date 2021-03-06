use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, testing::MockStorage, Order
};

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use crate::state::{Person, PEOPLE};
use crate::state::{Group, GROUPS};
use crate::state::{Membership, MEMBERSHIPS};
use crate::state::{MembershipStatus, MEMBERSHIP_STATUSES};


// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender,
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
    }
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::default())
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }

    /*
    #[test]
    fn test_groups() {

        let mut store = MockStorage::new();

        let person_id = "john".to_string();
        let group1_id = "dandelion".to_string();
        let group2_id = "autopia".to_string();
        let membership1_id = "membership1".to_string();
        let membership2_id = "membership2".to_string();
        let membership_status_regular_id = "regular".to_string();
        let membership_status_admin_id = "admin".to_string();
        let membership_status_superadmin_id = "superadmin".to_string();

        let person = Person {
            name: "John".to_string(),
            age: 32,
            membership_ids: vec![membership1_id.clone(), membership2_id.clone()]
        };

        let group1 = Group {
            name: "Dandelion".to_string(),
            membership_ids: vec![membership1_id.clone()]
        };

        let group2 = Group {
            name: "Autopia".to_string(),
            membership_ids: vec![membership2_id.clone()]
        };

        let membership_status_regular = MembershipStatus {
            status: "Regular".to_string(),
            membership_ids: vec![membership1_id.clone()]
        };

        let membership_status_admin = MembershipStatus {
            status: "Admin".to_string(),
            membership_ids: vec![membership2_id.clone()]
        };

        let membership_status_superadmin = MembershipStatus {
            status: "Super Admin".to_string(),
            membership_ids: vec![]
        };

        let membership1 = Membership {
            person_id: person_id.clone(),
            group_id: group1_id.clone(),
            membership_status_id: membership_status_regular_id.clone()
        };

        let membership2 = Membership {
            person_id: person_id.clone(),
            group_id: group2_id.clone(),
            membership_status_id: membership_status_admin_id.clone()
        };

        PEOPLE.save(&mut store, person_id.as_ref(), &person).unwrap();
        let loaded_person = PEOPLE.key(person_id.as_ref()).load(&store).unwrap();
        assert_eq!(person, loaded_person);

        GROUPS.save(&mut store, group1_id.as_ref(), &group1).unwrap();
        let loaded_group1 = GROUPS.key(group1_id.as_ref()).load(&store).unwrap();
        assert_eq!(group1, loaded_group1);

        GROUPS.save(&mut store, group2_id.as_ref(), &group2).unwrap();
        let loaded_group2 = GROUPS.key(group2_id.as_ref()).load(&store).unwrap();
        assert_eq!(group2, loaded_group2);

        MEMBERSHIPS.save(&mut store, membership1_id.as_ref(), &membership1).unwrap();
        let loaded_membership1 = MEMBERSHIPS.key(membership1_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership1, loaded_membership1);

        MEMBERSHIPS.save(&mut store, membership2_id.as_ref(), &membership2).unwrap();
        let loaded_membership2 = MEMBERSHIPS.key(membership2_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership2, loaded_membership2);

        MEMBERSHIP_STATUSES.save(&mut store, membership_status_regular_id.as_ref(), &membership_status_regular).unwrap();
        let loaded_membership_status_regular = MEMBERSHIP_STATUSES.key(membership_status_regular_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership_status_regular, loaded_membership_status_regular);

        MEMBERSHIP_STATUSES.save(&mut store, membership_status_admin_id.as_ref(), &membership_status_admin).unwrap();
        let loaded_membership_status_admin = MEMBERSHIP_STATUSES.key(membership_status_admin_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership_status_admin, loaded_membership_status_admin);

        MEMBERSHIP_STATUSES.save(&mut store, membership_status_superadmin_id.as_ref(), &membership_status_superadmin).unwrap();
        let loaded_membership_status_superadmin = MEMBERSHIP_STATUSES.key(membership_status_superadmin_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership_status_superadmin, loaded_membership_status_superadmin);

        // how do I get all the memberships of a person?
        // in Ruby/Mongoid I would do something like Membership.where(person_id: 'john')
        // edit: I think I need to cache the membership IDs in Person, don't I?

        let person_memberships: Vec<_> = person.membership_ids.iter().map(|membership_id| {
                return MEMBERSHIPS.key(membership_id.as_ref()).load(&store).unwrap();
            })
            .collect();

       // person_memberships.iter().for_each(|membership| {
       //      println!("{}", membership.person_id);
       //      println!("{}", membership.group_id);
       //  });

        let filtered_memberships: Vec<_> = MEMBERSHIPS
            .range(&store, None, None, Order::Ascending)
            .map(|membership| {
                let (_membership_id, membership) = membership.unwrap();
                membership
            })
            .filter(|membership| {
                membership.person_id == person_id
            })
            .collect();

        assert_eq!(person_memberships, filtered_memberships);

        // how do I get all memberships that are Admins or SuperAdmins?
        // in Ruby/Mongoid I would do something like Membership.where(:membership_status.in => [MembershipStatus::Admin, MembershipStatus::SuperAdmin])

        let membership_status_admin_memberships: Vec<_> = membership_status_admin.membership_ids.iter().map(|membership_id| {
            return MEMBERSHIPS.key(membership_id.as_ref()).load(&store).unwrap();
        })
            .collect();

        let membership_status_superadmin_memberships: Vec<_> = membership_status_superadmin.membership_ids.iter().map(|membership_id| {
            return MEMBERSHIPS.key(membership_id.as_ref()).load(&store).unwrap();
        })
            .collect();

        let admin_and_superadmin_memberships: Vec<_> = [membership_status_admin_memberships, membership_status_superadmin_memberships].concat();

        let filtered_memberships: Vec<_> = MEMBERSHIPS
            .range(&store, None, None, Order::Ascending)
            .map(|membership| {
                let (_membership_id, membership) = membership.unwrap();
                membership
            })
            .filter(|membership| {
                membership.membership_status_id == membership_status_admin_id || membership.membership_status_id == membership_status_superadmin_id
            })
            .collect();

        assert_eq!(admin_and_superadmin_memberships, filtered_memberships);

    }


     */
}
