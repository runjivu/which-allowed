use aws_sdk_iam::error::SdkError;
use aws_sdk_iam::operation::list_policies::ListPoliciesError;
use clap::{Parser, ValueEnum};
use std::str::FromStr;

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

#[tokio::main]
async fn main() -> Result<(), SdkError<ListPoliciesError>> {
    let sdk_config = aws_config::load_from_env().await;
    let client = aws_sdk_iam::Client::new(&sdk_config);
    let default_path = "/".to_owned();
    let args = WhichAllowedArgs::parse();

    let attached_policies = match args.entity_type {
        EntityType::User => match iam_service::list_attached_user_policies(&client, args.entity_name).await {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        }
        EntityType::Role => match iam_service::list_attached_role_policies(&client, args.entity_name).await {
            Ok(r) => r,
            Err(e) => panic!("{:?}", e),
        }
    };

    println!("{:?}", attached_policies);

    Ok(())
}
