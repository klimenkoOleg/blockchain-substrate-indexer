use futures::StreamExt;
use subxt::{
    dynamic::Value,
    tx::PairSigner,
    OnlineClient,
    OfflineClient,
    PolkadotConfig,
};
use subxt::ext::sp_core::H256;
use hex_literal::hex;
use sp_keyring::AccountKeyring;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result, params};

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // DB_CONN.
    let conn = Connection::open("data.sqlite")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS energy(
            account TEXT NOT NULL,
            energy_amount TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        (), // empty list of parameters.
    )?;





    // Create a client to use:
    let api = OnlineClient::<PolkadotConfig>::new().await?;
    let signer = PairSigner::new(AccountKeyring::Alice.pair());
    let timestamp: u64 = 555;

    // Create a transaction to submit:
    let tx = subxt::dynamic::tx(
        "EnergyMeterModule",
        "store_energy",
        vec![
            Value::u128(555),
            Value::u128(555),
        ],
    );
    // submit the transaction with default params:
    let hash = api.tx().sign_and_submit_default(&tx, &signer).await?;
    println!("store_energy: {}", hash);

    indexLastFinilizedBlocks(&api, &conn).await?;
    subscribe(&api, &conn).await?;
    Ok(())
}

fn saveEventToDatabase(conn: &Connection, fields : Result<scale_value::Composite<scale_value::scale::TypeId>, subxt::Error> ) {
    conn.execute(
        "INSERT INTO people (name, sex)
             values (?, ?, ?)",
        params![
                "Oleg",
                "Sometimes",
            ],
    );
}

async fn indexLastFinilizedBlocks(api: &OnlineClient<PolkadotConfig>, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    let mut finalized_hash = api.rpc().finalized_head().await?;
    let mut block = api.blocks().at(Some(finalized_hash)).await?;

    for i in 1..30 {
        let block_number = block.header().number;
        let block_hash = block.hash();
        let body = block.body().await?;
        for ext in body.extrinsics() {
            let idx = ext.index();
            let events = ext.events().await?;
            let bytes_hex = format!("0x{}", hex::encode(ext.bytes()));
            for evt in events.iter() {
                let evt = evt?;
                let pallet_name = evt.pallet_name();
                let event_name = evt.variant_name();
                if ("MeteringSaved" == event_name) {
                    println!("        {pallet_name}_{event_name}");
                    print!("variant_name: {}", evt.variant_name());
                    for event in evt.field_values() {
                        println!("event: {}", event);
                    }
                    let (a, b) = (1 , 5);
                    saveEventToDatabase(&conn, evt.field_values());
                }
            }
        }
        finalized_hash = block.header().parent_hash;
        block = api.blocks().at(Some(finalized_hash)).await?;
    }
    Ok(())
}

// async fun create_table() {
//
// }

async fn subscribe(api: &OnlineClient<PolkadotConfig>, conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
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
            let pallet_name = event.pallet_name();
            let event_name = event.variant_name();
            if ("MeteringSaved" == event_name) {
                println!("        {pallet_name}_{event_name}");
                println!("variant_name: {}", event.variant_name());

                // println!("event.field_values(): {}", event.field_values()[0]);

                let var1 = event.field_values().unwrap();

                let mut iter1 = var1.values();

                let bytes = iter1.next().unwrap().value;
                let bytes_hex = format!("0x{}", hex::encode(bytes));

                println!("value0: {}", bytes_hex);
                println!("value0: {}", iter1.next().unwrap().value);
                println!("value0: {}", iter1.next().unwrap().value);
                // println!("value1: {}", event.field_values().unwrap().values().next().unwrap().value);
                // println!("value2: {}", event.field_values().unwrap().values().next().unwrap().value);

                for evt in event.field_values() {
                    // println!("event: {:?}", evt);
                    // println!("event: {:?}", evt);
                }
                let (a, b) = (1 , 5);
                saveEventToDatabase(&conn, event.field_values());
                let actual_fields_no_context: Vec<_> = event
                    .field_values()
                    .expect("can decode field values (2)")
                    .into_values()
                    .map(|value| value.remove_context())
                    .collect();
                // println!("actual_fields_no_context: {:?}", actual_fields_no_context);

            }
        }
    }
    Ok(())
}
