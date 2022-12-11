use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result, params};
use crate::models::hw_data_ints;



fn connect() -> Connection {
    // DB_CONN.
    let conn = Connection::open("data.sqlite").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS energy5(
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

pub fn write_to_db(house_id: String, data: &hw_data_ints) {
    let conn = connect();
    conn.execute(
        "INSERT INTO energy5 (house_id, time, has_panel, has_battery, panel_power, battery_capacity, panel, battery, production, consumption)
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


    // "select time, has_panel, has_battery, panel_power, battery_capacity, panel, battery, production, consumption from energy2 where time BETWEEN datetime('now', 'start of day') AND datetime('now', 'localtime');";
    -- last 24 hours
    SELECT strftime('%H', datetime(time, 'unixepoch', 'localtime')) as hour1,
    avg(panel), avg(battery), avg(production), avg(consumption)
    FROM energy5
    where house_id = 'DEFAULT_ADDRESS' and  datetime(time, 'unixepoch', 'localtime') BETWEEN datetime('now', '-1 days', 'localtime') AND datetime('now', 'localtime')
    group by strftime('%H', datetime(time, 'unixepoch', 'localtime')) ORDER by datetime(time, 'unixepoch', 'localtime');

    -- last 1 hour
    SELECT strftime('%M', datetime(time, 'unixepoch', 'localtime')) as hour1,
    avg(panel), avg(battery), avg(production), avg(consumption)
    FROM energy5
    where house_id = 'DEFAULT_ADDRESS' and  datetime(time, 'unixepoch', 'localtime') BETWEEN datetime('now', '-1 hours', 'localtime') AND datetime('now', 'localtime')
    group by strftime('%M', datetime(time, 'unixepoch', 'localtime')) ORDER by datetime(time, 'unixepoch', 'localtime');

    -- last 15 minutes
    SELECT strftime('%M', datetime(time, 'unixepoch', 'localtime')) as hour1,
    avg(panel), avg(battery), avg(production), avg(consumption)
    FROM energy5
    where house_id = 'DEFAULT_ADDRESS' and  datetime(time, 'unixepoch', 'localtime') BETWEEN datetime('now', '-15 minutes', 'localtime') AND datetime('now', 'localtime')
    group by strftime('%M', datetime(time, 'unixepoch', 'localtime')) ORDER by datetime(time, 'unixepoch', 'localtime');


    conn.close().unwrap();
}