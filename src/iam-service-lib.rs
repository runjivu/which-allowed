// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

// snippet-start:[rust.example_code.iam.scenario_getting_started.lib]

use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::{
    list_attached_user_policies::*, list_attached_role_policies::*, list_groups::*, list_policies::*, list_role_policies::*,
    list_roles::*, list_users::*, get_role::*,
};
use aws_sdk_iam::types::{AccessKey, Policy, PolicyScopeType, Role, User, AttachedPolicy};
use aws_sdk_iam::Client as iamClient;
use tokio::time::{sleep, Duration};



// snippet-start:[rust.example_code.iam.service.list_roles]
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
// snippet-end:[rust.example_code.iam.service.list_roles]

// snippet-start:[rust.example_code.iam.service.get_role]
pub async fn get_role(
    client: &iamClient,
    role_name: String,
) -> Result<GetRoleOutput, SdkError<GetRoleError>> {
    let response = client.get_role().role_name(role_name).send().await?;
    Ok(response)
}
// snippet-end:[rust.example_code.iam.service.get_role]

// snippet-start:[rust.example_code.iam.service.list_users]
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
// snippet-end:[rust.example_code.iam.service.list_users]

// snippet-start:[rust.example_code.iam.service.list_policies]
// snippet-start:[rust.example_code.iam.hello_lib]
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
// snippet-end:[rust.example_code.iam.hello_lib]
// snippet-end:[rust.example_code.iam.service.list_policies]

// snippet-start:[rust.example_code.iam.service.list_groups]
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
// snippet-end:[rust.example_code.iam.service.list_groups]

// snippet-start:[rust.example_code.iam.service.list_attached_role_policies]
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
// snippet-end:[rust.example_code.iam.service.list_attached_role_policies]

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

// snippet-start:[rust.example_code.iam.service.list_role_policies]
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
// snippet-end:[rust.example_code.iam.service.list_role_policies]

