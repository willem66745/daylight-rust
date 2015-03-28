extern crate time;
extern crate daylight;

use time::{at, now};
use daylight::calculate_daylight;

fn main() {
    let lat_apeldoorn = 52.0 + 13.0/60.0;
    let long_apeldoorn = 5.0 + 58.0/60.0;

    let daylight = calculate_daylight(now(), lat_apeldoorn, long_apeldoorn);

    println!("Today the sun sets in Apeldoorn at {}",
        at(daylight.sunset).strftime("%I:%M%p").unwrap());
}
