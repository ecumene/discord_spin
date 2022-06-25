use anyhow::Result;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;
use ed25519_dalek::Verifier;
use spin_sdk::{
    http::{Request, Response},
    http_component,
};

const DISCORD_PUB_KEY: &str = "DISCORD_PUB_KEY";

/// A simple Spin HTTP component.
#[http_component]
fn hello_world(req: Request) -> Result<Response> {
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

    println!("1");
    let public_key = PublicKey::from_bytes(pub_key.as_bytes())?;
    let signature = Signature::from_bytes(&hex::decode(signature)?)?;
    println!("2");

    if public_key
        .verify(
            format!("{}{}", timestamp.to_str()?, body).as_bytes(),
            &signature,
        )
        .is_ok()
    {
        Ok(http::Response::builder()
            .status(200)
            .body(Some("{\"type\": 1}".into()))?)
    } else {
        Ok(http::Response::builder()
            .status(400)
            .body(Some("Bad Signature".into()))?)
    }
}
