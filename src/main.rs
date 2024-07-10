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

    let attached_policies = match args.entity_type {
        EntityType::User => {
            match iam_service::list_attached_user_policies(&client, args.entity_name).await {
                Ok(r) => r,
                Err(e) => panic!("{:?}", e),
            }
        }
        EntityType::Role => {
            match iam_service::list_attached_role_policies(&client, args.entity_name).await {
                Ok(r) => r,
                Err(e) => panic!("{:?}", e),
            }
        }
    };

    let attached_policies = attached_policies.expect("None here shouldn't happen");
    // println!("{:?}", attached_policies);

    let allowing_policies = attached_policies.clone()
        .into_iter()
        .map(|a_p: AttachedPolicy| async {
            let policy = match iam_service::get_policy(&client, a_p).await {
                Ok(p) => p,
                Err(e) => panic!("{:?}", e),
            };

            match iam_service::get_policy_version(&client, policy).await {
                Ok(p_v) => p_v,
                Err(e) => panic!("{:?}", e),
            }
        });
    // .filter() // Vec<PolicyVersion> -> Vec<PolicyVersion> that contains action
    let allowing_policies: Vec<PolicyVersion> = join_all(allowing_policies).await;

    //println!("{:?}", allowing_policies);

    let allowing_policies_json: Vec<Value> = allowing_policies
        .iter()
        .filter_map(|policy_version| {
            policy_version.document.as_ref().map(|doc| {
                let decoded = decode(doc).expect("Failed to decode document");
                let json: Value = serde_json::from_str(&decoded).expect("Failed to parse JSON");
                json
            })
        })
        .collect();


    println!("{:?}", attached_policies.clone().len());
    println!("{:?}", allowing_policies_json.clone().len());
    let policies_iter = zip(attached_policies.clone(), allowing_policies_json);

    for (attached_policy, policy_json) in policies_iter {
        let statements = policy_json
            .get("Statement")
            .expect("Statment in a policy is expected")
            .as_array()
            .expect("Statment is always an array");
        println!("[*] Checking policy : {}", attached_policy.policy_name().unwrap());

        for statement in statements {
            if check_action_in_statment(statement, &args.action_name) {
                match to_string_pretty(statement) {
                    Ok(pretty) => println!("Statement : \n{}\nallowed {}", pretty, args.action_name.clone()),
                    Err(e) => eprintln!("Pretty print error : {}", e),
                }
            }
        }
    }

    Ok(())
}
