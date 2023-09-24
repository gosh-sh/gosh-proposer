use crate::gosh::burn::find_burns;
use common::gosh::helper::create_client;
use std::env;

mod gosh;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let context = create_client()?;
    let root_address = env::var("ROOT_ADDRESS")?;
    let burns = find_burns(&context, &root_address).await?;
    println!("burns: {burns:?}");
    Ok(())
}
