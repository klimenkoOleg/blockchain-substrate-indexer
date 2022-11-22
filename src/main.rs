// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! To run this example, a local polkadot node should be running. Example verified against polkadot 0.9.29-41a9d84b152.
//!
//! E.g.
//! ```bash
//! curl "https://github.com/paritytech/polkadot/releases/download/v0.9.29/polkadot" --output /usr/local/bin/polkadot --location
//! polkadot --dev --tmp
//! ```

use futures::StreamExt;
use subxt::{
    OnlineClient,
    OfflineClient,
    PolkadotConfig,
};
// use subxt::{ClientBuilder, DefaultConfig, SubstrateExtrinsicParams};
use subxt::ext::sp_core::H256;
use hex_literal::hex;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Create a client to use:
    let api = OnlineClient::<PolkadotConfig>::new().await?;

    // print!("api: {:?}", api);

    // let api2 = ClientBuilder::new()
    //     .set_url("ws://localhost:11144")
    //     .build()
    //     .await?
    //     .to_runtime_api::<runtime::RuntimeApi<DefaultConfig, SubstrateExtrinsicParams<DefaultConfig>>>();


    // let events = api.blocks().at(hex!("a63635d4411d6f9185cfc76a834915b17eb4360a3036477fe30c40a095ad65d1")).await?.events();
    // let hash: Option<H256> = Some(hex!("7804a29cde7251b3fe4ffecc7f24980aa1dfff0ecb46f13deef2a121197ed98f").into());


    // let hash: Option<H256> = Some(H256.rando).into());
    // let events = api.storage().system().events(hash).await?;

    // let events = api.blocks().at(hash).await?.events().await?;
    // let events = api.blocks().at(hash).events().await?;

    let mut finalized_hash = api.rpc().finalized_head().await?;
    let mut block = api.blocks().at(Some(finalized_hash)).await?;


    for i in 1..30 {

        // block.header().parent_hash
        // api.blocks()


        // println!("{}", &events[0].event);


        // let block_number = block.header().number;
        // let block_hash = block.hash();

        // println!("Block #{block_number}:");
        // println!("  Hash: {block_hash}");
        // println!("  Extrinsics:");

        // if let runtime::runtime_types::snowbase_runtime::Event::BasicOutboundChannel(ev) = &events[0].event {
        //     if let runtime::runtime_types::snowbridge_basic_channel::outbound::pallet::Event::Committed { .. }  = ev {
        //         println!("found Committed event");
        //     }
        // }


        // Subscribe to all finalized blocks:
        // let mut blocks_sub = api.blocks().subscribe_finalized().await?;

        // while let Some(block) = blocks_sub.next().await {
        //     let block = block?;

        let block_number = block.header().number;
        let block_hash = block.hash();

        // println!("Block #{block_number}:");
        // println!("  Hash: {block_hash}");
        // println!("  Extrinsics:");

        let body = block.body().await?;
        for ext in body.extrinsics() {
            let idx = ext.index();
            let events = ext.events().await?;
            let bytes_hex = format!("0x{}", hex::encode(ext.bytes()));

            // println!("    Extrinsic #{idx}:");
            // println!("      Bytes: {bytes_hex}");
            // println!("      Events:");

            for evt in events.iter() {
                let evt = evt?;

                let pallet_name = evt.pallet_name();
                let event_name = evt.variant_name();
                // println!("event!!!: {:?}", evt.field_values());
                if ("MeteringSaved" == event_name) {
                    println!("        {pallet_name}_{event_name}");
                    for event in evt.field_values() {
                        println!("event: {}", event);
                        // for x in event.values() {
                        //     println!("event value x: {}", event);
                        // }
                    }
                }
            }
        }
        // }
        // :subxt::ext::sp_core::H256
        finalized_hash = block.header().parent_hash;
        block = api.blocks().at(Some(finalized_hash)).await?;
    }

    // Subscribe to (in this case, finalized) blocks.
    let mut block_sub = api.blocks().subscribe_finalized().await?;

    // Get each finalized block as it arrives.
    while let Some(block) = block_sub.next().await {
        let block = block?;

        // Ask for the events for this block.
        let events = block.events().await?;

        let block_hash = block.hash();

        // We can dynamically decode events:
        println!("  Dynamic event details from block: {block_hash:?}:");
        for event in events.iter() {
            let event = event?;
            let is_balance_transfer = event
                .as_event::<polkadot::balances::events::Transfer>()?
                .is_some();
            // let pallet = event.pallet_name();
            // let variant = event.variant_name();
            // println!(
            //     "    {pallet}::{variant} (is balance transfer? {is_balance_transfer})"
            // );


            let pallet_name = event.pallet_name();
            let event_name = event.variant_name();
            // println!("event!!!: {:?}", evt.field_values());
            if ("MeteringSaved" == event_name) {
                println!("        {pallet_name}_{event_name}");
                for evt in event.field_values() {
                    println!("event: {}", evt);
                    // for x in event.values() {
                    //     println!("event value x: {}", event);
                    // }
                }
            }
        }

        // Or we can find the first transfer event, ignoring any others:
        // let transfer_event =
        //     events.find_first::<polkadot::balances::events::Transfer>()?;
        //
        // if let Some(ev) = transfer_event {
        //     println!("  - Balance transfer success: value: {:?}", ev.amount);
        // } else {
        //     println!("  - No balance transfer event found in this block");
        // }
    }

    Ok(())
}
