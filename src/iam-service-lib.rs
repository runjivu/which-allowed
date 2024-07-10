use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::{
    get_policy::*, get_policy_version::*, get_role::*, list_attached_role_policies::*,
    get_user_policy::*, get_role_policy::*, get_group_policy::*, list_attached_group_policies::*,
    list_attached_user_policies::*, list_group_policies::*, list_groups::*,
    list_groups_for_user::*, list_policies::*, list_role_policies::*, list_roles::*,
    list_user_policies::*, list_users::*,
};
use aws_sdk_iam::types::{
    AccessKey, AttachedPolicy, Group, Policy, PolicyScopeType, PolicyVersion, Role, User,
};
use aws_sdk_iam::Client as iamClient;
use tokio::time::{sleep, Duration};

pub async fn get_policy(
    client: &iamClient,
    policy: AttachedPolicy,
) -> Result<Policy, SdkError<GetPolicyError>> {
    let response = client
        .get_policy()
        .set_policy_arn(policy.policy_arn)
        .send()
        .await?;
    let response = response.policy.unwrap();
    Ok(response)
}

pub async fn get_user_policy(
    client: &iamClient,
    user_name: &String,
    policy_name: &String,
) -> Result<String, SdkError<GetUserPolicyError>> {
    let response = client
        .get_user_policy()
        .user_name(user_name)
        .policy_name(policy_name)
        .send()
        .await?;
    let document = response.policy_document;
    Ok(document)
}

pub async fn get_role_policy(
    client: &iamClient,
    role_name: &String,
    policy_name: &String,
) -> Result<String, SdkError<GetRolePolicyError>> {
    let response = client
        .get_role_policy()
        .role_name(role_name)
        .policy_name(policy_name)
        .send()
        .await?;
    let document = response.policy_document;
    Ok(document)
}

pub async fn get_group_policy(
    client: &iamClient,
    group_name: &String,
    policy_name: &String,
) -> Result<String, SdkError<GetGroupPolicyError>> {
    let response = client
        .get_group_policy()
        .group_name(group_name)
        .policy_name(policy_name)
        .send()
        .await?;
    let document = response.policy_document;
    Ok(document)
}


pub async fn get_policy_version(
    client: &iamClient,
    policy: Policy,
) -> Result<PolicyVersion, SdkError<GetPolicyVersionError>> {
    let response = client
        .get_policy_version()
        .set_policy_arn(policy.arn)
        .set_version_id(policy.default_version_id)
        .send()
        .await?;
    let response = response.policy_version.unwrap();
    Ok(response)
}

pub async fn list_roles(
    client: &iamClient,
    path_prefix: Option<String>,
    marker: Option<String>,
    max_items: Option<i32>,
) -> Result<ListRolesOutput, SdkError<ListRolesError>> {
    let response = client
        .list_roles()
        .set_path_prefix(path_prefix)
        .set_marker(marker)
        .set_max_items(max_items)
        .send()
        .await?;
    Ok(response)
}

pub async fn get_role(
    client: &iamClient,
    role_name: String,
) -> Result<GetRoleOutput, SdkError<GetRoleError>> {
    let response = client.get_role().role_name(role_name).send().await?;
    Ok(response)
}

pub async fn list_users(
    client: &iamClient,
    path_prefix: Option<String>,
    marker: Option<String>,
    max_items: Option<i32>,
) -> Result<ListUsersOutput, SdkError<ListUsersError>> {
    let response = client
        .list_users()
        .set_path_prefix(path_prefix)
        .set_marker(marker)
        .set_max_items(max_items)
        .send()
        .await?;
    Ok(response)
}

pub async fn list_policies(
    client: iamClient,
    path_prefix: String,
) -> Result<Vec<String>, SdkError<ListPoliciesError>> {
    let list_policies = client
        .list_policies()
        .path_prefix(path_prefix)
        .scope(PolicyScopeType::Local)
        .into_paginator()
        .items()
        .send()
        .try_collect()
        .await?;

    let policy_names = list_policies
        .into_iter()
        .map(|p| {
            let name = p
                .policy_name
                .unwrap_or_else(|| "Missing Policy Name".to_string());
            name
        })
        .collect();

    Ok(policy_names)
}

pub async fn list_groups(
    client: &iamClient,
    path_prefix: Option<String>,
    marker: Option<String>,
    max_items: Option<i32>,
) -> Result<ListGroupsOutput, SdkError<ListGroupsError>> {
    let response = client
        .list_groups()
        .set_path_prefix(path_prefix)
        .set_marker(marker)
        .set_max_items(max_items)
        .send()
        .await?;

    Ok(response)
}

pub async fn list_attached_role_policies(
    client: &iamClient,
    role_name: &String,
) -> Result<Option<Vec<AttachedPolicy>>, SdkError<ListAttachedRolePoliciesError>> {
    let response = client
        .list_attached_role_policies()
        .role_name(role_name)
        .send()
        .await?;

    let attached_policies = response.attached_policies;
    Ok(attached_policies)
}

pub async fn list_attached_user_policies(
    client: &iamClient,
    user_name: &String,
) -> Result<Option<Vec<AttachedPolicy>>, SdkError<ListAttachedUserPoliciesError>> {
    let response = client
        .list_attached_user_policies()
        .user_name(user_name)
        .send()
        .await?;
    let attached_policies = response.attached_policies;
    Ok(attached_policies)
}

pub async fn list_attached_group_policies(
    client: &iamClient,
    group_name: &String,
) -> Result<Option<Vec<AttachedPolicy>>, SdkError<ListAttachedGroupPoliciesError>> {
    let response = client
        .list_attached_group_policies()
        .group_name(group_name)
        .send()
        .await?;

    let attached_policies = response.attached_policies;
    Ok(attached_policies)
}

pub async fn list_role_policies(
    client: &iamClient,
    role_name: &str,
) -> Result<Vec<String>, SdkError<ListRolePoliciesError>> {
    let response = client
        .list_role_policies()
        .role_name(role_name)
        .send()
        .await?;
    let policy_names = response.policy_names;
    Ok(policy_names)
}

pub async fn list_user_policies(
    client: &iamClient,
    user_name: &str,
) -> Result<Vec<String>, SdkError<ListUserPoliciesError>> {
    let response = client
        .list_user_policies()
        .user_name(user_name)
        .send()
        .await?;

    let policy_names = response.policy_names;
    Ok(policy_names)
}
pub async fn list_group_policies(
    client: &iamClient,
    group_name: &str,
) -> Result<Vec<String>, SdkError<ListGroupPoliciesError>> {
    let response = client
        .list_group_policies()
        .group_name(group_name)
        .send()
        .await?;

    let policy_names = response.policy_names;
    Ok(policy_names)
}

pub async fn list_groups_for_user(
    client: &iamClient,
    user_name: &str,
) -> Result<Vec<String>, SdkError<ListGroupsForUserError>> {
    let response = client
        .list_groups_for_user()
        .user_name(user_name)
        .send()
        .await?;

    let groups: Vec<String> = response
        .groups
        .iter()
        .map(|g| g.group_name.clone())
        .collect();
    Ok(groups)
}
