use aws_sdk_iam::operation::list_policies::ListPoliciesError;
use aws_sdk_iam::types::{AttachedPolicy, Policy, PolicyVersion};
use aws_sdk_iam::{error::SdkError, operation::get_policy};
use clap::{Parser, ValueEnum};
use futures::future::join_all;
use futures::stream::Zip;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};
use std::iter::zip;
use std::str::FromStr;
use tokio;
use urlencoding::decode;

const PATH_PREFIX_HELP: &str = "The path prefix for filtering the results.";
const ENTITY_TYPE: &str = "The type of IAM Entity: Is Either \"role\" or \"user\".";
const ENTITY_NAME: &str = "The name of IAM Entity";
const ACTION_NAME: &str = "The name of action IAM entity performed";

#[derive(Debug, clap::Parser)]
#[command(about)]
struct WhichAllowedArgs {
    #[arg(long, help=ENTITY_TYPE)]
    pub entity_type: EntityType,
    #[arg(long, help=ENTITY_NAME)]
    pub entity_name: String,
    #[arg(long, help=ACTION_NAME)]
    pub action_name: String,
}

#[derive(Debug, Clone, ValueEnum)]
enum EntityType {
    User,
    Role,
}

impl FromStr for EntityType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(EntityType::User),
            "role" => Ok(EntityType::Role),
            _ => Err(format!(
                "'{}' is not a valid entity type. Use 'user' or 'role'.",
                s
            )),
        }
    }
}

// #[derive(Serialize, Deserialize, Debug)]
// struct Statement {
//     Sid: String,
//     Effect: String,
//     Action: String,
//     Resource: String,
// }
//
// #[derive(Serialize, Deserialize, Debug)]
// struct PolicyDocument {
//     Version: String,
//     Statement: Vec<Statement>,
// }

fn check_action_in_statment(statement: &Value, action_name: &String) -> bool {
    if statement
        .get("Effect")
        .expect("Effect is expected")
        .as_str()
        == Some("Allow")
    {
        let action = statement
            .get("Action")
            .expect("Action is expected in statement");

        match action {
            Value::String(s) => {
                if s == action_name {
                    return true;
                }
                let pattern = s.replace("*", ".*");
                let re = Regex::new(&pattern).expect("Invalid regex pattern");
                re.is_match(&action_name)
            }

            Value::Array(arr) => {
                for action_elem in arr.clone() {
                    let action_str = action_elem
                        .as_str()
                        .expect("Action Array must be an array of string");
                    if action_str == action_name {
                        return true;
                    }
                    let pattern = action_str.replace("*", ".*");
                    let re = Regex::new(&pattern).expect("Invalid regex pattern");
                    if re.is_match(&action_name) {
                        return true;
                    }
                }
                false
            }

            _ => false,
        }
    } else {
        false
    }
}

#[tokio::main]
async fn main() -> Result<(), SdkError<ListPoliciesError>> {
    let sdk_config = aws_config::load_from_env().await;
    let client = aws_sdk_iam::Client::new(&sdk_config);
    let args = WhichAllowedArgs::parse();

    let attached_policy_pairs: Vec<(String, String)> = match args.entity_type {
        EntityType::User => {
            // user attached managed policy (both aws and customer managed)
            let user_managed_p =
                iam_service::list_attached_user_policies(&client, &args.entity_name)
                    .await
                    .unwrap()
                    .unwrap();

            let user_managed_p_document = user_managed_p.iter().map(|a_p| async {
                let policy = iam_service::get_policy(&client, a_p.clone()).await.unwrap();
                let policy_document = iam_service::get_policy_version(&client, policy)
                    .await
                    .unwrap()
                    .document
                    .unwrap();
                policy_document
            });
            let mut user_managed_p_document: Vec<String> = join_all(user_managed_p_document).await;

            let mut user_managed_p_name: Vec<String> = user_managed_p
                .iter()
                .map(|p| p.policy_name.clone().unwrap())
                .collect();

            // user attached inline policy
            let mut user_inline_p_name = iam_service::list_user_policies(&client, &args.entity_name)
                .await
                .unwrap();

            let user_inline_p_document = user_inline_p_name.iter().map(|p_n| async {
                let policy_document = iam_service::get_user_policy(&client, &args.entity_name, p_n)
                    .await
                    .unwrap();
                policy_document
            });
            let mut user_inline_p_document: Vec<String> = join_all(user_inline_p_document).await;

            // group attached managed policies
            let groups = iam_service::list_groups_for_user(&client, &args.entity_name)
                .await
                .unwrap();

            let mut group_managed_p: Vec<AttachedPolicy> = Vec::new();
            for group in groups.clone() {
                let mut p = iam_service::list_attached_group_policies(&client, &group)
                    .await
                    .unwrap()
                    .unwrap();
                group_managed_p.append(&mut p);
            }

            let group_managed_p_document = group_managed_p.iter().map(|a_p| async {
                let policy = iam_service::get_policy(&client, a_p.clone()).await.unwrap();
                let policy_document = iam_service::get_policy_version(&client, policy)
                    .await
                    .unwrap()
                    .document
                    .unwrap();
                policy_document
            });
            let mut group_managed_p_document = join_all(group_managed_p_document).await;

            let mut group_managed_p_name: Vec<String> = group_managed_p
                .iter()
                .map(|a_p| a_p.policy_name.clone().unwrap())
                .collect();

            // group attached inline policies
            let mut group_inline_p_name: Vec<String> = Vec::new();
            let mut group_inline_p_document: Vec<String> = Vec::new();
            for group in groups {
                let mut p_n = iam_service::list_group_policies(&client, &group)
                    .await
                    .unwrap();

                let p_d = p_n.iter().map(|p_n| async {
                    iam_service::get_group_policy(&client, &group, p_n)
                        .await
                        .unwrap()
                });
                let mut p_d = join_all(p_d).await;

                group_inline_p_name.append(&mut p_n);
                group_inline_p_document.append(&mut p_d);
            }

            let mut user_p_name: Vec<String> = Vec::new();
            user_p_name.append(&mut user_managed_p_name);
            user_p_name.append(&mut user_inline_p_name);
            user_p_name.append(&mut group_managed_p_name);
            user_p_name.append(&mut group_inline_p_name);

            let mut user_p_document: Vec<String> = Vec::new();
            user_p_document.append(&mut user_managed_p_document);
            user_p_document.append(&mut user_inline_p_document);
            user_p_document.append(&mut group_managed_p_document);
            user_p_document.append(&mut group_inline_p_document);

            assert_eq!(user_p_name.len(), user_p_document.len());
            let user_result: Vec<(String, String)> = zip(user_p_name, user_p_document).collect();
            user_result
        }

        EntityType::Role => {
            let role_managed_p =
                iam_service::list_attached_role_policies(&client, &args.entity_name)
                    .await
                    .unwrap()
                    .unwrap();

            let role_managed_p_document = role_managed_p.iter().map(|a_p| async {
                let policy = iam_service::get_policy(&client, a_p.clone()).await.unwrap();
                let policy_document = iam_service::get_policy_version(&client, policy)
                    .await
                    .unwrap()
                    .document
                    .unwrap();
                policy_document
            });
            let mut role_managed_p_document: Vec<String> = join_all(role_managed_p_document).await;

            let mut role_managed_p_name: Vec<String> = role_managed_p
                .iter()
                .map(|p| p.policy_name.clone().unwrap())
                .collect();

            // role attached inline policy
            let mut role_inline_p_name = iam_service::list_role_policies(&client, &args.entity_name)
                .await
                .unwrap();

            let role_inline_p_document = role_inline_p_name.iter().map(|p_n| async {
                let policy_document = iam_service::get_role_policy(&client, &args.entity_name, p_n)
                    .await
                    .unwrap();
                policy_document
            });
            let mut role_inline_p_document: Vec<String> = join_all(role_inline_p_document).await;

            let mut role_p_name: Vec<String> = Vec::new();
            role_p_name.append(&mut role_managed_p_name);
            role_p_name.append(&mut role_inline_p_name);

            let mut role_p_document: Vec<String> = Vec::new();
            role_p_document.append(&mut role_managed_p_document);
            role_p_document.append(&mut role_inline_p_document);

            assert_eq!(role_p_name.len(), role_p_document.len());
            let role_result: Vec<(String, String)> = zip(role_p_name, role_p_document).collect();
            role_result
        }
    };


    //println!("{:?}", attached_policy_pairs);

    println!("{:?}", attached_policy_pairs.clone().len());

    let decoded_policy_pairs: Vec<(String, Value)> = attached_policy_pairs
        .iter()
        .filter_map(|(policy_name, policy_document)| {
            let decoded = decode(policy_document).ok()?;
            let json: Value = serde_json::from_str(&decoded).ok()?;
            Some((policy_name.clone(), json))
        })
        .collect(); 

    println!("{:?}", attached_policy_pairs.clone().len());

    let policies_iter = decoded_policy_pairs.iter();

    for (attached_policy, policy_json) in policies_iter {
        let statements = policy_json
            .get("Statement")
            .expect("Statment in a policy is expected")
            .as_array()
            .expect("Statment is always an array");
        println!(
            "[*] Checking policy : {}",
            attached_policy
        );

        for statement in statements {
            if check_action_in_statment(statement, &args.action_name) {
                match to_string_pretty(statement) {
                    Ok(pretty) => println!(
                        "Statement : \n{}\nallowed {}",
                        pretty,
                        args.action_name.clone()
                    ),
                    Err(e) => eprintln!("Pretty print error : {}", e),
                }
            }
        }
    }

    Ok(())
}
