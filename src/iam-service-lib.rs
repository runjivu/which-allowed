use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::{
    get_policy::*, get_policy_version::*, get_role::*, list_attached_role_policies::*,
    list_attached_user_policies::*, list_groups::*, list_policies::*, list_role_policies::*,
    list_roles::*, list_users::*,
};
use aws_sdk_iam::types::{
    AccessKey, AttachedPolicy, Policy, PolicyScopeType, PolicyVersion, Role, User,
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
    role_name: String,
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
    user_name: String,
) -> Result<Option<Vec<AttachedPolicy>>, SdkError<ListAttachedUserPoliciesError>> {
    let response = client
        .list_attached_user_policies()
        .user_name(user_name)
        .send()
        .await?;
    let attached_policies = response.attached_policies;
    Ok(attached_policies)
}

pub async fn list_role_policies(
    client: &iamClient,
    role_name: &str,
    marker: Option<String>,
    max_items: Option<i32>,
) -> Result<ListRolePoliciesOutput, SdkError<ListRolePoliciesError>> {
    let response = client
        .list_role_policies()
        .role_name(role_name)
        .set_marker(marker)
        .set_max_items(max_items)
        .send()
        .await?;

    Ok(response)
}
