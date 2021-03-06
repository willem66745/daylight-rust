//! This library calculates moment of sunrise and sunset at a given date,
//! [latitude](http://en.wikipedia.org/wiki/Latitude) and
//! [longitude](http://en.wikipedia.org/wiki/Longitude). Also the civil
//! twilight at am and pm, moment of solar noon, the
//! [declination](http://en.wikipedia.org/wiki/Declination) of the sun and the
//! [solar azimuth angle](http://en.wikipedia.org/wiki/Solar_azimuth_angle) is
//! calculated.
//!
//! The original algorithm is written in C is available at
//! http://www.sci.fi/~benefon/rscalc.c (by Jarmo Lammi).
//! More recent implementation is available at Github page
//! https://github.com/jarmol/suncalcs. Parts of the code (variable and
//! function names) are kept intentionally the same as reference.

// Original text in rscalc.c:
//
// C program calculating the sunrise and sunset for
// the current date and a fixed location(latitude,longitude)
// Note, twilight calculation gives insufficient accuracy of results
// Jarmo Lammi 1999 - 2001

extern crate time;

use time::{Timespec, Tm, Duration};
use std::f64::consts;

const SUNRADIUS: f64 = 0.53;
const AIRREFR: f64 = 34.0 / 60.0;
const Y2000: Tm = Tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 1,
    tm_mon: 0,
    tm_year: 100,
    tm_wday: 6,
    tm_yday: 0,
    tm_isdst: 0,
    tm_utcoff: 0,
    tm_nsec: 0,
};
const SECS_IN_HOUR: f64 = 3600.0;
const HOURS_IN_DAY: f64 = 24.0;
const FRAC_HOURS_IN_DAY_2: f64 = 12.0;

fn to_radians(target: f64) -> f64 {
    let value: f64 = consts::PI;
    target * (value / 180.0)
}

#[inline]
fn to_degrees(target: f64) -> f64 {
    target * (180.0f64 / consts::PI)
}

/// Result of the daylight calculation (calculated times are UTC based)
#[derive(Clone, Copy, Debug)]
pub struct Daylight {
    pub twilight_morning: Timespec,
    pub sunrise: Timespec,
    pub sunset: Timespec,
    pub twilight_evening: Timespec,
    pub noon: Timespec,
    /// Declination of the sun in angle degrees
    pub declination: f64,
    /// Duration of the day (calculated in seconds)
    pub daylength: Duration,
    /// Sun altitude in angle degrees
    pub sun_altitude: f64,
}

/// the function below returns an angle in the range 0 to 2*pi
fn fnrange(x: f64) -> f64 {
    let b = 0.5 * x / consts::PI;
    let a = consts::PI * 2.0 * (b - b.floor());
    if a.is_sign_negative() {
        a + consts::PI * 2.0
    } else {
        a
    }
}

// Commonality between original f0 and f1 function
fn calculate_angle(lat: f64, declin: f64, fraction: f64) -> f64 {
    // Correction: different sign as S HS
    let df = if lat.is_sign_negative() {
        -fraction
    } else {
        fraction
    };
    let f = (declin + df).tan() * lat.tan();
    f.min(1.0).max(-1.0).asin() + consts::FRAC_PI_2
}

/// Calculating the hourangle
fn f0(lat: f64, declin: f64) -> f64 {
    let df0 = to_radians(0.5 * SUNRADIUS + AIRREFR);
    calculate_angle(lat, declin, df0)
}

/// Calculating the hourangle for twilight times
fn f1(lat: f64, declin: f64) -> f64 {
    let df1 = to_radians(6.0);
    calculate_angle(lat, declin, df1)
}

/// Find the ecliptic longitude of the sun
fn fnsun(d: f64) -> (f64, f64) {
    // mean longitude of the sun
    let mean_longitude = fnrange(to_radians(280.461) + to_radians(0.9856474) * d);

    // mean anomaly of the sun
    let g = fnrange(to_radians(357.528) + to_radians(0.9856003) * d);

    // Ecliptic longitude of the sun
    let ecliptic_longitude = fnrange(mean_longitude + to_radians(1.915) * g.sin() +
                                     to_radians(0.02) * (2.0 * g).sin());

    (ecliptic_longitude, mean_longitude)
}

/// Returns the number of days (including fraction) since midnight 2000-01-01
fn days_since_2000(date: Tm) -> f64 {
    let duration = date - Y2000;

    duration.num_seconds() as f64 / (HOURS_IN_DAY * SECS_IN_HOUR)
}

/// Converts daylight hours to Timespec
fn daylight_hours_to_timespec(midnight: Timespec, hours: f64) -> Timespec {
    Timespec {
        sec: midnight.sec + (hours * SECS_IN_HOUR) as i64,
        nsec: 0,
    }
}

/// Calculate civil twilight (am/pm) and sunrise and sunset at given date
pub fn calculate_daylight(date: Tm, latitude: f64, longitude: f64) -> Daylight {
    let lat_rad = to_radians(latitude);
    let utc = date.to_utc();
    let d2000 = days_since_2000(utc);

    // find the ecliptic longitude of the sun
    let (ecliptic_longitude, mean_longitude) = fnsun(d2000);

    // Obliquity of the ecliptic
    let obliq = to_radians(23.439) - to_radians(0.0000004) * d2000;

    // Find the RA and DEC of the sun
    let alpha = (obliq.cos() * ecliptic_longitude.sin()).atan2(ecliptic_longitude.cos());
    let delta = (obliq.sin() * ecliptic_longitude.sin()).asin();

    // Find the equation of time
    // in minutes
    // Correction suggested by David Smith
    let mean_longitude_corr = mean_longitude - alpha;
    let mean_longitude_corr2 = if mean_longitude_corr < consts::PI {
        mean_longitude_corr + consts::PI * 2.0
    } else {
        mean_longitude_corr
    };
    let equation = HOURS_IN_DAY * (1.0 - mean_longitude_corr2 / (consts::PI * 2.0));
    let ha = f0(lat_rad, delta);
    let hb = f1(lat_rad, delta);
    let twx_radians = hb - ha; // length of twilight in radions
    let twx = FRAC_HOURS_IN_DAY_2 * twx_radians / consts::PI; // lenth of twilight in hours

    // artic winter
    let halfday = FRAC_HOURS_IN_DAY_2 * ha / consts::PI;
    let riset = FRAC_HOURS_IN_DAY_2 - halfday - longitude / 15.0 + equation;
    let settm = FRAC_HOURS_IN_DAY_2 + halfday - longitude / 15.0 + equation;
    let noon = riset + halfday;

    let twam = riset - twx;
    let twpm = settm + twx;

    let altmax_nh = consts::FRAC_PI_2 + delta - lat_rad;
    let altmax = if lat_rad < delta {
        consts::PI - altmax_nh
    } else {
        altmax_nh
    };

    // get midnight reference
    let utcmidnight = Tm {
        tm_mday: utc.tm_mday,
        tm_mon: utc.tm_mon,
        tm_year: utc.tm_year,
        tm_wday: utc.tm_wday,
        tm_yday: utc.tm_yday,
        tm_utcoff: utc.tm_utcoff,
        tm_isdst: utc.tm_isdst,
        tm_nsec: 0,
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
    };
    let tsmidnight = utcmidnight.to_timespec();

    Daylight {
        twilight_morning: daylight_hours_to_timespec(tsmidnight, twam),
        sunrise: daylight_hours_to_timespec(tsmidnight, riset),
        sunset: daylight_hours_to_timespec(tsmidnight, settm),
        twilight_evening: daylight_hours_to_timespec(tsmidnight, twpm),
        noon: daylight_hours_to_timespec(tsmidnight, noon),
        declination: to_degrees(delta),
        daylength: Duration::seconds((halfday * SECS_IN_HOUR * 2.0) as i64),
        sun_altitude: to_degrees(altmax),
    }
}

#[test]
fn days_since_20150327_1200_utc() {
    let tm20150327_1200 = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 12,
        tm_mday: 27,
        tm_mon: 2,
        tm_year: 115,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_utcoff: 0,
        tm_nsec: 0,
    };

    assert_eq!(days_since_2000(tm20150327_1200), 5564.5);
}

#[test]
fn daylight_apeldoorn_20150327_1200_utc() {
    let tm20150327_1200 = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 12,
        tm_mday: 27,
        tm_mon: 2,
        tm_year: 115,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_utcoff: 0,
        tm_nsec: 0,
    };
    let lat_apeldoorn = 52.0 + 13.0 / 60.0;
    let long_apeldoorn = 5.0 + 58.0 / 60.0;

    let daylight = calculate_daylight(tm20150327_1200, lat_apeldoorn, long_apeldoorn);

    assert_eq!(daylight.twilight_morning.sec, 1427432129); // 2015-03-27T05:55:29+01:00
    assert_eq!(daylight.sunrise.sec, 1427433766); // 2015-03-27T06:22:46+01:00
    assert_eq!(daylight.noon.sec, 1427456487);
    assert_eq!(daylight.sunset.sec, 1427479207); // 2015-03-27T19:00:07+01:00
    assert_eq!(daylight.twilight_evening.sec, 1427480844); // 2015-03-27T19:27:24+01:00
    assert_eq!(daylight.daylength.num_seconds(), 45440);
    assert!(daylight.declination > 2.777311 && daylight.declination < 2.777313,
            "declination != {}",
            daylight.declination);
    assert!(daylight.sun_altitude > 40.55 && daylight.sun_altitude < 40.57,
            "sun_altitude != {}",
            daylight.sun_altitude);
}

#[test]
fn daylight_tokyo_20150327_1200_utc() {
    let tm20150327_1200 = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 12,
        tm_mday: 27,
        tm_mon: 2,
        tm_year: 115,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_utcoff: 0,
        tm_nsec: 0,
    };
    let lat_tokyo = 35.41;
    let long_tokyo = 139.41;

    let daylight = calculate_daylight(tm20150327_1200, lat_tokyo, long_tokyo);

    assert_eq!(daylight.twilight_morning.sec, 1427401349);
    assert_eq!(daylight.sunrise.sec, 1427402244);
    assert_eq!(daylight.noon.sec, 1427424460);
    assert_eq!(daylight.sunset.sec, 1427446677);
    assert_eq!(daylight.twilight_evening.sec, 1427447573);
    assert_eq!(daylight.daylength.num_seconds(), 44433);
    assert!(daylight.declination > 2.777311 && daylight.declination < 2.777313,
            "declination != {}",
            daylight.declination);
    assert!(daylight.sun_altitude > 57.35 && daylight.sun_altitude < 57.37,
            "sun_altitude != {}",
            daylight.sun_altitude);
}

#[test]
fn daylight_avarua_20150327_1200_utc() {
    let tm20150327_1200 = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 12,
        tm_mday: 27,
        tm_mon: 2,
        tm_year: 115,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_utcoff: 0,
        tm_nsec: 0,
    };
    let lat_tokyo = -21.12;
    let long_tokyo = -159.46;

    let daylight = calculate_daylight(tm20150327_1200, lat_tokyo, long_tokyo);

    assert_eq!(daylight.twilight_morning.sec, 1427474290);
    assert_eq!(daylight.sunrise.sec, 1427474769);
    assert_eq!(daylight.noon.sec, 1427496189);
    assert_eq!(daylight.sunset.sec, 1427517608);
    assert_eq!(daylight.twilight_evening.sec, 1427518088);
    assert_eq!(daylight.daylength.num_seconds(), 42839);
    assert!(daylight.declination > 2.777311 && daylight.declination < 2.777313,
            "declination != {}",
            daylight.declination);
    assert!(daylight.sun_altitude > 66.09 && daylight.sun_altitude < 66.11,
            "sun_altitude != {}",
            daylight.sun_altitude);
}

#[test]
fn daylight_longyearbyen_20150621_1200_utc_midsummer() {
    let tm20150621_1200 = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 12,
        tm_mday: 21,
        tm_mon: 5,
        tm_year: 115,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_utcoff: 0,
        tm_nsec: 0,
    };
    let lat_tokyo = 78.22;
    let long_tokyo = 15.65;

    let daylight = calculate_daylight(tm20150621_1200, lat_tokyo, long_tokyo);

    assert_eq!((daylight.sunset - daylight.sunrise).num_minutes(),
               23 * 60 + 59); // midsummer
    assert_eq!(daylight.twilight_morning.sec, 1434841155);
    assert_eq!(daylight.sunrise.sec, 1434841155);
    assert_eq!(daylight.noon.sec, 1434884354);
    assert_eq!(daylight.sunset.sec, 1434927554);
    assert_eq!(daylight.twilight_evening.sec, 1434927554);
    assert_eq!(daylight.daylength.num_seconds(), 86400);
    assert!(daylight.declination > 23.436411 && daylight.declination < 23.436413,
            "declination != {}",
            daylight.declination);
    assert!(daylight.sun_altitude > 35.20 && daylight.sun_altitude < 35.22,
            "sun_altitude != {}",
            daylight.sun_altitude);
}

#[test]
fn daylight_longyearbyen_20151221_1200_utc_midwinter() {
    let tm20151221_1200 = Tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 12,
        tm_mday: 21,
        tm_mon: 11,
        tm_year: 115,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_utcoff: 0,
        tm_nsec: 0,
    };
    let lat_tokyo = 78.22;
    let long_tokyo = 15.65;

    let daylight = calculate_daylight(tm20151221_1200, lat_tokyo, long_tokyo);

    assert_eq!((daylight.sunset - daylight.sunrise).num_minutes(), 0); // midwinter
    assert_eq!(daylight.twilight_morning.sec, 1450695334);
    assert_eq!(daylight.sunrise.sec, 1450695334);
    assert_eq!(daylight.noon.sec, 1450695334);
    assert_eq!(daylight.sunset.sec, 1450695334);
    assert_eq!(daylight.twilight_evening.sec, 1450695334);
    assert_eq!(daylight.daylength.num_seconds(), 0);
    assert!(daylight.declination > -23.43652 && daylight.declination < -23.43650,
            "declination != {}",
            daylight.declination);
    assert!(daylight.sun_altitude > -11.66 && daylight.sun_altitude < -11.64,
            "sun_altitude != {}",
            daylight.sun_altitude);
}

#[test]
fn range_check() {
    for long in (-180..180).filter(|x| x % 8 == 0) {
        for lat in (-90..91).filter(|x| x % 8 == 0) {
            for year in (70..138).filter(|x| x % 2 == 0) {
                // 138 -> last supported year at 32-bit systems
                for month in 0..12 {
                    let tm = Tm {
                        tm_sec: 0,
                        tm_min: 0,
                        tm_hour: 0,
                        tm_mday: 15,
                        tm_mon: month,
                        tm_year: year,
                        tm_wday: 0,
                        tm_yday: 0,
                        tm_isdst: 0,
                        tm_utcoff: 0,
                        tm_nsec: 0, /* 2 january; because 1 january may still result in 1969 timestamps */
                    };

                    let daylight = calculate_daylight(tm, lat as f64, long as f64);

                    assert!(daylight.twilight_morning.sec > 0,
                            "daylight={:?} lat={} long={} year={}",
                            daylight,
                            lat,
                            long,
                            year);
                    assert!(daylight.twilight_morning <= daylight.sunrise);
                    assert!(daylight.sunrise <= daylight.noon);
                    assert!(daylight.noon <= daylight.sunset);
                    assert!(daylight.sunset <= daylight.twilight_evening);
                    assert!(daylight.daylength.num_seconds() >= 0);
                }
            }
        }
    }
}
