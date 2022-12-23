use rocket::form::validate::range;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result, params};
use crate::models::hw_data_ints;
use crate::models::MeteringHistoryDto;
use crate::models::LatestMeteringDto;
use crate::models::MeteringHistorySingleDto;
use crate::models::TotalEnergyDto;
use chrono::{Datelike, Timelike, Utc, Duration};
use std::collections::{HashMap};

pub fn get_date_vect_24_hours() -> Vec<u32> {
    let now = Utc::now();
    let mut hours: Vec<u32> = Vec::with_capacity(24);
    for i in 0..24 {
        // let hour_shift = (24 as u32) -  (i as u32) - 1;
        // let val = now.checked_sub_signed(Duration::hours(hour_shift as i64)).unwrap();
        let val = now.checked_sub_signed(Duration::hours(i)).unwrap();
        hours.push(val.hour());
    }
    return hours;
}

pub fn get_date_vect_60_minutes() -> Vec<u32> {
    let now = Utc::now();
    // print!("hour: {}", now.hour());
    let mut hours: Vec<u32> = Vec::with_capacity(60);
    for i in 0..60 {
        // let shift = (60 as u32) -  (i as u32);// - 1;
        // let shift = (60 as u32) -  (i as u32);// - 1;
        // let val = now.checked_sub_signed(Duration::minutes(shift as i64)).unwrap();
        let val = now.checked_sub_signed(Duration::minutes(i)).unwrap();
        hours.push( val.minute() );
    }
    return hours;
}

pub fn get_date_vect_60_seconds() -> Vec<u32> {
    let now = Utc::now();
    let mut hours: Vec<u32> = Vec::with_capacity(60);
    for i in 0..60 {
        // let shift = (60 as u32) -  (i as u32) - 1;
        let val = now.checked_sub_signed(Duration::seconds(i)).unwrap();
        hours.push(val.second());
    }
    return hours;
}

fn connect() -> Connection {
    // DB_CONN.
    println!("create TABLE!!");
    let conn = Connection::open("data.sqlite").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS energy6 (
            house_id TEXT NOT NULL,
            time timestamp NOT NULL,
            has_panel INTEGER NOT NULL,
            has_battery INTEGER NOT NULL,
            panel_power INTEGER NOT NULL,
            battery_capacity INTEGER NOT NULL,
            panel INTEGER NOT NULL,
            battery INTEGER NOT NULL,
            production INTEGER NOT NULL,
            consumption INTEGER NOT NULL
        )",
        (), // empty list of parameters.
    ).unwrap();
    return conn;
}

pub fn get_latest_record(house_name: String) -> LatestMeteringDto {
    let where_house_id = get_house_name_sql(house_name);
    let sql = format!("select time, has_panel, has_battery, panel_power/1000., battery_capacity/1000., panel/1000., battery/1000., production/1000., consumption/1000.
                                from energy6 where {where_house_id} 1=1
                                order by time desc
                                limit 1");
    let conn = Connection::open("data.sqlite").unwrap();
    let mut stmt = conn.prepare(&*sql).unwrap();
    let mut latest_metering_iter = stmt.query_map([], |row| {
        Ok(LatestMeteringDto {
            time: row.get(0).unwrap(),
            has_panel: row.get(1).unwrap(),
            has_battery: row.get(2).unwrap(),
            // HW specs
            panel_power: row.get(3).unwrap(),
            battery_capacity: row.get(4).unwrap(),
            // HW : panel & battery values + meters' reading
            panel: row.get(5).unwrap(),
            battery: row.get(6).unwrap(),
            production: row.get(7).unwrap(),
            consumption: row.get(8).unwrap(),
        })
    }).unwrap();
    let metering_next = latest_metering_iter.next();
    let metering = metering_next.unwrap_or(Ok(LatestMeteringDto::get_empty()));
    // conn.close().unwrap();
    return metering.unwrap();
}

pub fn get_db_total(time_param: String, house_name: String) -> TotalEnergyDto {

    let where_house_id = get_house_name_sql(house_name);

    let sql =  format!("select round(sum(e.consumption)/1000., 2) as internal_consumption,
	round(sum(e.production)/1000., 2) as internal_production,
	round(sum( case WHEN e.consumption - e.production < 0 then - e.consumption + e.production
			else 0
			end )/1000, 2) as external_production,
	   round(sum( case WHEN e.consumption - e.production > 0 then e.consumption - e.production
			else 0
			end )/1000, 2) as external_consumption
	from energy6 e
	where {where_house_id} datetime(time, 'unixepoch', 'localtime') BETWEEN datetime('now', '-1 {time_param}', 'localtime') AND datetime('now', 'localtime')
    limit 1");
    let conn = Connection::open("data.sqlite").unwrap();
    let mut stmt = conn.prepare(&*sql).unwrap();

    let mut latest_metering_iter = stmt.query_map([], |row| {


        let mut external_production_energy: f64;
        external_production_energy = row.get(2).unwrap() ;
        // external_production_money1 = external_consumption_money1 * 2;

        let mut external_consumption_energy: f64;
        external_consumption_energy = row.get(3).unwrap() ;
        // external_consumption_money1 = external_consumption_money1 * 10.;

        Ok(TotalEnergyDto {
            internal_consumption: row.get(0).unwrap(),
            internal_production: row.get(1).unwrap(),

            external_production_energy: external_production_energy,
            external_consumption_energy: external_consumption_energy,
            external_production_money: external_production_energy * 2.,
            external_consumption_money: external_consumption_energy * 10.,
        })
    }).unwrap();
    let metering_next = latest_metering_iter.next();
    let metering = metering_next.unwrap_or(Ok(TotalEnergyDto::get_empty()));
    // conn.close().unwrap();
    return metering.unwrap();
}

fn get_house_name_sql(house_name: String) -> String {
    match house_name.as_str() {
        "all" => {
            "".to_owned()
        }
        _ => format!(" house_id='{}' and ", house_name),
    }
}

pub fn get_db_history(group_param: String, time_back_range: String, time_grouping_ticks : &Vec<u32>, house_name: String) -> MeteringHistoryDto {
    let where_house_id: String = get_house_name_sql(house_name);

    let sql =  format!("SELECT CAST(strftime('{group_param}', datetime(time, 'unixepoch', 'localtime')) as INT) as hour1, \
                                round(avg(panel)/1000., 2), round(avg(battery)/1000., 2), round(avg(production)/1000., 2), round(avg(consumption)/1000., 2) \
                                FROM energy6 \
                                where {where_house_id} datetime(time, 'unixepoch', 'localtime') BETWEEN datetime('now', '{time_back_range}', 'localtime') AND datetime('now', 'localtime') \
                                group by strftime('{group_param}', datetime(time, 'unixepoch', 'localtime')) ORDER by datetime(time, 'unixepoch', 'localtime') DESC limit 1000");
    let conn = Connection::open("data.sqlite").unwrap();
    let mut stmt = conn.prepare(&*sql).unwrap();
    let hours_map : HashMap<u32, Vec<MeteringHistorySingleDto>> = time_grouping_ticks
        .iter()
        .map(|&hour|  (hour, Vec::<MeteringHistorySingleDto>::new()))
        .collect();


    let mut hours_map2 : HashMap<u32, MeteringHistorySingleDto> = HashMap::new();
    let meterings_iter = stmt.query_map([], |row| {
        let time_groupping1 = row.get(0).unwrap();
        print!("time_groupping1: {}", time_groupping1);
        let mut vec1 = hours_map.get(&time_groupping1).unwrap();
        // print!("time_groupping1: {}", time_groupping1);
        hours_map2.insert(time_groupping1, MeteringHistorySingleDto {
            time_groupping: time_groupping1,
            avg_panel: row.get(1).unwrap(),
            avg_battery: row.get(2).unwrap(),
            avg_production: row.get(3).unwrap(),
            avg_consumption: row.get(4).unwrap(),
        });
        /*vec1.push(MeteringHistorySingleDto {
            time_groupping: time_groupping1,
            avg_panel: row.get(1).unwrap(),
            avg_battery: row.get(2).unwrap(),
            avg_production: row.get(3).unwrap(),
            avg_consumption: row.get(4).unwrap(),
        });*/
        Ok(1)
    }).unwrap();
    // if (group_param ) {
    let result: Vec<u32> = meterings_iter.map(|x| x.unwrap() ).collect();

    let mut avg_panel1 = Vec::new();
    let mut avg_battery1 = Vec::new();
    let mut avg_production1 = Vec::new();
    let mut avg_consumption1 = Vec::new();
    for i in time_grouping_ticks {
        let val = hours_map2.get(&i);
        if val.is_some() {
            let val1 = val.unwrap();
            avg_panel1.push(val1.avg_panel);
            avg_battery1.push(val1.avg_battery);
            avg_production1.push(val1.avg_production);
            avg_consumption1.push(val1.avg_consumption);
        } else {
            avg_panel1.push(0.);
            avg_battery1.push(0.);
            avg_production1.push(0.);
            avg_consumption1.push(0.);
        }
    }

    let result = MeteringHistoryDto {
        avg_panel: avg_panel1,
        avg_battery: avg_battery1,
        avg_production: avg_production1,
        avg_consumption: avg_consumption1,
    };

    // print!("result!!!: {:?}", result);
    // }
    // let result: Vec<MeteringHistoryDto> = meterings_iter.map(|x| x.unwrap() ).collect();
    // conn.close().unwrap();
    // return result;
    return result;
}

pub fn write_to_db(house_id: String, data: &hw_data_ints) {
    let conn = connect();
    conn.execute(
        "INSERT INTO energy6 (house_id, time, has_panel, has_battery, panel_power, battery_capacity, panel, battery, production, consumption)
             values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
                house_id,
                data.time,
                data.has_panel,
                data.has_battery,
                data.panel_power,
                data.battery_capacity,
                data.panel,
                data.battery,
                data.production,
                data.consumption,
            ],
    ).unwrap();
    conn.close().unwrap();
}