use futures::StreamExt;
use subxt::{
    dynamic::Value,
    tx::PairSigne0neClient,
    OfflineClient,
    PolkadotConfig,
};
use subxt::ex1:H256;
use hex_literal::hex;
use serde::Deserializer;
use sp_keyr2Keyring;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result, params};
// use rumqttc::{MqttOptions, AsyncClient, , Incoming};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Transport, QoS};
use tokio::{task, time};
use std::time::Duration;
use std::error::Error;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::env;
use clap::Parser;
use std::ffi::OsStr;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

// static SOME_STR: 'static str  = "asd";

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args(String, u16, String);

async fn readLatestMosquittoMessages() -> Result<(), Box<dyn std::error::Error>> {
    let mut mqttoptions = MqttOptions::new("rumqtt-async", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe("myfirst/test", QoS::AtMostOnce).await.unwrap();

    task::spawn(async move {
        for i in 0..10 {
            let to_send = i * 100;
            let mut wtr = vec![];
            wtr.write_u32::<BigEndian>(to_send).unwrap();
            // client.publish("myfirst/test", QoS::AtLeastOnce, false, vec![i; i as usize]).await.unwrap();
            client.publish("myfirst/test", QoS::AtLeastOnce, false, wtr).await.unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });
    while let event = eventloop.poll().await {
        match event {
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                let mut rdr = Cursor::new(p.payload);
                println!("Topic: {}, Payload: {:?}", p.topic, rdr.read_u32::<BigEndian>().unwrap());
            }
            Ok(Event::Incoming(i)) => {
                println!("Incoming = {:?}", i);
            }
            Ok(Event::Outgoing(o)) => println!("Outgoing = {:?}", o),
            Err(e) => {
                println!("Error = {:?}", e);
            }
        }
        // Incoming::Publish(p)
        // opic: {}, Payload: {:?}", p.topic, p.payload

        // println!("Received = {:?}", event);
    //    event.
        // println!("Received = {:?}", notification);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // for _ in 0..args.count {
    println!("Hello {}, {}, {}!", args.broker_host, args.broker_port, args.broker_topic);
    // }

    return Ok(());
    // let args: Vec<String> = env::args().collect();
    // dbg!(args);


    let mut rdr = Cursor::new(vec![0, 0, 0, 1]);
    println!("{}", rdr.read_u32::<BigEndian>().unwrap());

    // let mut wtr = vec![];
    // wtr.write_u16::<LittleEndian>(517).unwrap();
    // wtr.write_u16::<LittleEndian>(768).unwrap();
    // assert_eq!(wtr, vec![5, 2, 0, 3]);


    readLatestMosquittoMessages().await?;
/*
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


    // mosquitto client



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
    subscribe(&api, &conn).await?; */
    Ok(())
}

fn saveEventToDatabase(conn: &Connection, fields : Result<scale_value::Composite<scale_value::scale::TypeId>, subxt::Error> ) {

    let var1 = fields.unwrap();

    let mut iter1 = var1.values();

    // let bytes = iter1.next().unwrap().value;
    // let bytes_hex = format!("0x{}", bytes.deserialize_bytes().unwrap());

    // println!("value0: {}", bytes_hex);

    let address = format!("{}", iter1.next().unwrap().value);
    let energy_amount = format!("{}", iter1.next().unwrap().value);
    let timestamp = format!("{}", iter1.next().unwrap().value);
    // let energy_amount = iter1.next().unwrap().value;
    // let timestamp = iter1.next().unwrap().value;


    conn.execute(
        "INSERT INTO energy (account, energy_amount, timestamp)
             values (?, ?, ?)",
        params![
                address,
                energy_amount,
                timestamp
            ],
    );

    // println!("value0: {}", iter1.next().unwrap().value);
    // println!("value0: {}", iter1.next().unwrap().value);
    // println!("value0: {}", iter1.next().unwrap().value);
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


                // println!("value1: {}", event.field_values().unwrap().values().next().unwrap().value);
                // println!("value2: {}", event.field_values().unwrap().values().next().unwrap().value);

                // for evt in event.field_values() {
                    // println!("event: {:?}", evt);
                    // println!("event: {:?}", evt);
                // }
                saveEventToDatabase(&conn, event.field_values());
                // let actual_fields_no_context: Vec<_> = event
                //     .field_values()
                //     .expect("can decode field values (2)")
                //     .into_values()
                //     .map(|value| value.remove_context())
                //     .collect();
                // println!("actual_fields_no_context: {:?}", actual_fields_no_context);

            }
        }
    }
    Ok(())
}
