#![feature(std_misc)]
extern crate time;
extern crate daylight;
extern crate hyper;
extern crate rustc_serialize;

use time::{at, now};
use daylight::calculate_daylight;

use std::io::Read;

use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;

use rustc_serialize::json::Json;

fn main() {
    // create client
    let mut client = Client::new();

    // create request
    let mut res = client.get("http://freegeoip.net/json/")
        // set a header
        .header(Connection(vec![ConnectionOption::Close]))
        // let 'er go!
        .send().unwrap();

    // Read the response.
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let data = Json::from_str(&body).unwrap();

    let obj = data.as_object().unwrap();

    let lat = obj.get("latitude").unwrap().as_f64().unwrap();
    let long = obj.get("longitude").unwrap().as_f64().unwrap();
    let city = obj.get("city").unwrap().as_string().unwrap();
    let timezone = obj.get("time_zone").unwrap().as_string().unwrap();
    let today = now();

    let daylight = calculate_daylight(today, lat, long);

    let daylength_total_minutes = daylight.daylength.num_minutes();
    let daylength_hours = daylength_total_minutes / 60;
    let daylength_minutes = daylength_total_minutes % 60;

    println!("Sunrise and set times based on IP");
    println!("=================================");

    println!("Date:                 {}", today.asctime());
    println!("Timezone:             {}", timezone);
    println!("Latitude/Longitude:   {}/{} ({})", lat, long, city);
    println!("Declination:          {}°", daylight.declination);
    println!("Daylength:            {}:{}", daylength_hours, daylength_minutes);
    println!("Twilight AM:          {}", at(daylight.twilight_morning).asctime());
    println!("Sunrise:              {}", at(daylight.sunrise).asctime());
    println!("Noon:                 {}", at(daylight.noon).asctime());
    println!("Sunset:               {}", at(daylight.sunset).asctime());
    println!("Twilight PM:          {}", at(daylight.twilight_evening).asctime());
    println!("Sun altitude:         {}°", daylight.sun_altitude)
}
