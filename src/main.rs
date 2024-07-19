use aws_sdk_iam::operation::list_policies::ListPoliciesError;
use aws_sdk_iam::types::AttachedPolicy;
use aws_sdk_iam::error::SdkError;
use clap::{Parser, ValueEnum};
use futures::future::join_all;
use regex::Regex;
use serde_json::{to_string_pretty, Value};
use std::iter::zip;
use std::str::FromStr;
use tokio;
use urlencoding::decode;
use colored::*;
use inquire::{Select, Text};
use std::fmt::Display;
use aws_sdk_iam::Client as iamClient;

const ENTITY_TYPE: &str = "The type of IAM Entity";
const ENTITY_NAME: &str = "The name of IAM Entity";
const ACTION_NAME: &str = "The name of action IAM entity performed";
const ABOUT: &str = r#"CLI tool to check allowed actions for IAM entities.
Use it inside an environment where the cli can retrieve IAM credentials, 
which has IAMReadOnly or above permissions."#;

#[derive(Debug, clap::Parser)]
#[command(about=ABOUT)]
struct WhichAllowedArgs {
    #[arg(long, help=ENTITY_TYPE)]
    pub entity_type: Option<EntityType>,
    #[arg(long, help=ENTITY_NAME)]
    pub entity_name: Option<String>,
    #[arg(long, help=ACTION_NAME)]
    pub action_name: Option<String>,
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

impl Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            EntityType::User=> write!(f, "user"),
            EntityType::Role=> write!(f, "role"),
        }
    }
}


#[tokio::main]
async fn main() -> Result<(), SdkError<ListPoliciesError>> {
    let sdk_config = aws_config::load_from_env().await;
    let client = aws_sdk_iam::Client::new(&sdk_config);
    let args = WhichAllowedArgs::parse();

    let entity_type = match args.entity_type {
        Some(e_t) => e_t,
        None => Select::new(
            "Select the type of IAM Entity:",
            vec![EntityType::User, EntityType::Role],
        )
        .prompt()
        .unwrap(),
    };

    
    let entity_name = if let Some(e_n) = args.entity_name {
        e_n
    } else {
        set_entity_name(&client, &entity_type).await
    };

    let action_name = match args.action_name {
        Some(a_n) => a_n,
        None => Text::new("Enter the name of IAM action:")
            .prompt()
            .unwrap(),
    };


    let attached_policy_pairs: Vec<(String, String)> = match entity_type {
        EntityType::User => {
            // user attached managed policy (both aws and customer managed)
            let user_managed_p =
                iam_service::list_attached_user_policies(&client, &entity_name)
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
            let mut user_inline_p_name = iam_service::list_user_policies(&client, &entity_name)
                .await
                .unwrap();

            let user_inline_p_document = user_inline_p_name.iter().map(|p_n| async {
                let policy_document = iam_service::get_user_policy(&client, &entity_name, p_n)
                    .await
                    .unwrap();
                policy_document
            });
            let mut user_inline_p_document: Vec<String> = join_all(user_inline_p_document).await;

            // group attached managed policies
            let groups = iam_service::list_groups_for_user(&client, &entity_name)
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
                iam_service::list_attached_role_policies(&client, &entity_name)
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
            let mut role_inline_p_name = iam_service::list_role_policies(&client, &entity_name)
                .await
                .unwrap();

            let role_inline_p_document = role_inline_p_name.iter().map(|p_n| async {
                let policy_document = iam_service::get_role_policy(&client, &entity_name, p_n)
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

    //println!("{:?}", attached_policy_pairs.clone().len());

    let decoded_policy_pairs: Vec<(String, Value)> = attached_policy_pairs
        .iter()
        .filter_map(|(policy_name, policy_document)| {
            let decoded = decode(policy_document).ok()?;
            let json: Value = serde_json::from_str(&decoded).ok()?;
            Some((policy_name.clone(), json))
        })
        .collect(); 

    // println!("{:?}", attached_policy_pairs.clone().len());

    let policies_iter = decoded_policy_pairs.iter();
    let mut matching_policy = vec![];

    for (attached_policy, policy_json) in policies_iter {
        let statements = policy_json
            .get("Statement")
            .expect("Statment in a policy is expected")
            .as_array()
            .expect("Statment is always an array");

        let mut allowed_statements = vec![];

        for statement in statements {
            if check_action_in_statement(statement, &action_name) {
                allowed_statements.push(statement);
            }
        }

        if !allowed_statements.is_empty() {
            matching_policy.push(attached_policy);
            println!("[*] This policy : {}", attached_policy.bright_green().bold());
            for statement in allowed_statements {
                match to_string_pretty(statement) {
                    Ok(pretty) => println!("Statement:\n{}\nAllowed {}\n", pretty.cyan(), action_name.clone()),
                    Err(e) => eprintln!("Pretty print error: {}", e),
                }
            }
        }

    }

    if matching_policy.is_empty() {
        let message: &str = "[*] No policies allowed this action";
        println!("{}", message.bright_red().bold());
    }

    Ok(())
}

async fn set_entity_name(client: &iamClient, entity_type: &EntityType) -> String {
    let entity_list: Vec<String> = match entity_type {
        EntityType::Role => {
            let vec_role = iam_service::list_roles(client, None, None, Some(1000))
            .await
            .unwrap()
            .roles;

            vec_role.iter()
                .map(|r| r.role_name.clone())
                .collect()
        },
        EntityType::User => {
            let vec_user = iam_service::list_users(client, None, None, Some(1000))
            .await
            .unwrap()
            .users;

            vec_user.iter()
                .map(|u| u.user_name.clone())
                .collect()
        },
    };

    let autocomplete_closure = move |input: &str| {
            Ok(
                entity_list.iter()
                .filter(|e| e.to_lowercase().contains(&input.to_lowercase()))
                .map(|e| e.to_string())
                .collect::<Vec<String>>())
    };

    let result = Text::new("Enter the name of IAM Entity:")
        .with_page_size(5)
        .with_autocomplete(autocomplete_closure)
        .prompt()
        .unwrap();

    result

}

fn check_action_in_statement(statement: &Value, action_name: &String) -> bool {
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

