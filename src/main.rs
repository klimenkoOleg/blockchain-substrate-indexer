#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate core;

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
use serde::Deserializer;
use sp_keyring::AccountKeyring;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result, params};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, Transport, QoS};
use tokio::{task, time};
use std::time::Duration;
use std::error::Error;
use std::io::{BufRead, Cursor, Read};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt, LittleEndian};
use std::env;
use clap::Parser;
use std::ffi::OsStr;
use rocket::serde::json::{Json, json};
use crate::models::Energy;
use crate::models::EnergyNow;
use crate::models::hw_data_ints;
use crate::webserver::Cors;
use crate::webserver::all_options;
use crate::webserver::get_all2;
use crate::webserver::current;
use crate::webserver::get_total;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::database::write_to_db;
use rocket::config::Config;
use std::net::Ipv4Addr;
use rand::Rng; // 0.8.5
use substring::Substring;

mod models;
mod webserver;
mod database;

#[subxt::subxt(runtime_metadata_path = "metadata.scale")]
pub mod polkadot {}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// Blockchain ID of house. Should be different and passed from each Raspberry PI
    #[arg(long, default_value_t = ("DEFAULT_ADDRESS".to_owned()))]
    house_id: String,

    /// Name of the person to greet
    #[arg(long, default_value_t = ("localhost".to_owned()))]
    broker_host: String,

    /// Number of times to greet
    #[arg(long, default_value_t = 1883)]
    broker_port: u16,

    /// Number of times to greet
    #[arg(long, default_value_t = ("test/test".to_owned()))]
    broker_topic: String,

}

// fn rocket() -> rocket::Rocket {
//     dotenv().ok();
// let database_url = env::var("DATABASE_URL").expect("set DATABASE_URL");
// let pool = db::init_pool(database_url);
// rocket::ignite()
// .manage(pool)
// .mount(
//     "/api/v1/",
//     routes![get_all], //, new_user, find_user],
// )
// }


async fn readLatestMosquittoMessages(broker_host: String, broker_port: u16, broker_topic: String) {
    let rnd_mqtt_consumer = rand::thread_rng().gen_range(0..100);
    let mut mqttoptions = MqttOptions::new(
                            format!("{}{}{}","rumqtt-async", broker_topic, rnd_mqtt_consumer),
                                broker_host,
                                broker_port);
    // let mut mqttoptions = MqttOptions::new("rumqtt-async", "127.0.0.1", 9999);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client.subscribe(&broker_topic, QoS::AtMostOnce).await.unwrap();
    // client.subscribe("hardware/data", QoS::AtMostOnce).await.unwrap();

    /*task::spawn(async move {

        for i in 0..10 {
            let to_send = i * 100;
            let mut wtr = vec![];
            let mut broker_topic1 = broker_topic.clone();

            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");


            let data = hw_data_ints{
                time: since_the_epoch.as_secs() as u32,
                has_panel: 1,
                has_battery: 1,
                // nominal
                panel_power: 1024,
                battery_capacity: 5068,
                // metering
                panel: i+1,
                battery: i+11,
                production: i+101,
                consumption: i+10001,
            };

            wtr.write_u32::<LittleEndian>(data.time).unwrap();

            wtr.write_u8(data.has_panel);
            wtr.write_u8(data.has_battery);

            wtr.write_u32::<LittleEndian>(data.panel_power).unwrap();
            wtr.write_u32::<LittleEndian>(data.battery_capacity).unwrap();

            wtr.write_u32::<LittleEndian>(data.panel).unwrap();
            wtr.write_u32::<LittleEndian>(data.battery).unwrap();
            wtr.write_u32::<LittleEndian>(data.production).unwrap();
            wtr.write_u32::<LittleEndian>(data.consumption).unwrap();
            client.publish(broker_topic1, QoS::AtLeastOnce, false, wtr).await.unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });*/


    while let event = eventloop.poll().await {
        match event {
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                let pl = p.payload.clone();
                let mut rdr = Cursor::new(p.payload);


                // println!("p.payload.len: {}", pl.len());
               /* println!("time READ: {}", );
                println!("time has_panel: {}", );
                println!("time has_battery: {}", );

                println!("time panel_power: {}", );
                println!("time battery_capacity: {}", );

                println!("time panel: {}", );
                println!("time battery: {}", );
                println!("time production: {}", );
                println!("time consumption: {}", );*/
                // let panel_power1 = rdr.read_;
                let mut data1 = hw_data_ints::default();
                // let data = hw_data_ints{
                    data1.time = rdr.read_u32::<LittleEndian>().unwrap(); // 4
                    data1.has_panel= rdr.read_u16::<BigEndian>().unwrap();
                    // data1.has_panel= rdr.read_u8().unwrap();
                    data1.has_battery= rdr.read_u16::<BigEndian>().unwrap();
                data1.has_panel= 1;
                data1.has_battery= 1;
                    // data1.has_battery= rdr.read_u8().unwrap();
                    // nominal
                    data1.panel_power= rdr.read_u32::<LittleEndian>().unwrap() as u32;
                    data1.battery_capacity= rdr.read_u32::<LittleEndian>().unwrap() as u32;
                    // metering
                    data1.panel= rdr.read_u32::<LittleEndian>().unwrap();
                    data1.battery= rdr.read_u32::<LittleEndian>().unwrap();
                    data1.production= rdr.read_u32::<LittleEndian>().unwrap();
                    data1.consumption= rdr.read_u32::<LittleEndian>().unwrap();
                // };
                // let mut rdr = Cursor::new(p.payload);
                println!("Topic: {}, data: {:?}", p.topic, data1);

                let topic = p.topic;
                let pos = broker_topic.rfind('/') ;
                let house_name = broker_topic.substring(pos.unwrap_or(0) + 1, broker_topic.len());
                write_to_db(String::from(house_name), &data1);
            }
            Ok(Event::Incoming(i)) => {
                // println!("Incoming = {:?}", i);
            }
            Ok(Event::Outgoing(o)) => {
                println!("Outgoing = {:?}", o);
            },
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
    // Ok(())
}



// #[launch]
// fn rocket() -> _ {
/*fn rocket() -> _ {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    println!("Hello {}, {}, {}!", args.broker_host, args.broker_port, args.broker_topic);

    readLatestMosquittoMessages(args.broker_host, args.broker_port, args.broker_topic);


    rocket::build().mount("/api/v1/", routes![get_all2]) .launch().await?;

}*/

async fn routine(millis: u64) {
    println!("in routine({millis})");
    tokio::time::sleep(std::time::Duration::from_millis(millis)).await;
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    println!("Hello {}, {}, {}, {}", args.broker_host, args.broker_port, args.broker_topic, args.house_id);

    let mut broker_topics_split = args.broker_topic.split(",");
    // let broker_ropics: Vec<String> = broker_topics_split.collect();


    // print!("topics: {:?}", broker_ropics);

    // std::process::exit(exit_code);

    // readLatestMosquittoMessages(args.broker_host, args.broker_port, args.broker_topic);
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();

    // let task1 = rt.spawn(readLatestMosquittoMessages(args.broker_host, args.broker_port, args.broker_topic, args.house_id));

    let handles: Vec<tokio::task::JoinHandle<_>> = broker_topics_split.map(|topic| {
        rt.spawn(readLatestMosquittoMessages(args.broker_host.clone(), args.broker_port, topic.to_owned()))
    }).collect();

    // let handles: Vec<tokio::task::JoinHandle<_>> = (1..2_u64).map(|topic| {
    //     rt.spawn(readLatestMosquittoMessages(args.broker_host, args.broker_port, "asd".to_owned(), args.house_id));
    // }).collect();
    // let handles: Vec<tokio::task::JoinHandle<_>> = (1..10_u64).map(|i| {
    //     rt.spawn(routine(i))
    // }).collect();

    let config = Config {
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        ..Config::debug_default()
    };
    let task2 = rt.spawn(
        rocket::custom(config)
            // .address("0.0.0.0")
            // .port(8000)
            .attach(Cors)
            .mount("/api/v1/", routes![all_options, get_all2, current, get_total]).launch());

    rt.block_on(async {
        // task1.await.unwrap();
        task2.await.unwrap();
        // for handle in handles {
        // for handle in handles {
        //     handle.await.unwrap();
        // }
    });
    // let handles: Vec<tokio::task::JoinHandle<_>> = (1..10_u64).map(|i| {
    //     rt.spawn(routine(i))
    // }).collect();
}

/*
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    println!("Hello {}, {}, {}!", args.broker_host, args.broker_port, args.broker_topic);

    // return Ok(());


    let mut rdr = Cursor::new(vec![0, 0, 0, 1]);
    println!("{}", rdr.read_u32::<LittleEndian>().unwrap());

    // let mut wtr = vec![];
    // wtr.write_u16::<LittleEndian>(517).unwrap();
    // wtr.write_u16::<LittleEndian>(768).unwrap();
    // assert_eq!(wtr, vec![5, 2, 0, 3]);

    readLatestMosquittoMessages(args.broker_host, args.broker_port, args.broker_topic).await?;
*/
/*
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
// Ok(())
// }

fn saveEventToDatabase(conn: &Connection, fields: Result<scale_value::Composite<scale_value::scale::TypeId>, subxt::Error>) {
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
                    // print!("variant_name: {}", evt.variant_name());
                    for event in evt.field_values() {
                        println!("event: {}", event);
                    }
                    let (a, b) = (1, 5);
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
