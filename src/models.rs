use rocket::State;
use rocket::tokio::sync::Mutex;
use rocket::serde::json::{Json, Value, json};
use rocket::serde::{Serialize, Deserialize};
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};


#[serde(crate = "rocket::serde")]

// this is to get users from the database
#[derive(Serialize)]
pub struct Energy {
    pub energy_consumption: Vec<u64>,
    pub timestamp_consumption: Vec<u64>,
    pub energy_producing: Vec<u64>,
    pub timestamp_producing: Vec<u64>,
}

#[derive(Serialize)]
pub struct EnergyNow {
    pub energy_consumption: u64,
    pub battery_enery: u64,
    pub solar_panel_energy: u64,
}

// #[derive(Debug)]
#[derive(Serialize, Debug)]
pub struct hw_data_ints
{
    pub time: u32, // seconds since 1970 ...
    // has or doesn't
    pub has_panel: u8,
    pub has_battery: u8,
    // HW specs
    pub panel_power: u32,
    pub battery_capacity: u32,
    // HW : panel & battery values + meters' reading
    pub panel: u32,
    pub battery: u32,
    pub production: u32,
    pub consumption: u32,
}

impl Energy {
    pub fn get_now(/*conn: &PgConnection*/) -> EnergyNow {
        let mut rng = rand::thread_rng();

        EnergyNow{ energy_consumption : rng.gen_range(0..1024),
            battery_enery: rng.gen_range(0..1024),
            solar_panel_energy: rng.gen_range(0..1024)  }
    }

    pub fn get_all(/*conn: &PgConnection*/) -> Vec<Energy> {
        let mut rng = rand::thread_rng();

        let start = SystemTime::now();
        let mut since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");


        let mut energy_consumption = [0u64; 24];
        let mut energy_producing = [0u64; 24];
        let mut timestamp_consumption = [0u64; 24];
        let mut timestamp_producing = [0u64; 24];


        for i in 0..24 {
            let mut current_time = since_the_epoch.as_secs() - 3600 * (23 - i);

            energy_consumption[i as usize] = rng.gen_range(0..1024);
            energy_producing[i as usize] = rng.gen_range(0..1024);
            timestamp_consumption[i as usize] = current_time;
            timestamp_producing[i as usize] = current_time;
        }

        // let mut rng = rand::thread_rng();
        // let mut key: [u32; 24] = rng.gen_range(0..10); // up to 32 elements
        // let mut rng = Rng.gen_range(0..10);

        // let energy_consumption == rand::thread_rng().gen_range(0, 255);

        let energy_list = vec![Energy {
            // account: String::from("asd"),
            // energy_amount: -3222,
            energy_consumption: energy_consumption.to_vec(),
            energy_producing: energy_producing.to_vec(),
            timestamp_consumption: timestamp_consumption.to_vec(),
            timestamp_producing: timestamp_producing.to_vec(),
        }];
        return energy_list;
        // all_users
        //     .order(users::id.desc())
        //     .load::<User>(conn)
        //     .expect("error!")
    }
}