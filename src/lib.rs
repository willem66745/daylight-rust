// Based on: http://www.sci.fi/~benefon/rscalc.c
//
// C program calculating the sunrise and sunset for
// the current date and a fixed location(latitude,longitude)
// Note, twilight calculation gives insufficient accuracy of results
// Jarmo Lammi 1999 - 2001
//
// Kept the function names simular for future reference

#![feature(std_misc)]
#![feature(core)]

extern crate time;

use time::{Timespec, Tm};
use std::num::Float;
use std::f64::consts;

const SUNRADIUS: f64 = 0.53;
const AIRREFR: f64 = 34.0/60.0;
const Y2000: Tm = Tm {tm_sec: 0, tm_min: 0, tm_hour: 0, tm_mday: 1, tm_mon: 0, tm_year: 100,
                      tm_wday: 6, tm_yday: 0, tm_isdst: 0, tm_utcoff: 0, tm_nsec: 0};

/// Result of the daylight calculation (calculated times are UTC based)
#[derive(Clone, Copy, Debug)]
pub struct Daylight {
    pub twilight_morning: Timespec,
    pub sunrise: Timespec,
    pub sunset: Timespec,
    pub twilight_evening: Timespec
}

/// the function below returns an angle in the range 0 to 2*pi
fn fnrange(x: f64) -> f64 {
    let b = 0.5 * x / consts::PI;
    let a = consts::PI_2 * (b - b.floor());
    return if a.is_negative() {a + consts::PI_2} else {a}
}

// Commonality between original f0 and f1 function
fn calculate_angle(lat: f64, declin: f64, fraction: f64) -> f64 {
    // Correction: different sign as S HS
    let df = if lat.is_negative() {-fraction} else {fraction};
    let f = (declin + df).tan() * lat.to_radians().tan();
    f.min(1.0).asin() + consts::FRAC_PI_2
}

/// Calculating the hourangle
fn f0(lat: f64, declin: f64) -> f64 {
    let df0 = (0.5 * SUNRADIUS + AIRREFR).to_radians();
    calculate_angle(lat, declin, df0)
}

/// Calculating the hourangle for twilight times
fn f1(lat: f64, declin: f64) -> f64 {
    let df1 = 6.0.to_radians();
    calculate_angle(lat, declin, df1)
}

/// Find the ecliptic longitude of the sun
fn fnsun(d: f64) -> (f64, f64) {
    // mean longitude of the sun
    let mean_longitude = fnrange(280.461.to_radians() + 0.9856474.to_radians() * d);

    // mean anomaly of the sun
    let g = fnrange(357.528.to_radians() + 0.9856003.to_radians() * d);

    // Ecliptic longitude of the sun
    let ecliptic_longitude = fnrange(mean_longitude + 1.915.to_radians() * g.sin() +
                                     0.02.to_radians() * (2.0*g).sin());

    (ecliptic_longitude, mean_longitude)
}

/// Returns the number of days (including fraction) since midnight 2000-01-01
fn days_since_2000(date: Tm) -> f64 {
    let duration = date - Y2000;

    duration.num_seconds() as f64 / (24.0 * 3600.0)
}

/// Converts daylight hours to Timespec
fn daylight_hours_to_timespec(midnight: Timespec, hours: f64) -> Timespec {
    Timespec {
        sec: midnight.sec + (hours * 3600.0) as i64,
        nsec: 0
    }
}

/// Calculate civil twilight (am/pm) and sunrise and sunset at given date
pub fn calculate_daylight(date: Tm, latitude: f64, longitude: f64) -> Daylight {
    let utc = date.to_utc();
    let d2000 = days_since_2000(utc);

    // find the ecliptic longitude of the sun
    let (ecliptic_longitude, mean_longitude) = fnsun(d2000);

    // Obliquity of the ecliptic
    let obliq = 23.439.to_radians() - 0.0000004.to_radians() * d2000;

    // Find the RA and DEC of the sun
    let alpha = (obliq.cos() * ecliptic_longitude.sin()).atan2(ecliptic_longitude.cos());
    let delta = (obliq.sin() * ecliptic_longitude.sin()).asin();

    // Find the equation of time
    // in minutes
    // Correction suggested by David Smith
    let mean_longitude_corr = mean_longitude - alpha;
    let mean_longitude_corr2 = if mean_longitude_corr < consts::PI {
        mean_longitude_corr + consts::PI_2} else {
            mean_longitude_corr};
    let equation = 1440.0 * (1.0 - mean_longitude_corr2 / consts::PI/2.0);
    let ha = f0(latitude, delta);
    let hb = f1(latitude, delta);
    let twx_radians = hb - ha; // length of twilight in radions
    let twx = 12.0 * twx_radians / consts::PI; // lenth of twilight in hours

    // artic winter
    let riset = 12.0 - 12.0 * ha / consts::PI - longitude / 15.0 + equation / 60.0;
    let settm = 12.0 + 12.0 * ha / consts::PI - longitude / 15.0 + equation / 60.0;

    let twam = riset - twx;
    let twpm = settm + twx;

    // get midnight reference
    let utcmidnight = Tm {tm_mday: utc.tm_mday, tm_mon: utc.tm_mon, tm_year: utc.tm_year,
                          tm_wday: utc.tm_wday, tm_yday: utc.tm_yday, tm_utcoff: utc.tm_utcoff,
                          tm_isdst: utc.tm_isdst, tm_nsec: 0, tm_sec: 0, tm_min: 0, tm_hour: 0};
    let tsmidnight = utcmidnight.to_timespec();

    Daylight {twilight_morning: daylight_hours_to_timespec(tsmidnight, twam),
              sunrise: daylight_hours_to_timespec(tsmidnight, riset),
              sunset: daylight_hours_to_timespec(tsmidnight, settm),
              twilight_evening: daylight_hours_to_timespec(tsmidnight, twpm)}
}

#[test]
fn days_since_20150327_1200_utc() {
    let tm20150327_1200 = Tm {tm_sec: 0, tm_min: 0, tm_hour: 12, tm_mday: 27, tm_mon: 2, tm_year: 115,
        tm_wday: 0, tm_yday: 0, tm_isdst: 0, tm_utcoff:0, tm_nsec: 0};

    assert_eq!(days_since_2000(tm20150327_1200), 5564.5);
}

#[test]
fn daylight_apeldoorn_20150327_1200_utc() {
    let tm20150327_1200 = Tm {tm_sec: 0, tm_min: 0, tm_hour: 12, tm_mday: 27, tm_mon: 2, tm_year: 115,
        tm_wday: 0, tm_yday: 0, tm_isdst: 0, tm_utcoff:0, tm_nsec: 0};
    let lat_apeldoorn = 52.0 + 13.0/60.0;
    let long_apeldoorn = 5.0 + 58.0/60.0;

    let daylight = calculate_daylight(tm20150327_1200, lat_apeldoorn, long_apeldoorn);

    assert_eq!(daylight.twilight_morning.sec, 1427432129); // 2015-03-27T05:55:29+01:00
    assert_eq!(daylight.sunrise.sec, 1427433766); // 2015-03-27T06:22:46+01:00
    assert_eq!(daylight.sunset.sec, 1427479207); // 2015-03-27T19:00:07+01:00
    assert_eq!(daylight.twilight_evening.sec, 1427480844); // 2015-03-27T19:27:24+01:00
}
