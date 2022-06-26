use anyhow::{Context, Result};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use serde::{Deserialize, Serialize};
use spin_sdk::{
    http::{Request, Response},
    http_component,
};

type Snowflake = str;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct DiscordInteraction<'a> {
    id: &'a Snowflake,
    application_id: &'a Snowflake,
    #[serde(rename = "type")]
    command_type: u8,
    guild_id: Option<&'a Snowflake>,
    channel_id: Option<&'a Snowflake>,
    // TODO: types
    // member: &'a str,
    // user: Option<&'a str>,
    token: &'a str,
    version: u8,
    message: Option<&'a str>,
}

impl<'a> DiscordInteraction<'a> {
    // https://discord.com/developers/docs/interactions/receiving-and-responding#followup-messages
    pub fn reply(&self, message: &str) -> Result<String> {
        Ok(std::str::from_utf8(
            spin_sdk::outbound_http::send_request(
                http::Request::builder()
                    .method("POST")
                    .header("Content-Type", "application/json")
                    // TODO: types
                    .uri(format!(
                        "https://discord.com/api/v10/interactions/{}/{}/callback",
                        self.id, self.token
                    ))
                    .body(Some(
                        format!("{{\"type\":4,\"data\":{{\"content\":\"{}\"}}}}", message).into(),
                    ))
                    .unwrap(),
            )?
            .body()
            .as_ref()
            .expect("Empty body"),
        )?
        .to_owned())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DiscordChoice<'a> {
    name: &'a str,
    value: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
struct DiscordOption<'a> {
    name: &'a str,
    description: &'a str,
    #[serde(rename = "type")]
    command_type: u8,
    required: bool,
    choices: Option<Vec<DiscordChoice<'a>>>,
}
// TODO: example for these
impl<'a> DiscordOption<'a> {
    pub fn new(
        name: &'a str,
        description: &'a str,
        command_type: u8,
        required: bool,
    ) -> DiscordOption<'a> {
        DiscordOption {
            name,
            description,
            command_type,
            required,
            choices: None,
        }
    }

    pub fn add_choice(&mut self, choice: DiscordChoice<'a>) {
        if let Some(choices) = &mut self.choices {
            choices.push(choice);
        } else {
            self.choices = Some(vec![choice]);
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiscordCommand<'a> {
    name: &'a str,
    description: &'a str,
    options: Option<DiscordOption<'a>>,
}

impl<'a> DiscordCommand<'a> {
    pub fn new(name: &'a str, description: &'a str) -> DiscordCommand<'a> {
        DiscordCommand {
            name,
            description,
            options: None,
        }
    }
}

const DISCORD_PUB_KEY: &str = "DISCORD_PUB_KEY";
const DISCORD_BOT_TOKEN: &str = "DISCORD_BOT_TOKEN";

pub fn send_command(app_id: &str, command: DiscordCommand) -> Result<String> {
    let bot_token = std::env::var(DISCORD_BOT_TOKEN).expect("Couldn't find Discord Bot Token.");
    let json = serde_json::to_string(&command)?;
    let bytes = json.into();
    Ok(std::str::from_utf8(
        spin_sdk::outbound_http::send_request(
            http::Request::builder()
                .method("POST")
                .header("Authorization", format!("Bot {}", bot_token))
                .header("Content-Type", "application/json")
                .uri(format!(
                    "https://discord.com/api/v10/applications/{}/commands",
                    app_id
                ))
                .body(Some(bytes))
                .unwrap(),
        )?
        .body()
        .as_ref()
        .expect("Empty body"),
    )?
    .to_owned())
}

// https://discord.com/developers/docs/interactions/receiving-and-responding#interactions-and-bot-users
#[http_component]
fn handle_interaction(req: Request) -> Result<Response> {
    let pub_key = std::env::var(DISCORD_PUB_KEY).expect("Couldn't find Discord Pub Key.");
    let signature = req
        .headers()
        .get("x-signature-ed25519")
        .expect("No signature...");
    let timestamp = req
        .headers()
        .get("x-signature-timestamp")
        .expect("No timestamp...");
    let body = std::str::from_utf8(req.body().as_ref().expect("No body")).expect("Non-utf-8 body");

    let public_key = PublicKey::from_bytes(&hex::decode(pub_key).unwrap()).unwrap();
    let signature = Signature::from_bytes(&hex::decode(signature).unwrap()).unwrap();

    if public_key
        .verify(
            format!(
                "{}{}",
                timestamp.to_str().context("timestamp header to str")?,
                body
            )
            .as_bytes(),
            &signature,
        )
        .is_ok()
    {
        let event = serde_json::from_str::<DiscordInteraction>(body).expect("invalid interaction");

        match event.command_type {
            1 => {
                send_command(
                    event.application_id,
                    DiscordCommand::new("hello", "hello world"),
                )
                .unwrap();
            }
            2 => {
                event.reply("Hello from Spin!").unwrap();
            }
            _ => {
                panic!("Unknown type.");
            }
        }

        Ok(http::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Some("{\"type\": 1}".into()))?)
    } else {
        Ok(http::Response::builder()
            .status(400)
            .body(Some("Bad Signature".into()))?)
    }
}
