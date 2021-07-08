use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, testing::MockStorage
};

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use crate::state::{PersonData, PEOPLE};
use crate::state::{GroupData, GROUPS};
use crate::state::{MembershipStatus, MembershipData, MEMBERSHIPS};

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

    #[test]
    fn test_groups() {

        let mut store = MockStorage::new();

        let person_id = "john".to_string();
        let person_data = PersonData {
            name: "John".to_string(),
            age: 32,
        };

        let group1_id = "dandelion".to_string();
        let group1_data = GroupData {
            name: "Dandelion".to_string()
        };
        let group2_id = "autopia".to_string();
        let group2_data = GroupData {
            name: "Autopia".to_string()
        };
        
        let membership1_id = "membership1".to_string();
        let membership1_data = MembershipData {
            person_id: person_id.clone(),
            group_id: group1_id.clone(),
            membership_status: MembershipStatus::Regular
        };
        let membership2_id = "membership2".to_string();
        let membership2_data = MembershipData {
            person_id: person_id.clone(),
            group_id: group2_id.clone(),
            membership_status: MembershipStatus::Admin
        };

        PEOPLE.save(&mut store, person_id.as_ref(), &person_data).unwrap();
        let loaded_person = PEOPLE.key(person_id.as_ref()).load(&store).unwrap();
        assert_eq!(person_data, loaded_person);

        GROUPS.save(&mut store, group1_id.as_ref(), &group1_data).unwrap();
        let loaded_group1 = GROUPS.key(group1_id.as_ref()).load(&store).unwrap();
        assert_eq!(group1_data, loaded_group1);

        GROUPS.save(&mut store, group2_id.as_ref(), &group2_data).unwrap();
        let loaded_group2 = GROUPS.key(group2_id.as_ref()).load(&store).unwrap();
        assert_eq!(group2_data, loaded_group2);

        MEMBERSHIPS.save(&mut store, membership1_id.as_ref(), &membership1_data).unwrap();
        let loaded_membership1 = MEMBERSHIPS.key(membership1_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership1_data, loaded_membership1);

        MEMBERSHIPS.save(&mut store, membership2_id.as_ref(), &membership2_data).unwrap();
        let loaded_membership2 = MEMBERSHIPS.key(membership2_id.as_ref()).load(&store).unwrap();
        assert_eq!(membership2_data, loaded_membership2);

        // how do I get all the memberships of a person?
        // in Ruby/Mongoid I would do something like Membership.where(person_id: 'john')
        //
        // let person_memberships = MEMBERSHIPS.where(person_id: person_id)
        // assert_eq!(person_memberships, [loaded_membership1, loaded_membership2])

        // similarly, how do I get all the memberships of a group?
        // in Ruby/Mongoid I would do something like Membership.where(group_id: 'dandelion')
        //
        // let group1_memberships = MEMBERSHIPS.where(group_id: group1_id)
        // assert_eq!(group1_memberships, [loaded_membership1])
        //
        // let group2_memberships = MEMBERSHIPS.where(group_id: group2_id)
        // assert_eq!(group2_memberships, [loaded_membership2])

        // how do I get all memberships that are Admins or SuperAdmins?
        // in Ruby/Mongoid I would do something like Membership.where(:membership_status.in => [MembershipStatus::Admin, MembershipStatus::SuperAdmin])
        //
        // let admin_memberships = MEMBERSHIPS.where(:membership_status.in => [MembershipStatus::Admin, MembershipStatus::SuperAdmin])
        // assert_eq!(admin_memberships, [loaded_membership2])

    }

}
