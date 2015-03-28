[![Build Status][status]](https://travis-ci.org/willem66745/daylight-rust)

# daylight-rust

Very simple Rust library to calculate the moment of sunrise, sunset and
twilight times at a given date, latitude and longitude. The used algorithms to
calculate the timestamps are based on http://www.sci.fi/~benefon/rscalc.c.

```rust
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
```
