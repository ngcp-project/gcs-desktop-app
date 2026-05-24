use crate::missions::types::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static! {
    //  [ [] [] [] ] [ [1,2] [2,1] [1,4] [1,5] ] [ [1,2] [] [] ]
    //Use the mission id and store a vector of vector of coordinates  [ [] [][] []]
    pub static ref KEEP_OUT_ZONES: RwLock<HashMap<i32, Vec<Vec<GeoCoordinateStruct>>>> =
        RwLock::new(HashMap::new());
}

fn harversine_distance(a: &GeoCoordinateStruct, b: &GeoCoordinateStruct) -> f64 {
    let r = 6371000.0;
    // Convert the difference in radians
    // degrees -> radians sample: 15 * pi/180-> pi/12
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = (b.long - a.long).to_radians();

    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();

    // Intermediate value "a" ->  harvesine of the central angle
    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    // Central angle
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    // return distance
    r * c
}

pub fn is_near_keep_out_zone(
    mission_id: i32,
    point: &GeoCoordinateStruct,
    threshold_m: f64,
) -> bool {
    let zones = KEEP_OUT_ZONES.read().unwrap();
    println!("Checking zones for all vehicles");
    println!("Current position: ({}, {})", point.lat, point.long);

    if let Some(polygons) = zones.get(&mission_id) {
        // provide owner reference, borrowed
        for polygon in polygons.iter() {
            for coord in polygon.iter() {
                let dist = harversine_distance(point, coord);
                if dist <= threshold_m {
                    return true;
                }
            }
        }
    } else {
        println!("There are no zones to grab");
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harversine_same_point() {
        let coord = GeoCoordinateStruct {
            lat: 33.9326,
            long: -117.6306,
        };

        let distance = harversine_distance(&coord, &coord);

        assert_eq!(distance, 0.0, "Distance between same point should be 0");
    }

    #[test]
    fn test_harversine_known_distance_la_to_sf() {
        let la = GeoCoordinateStruct {
            lat: 34.0522,
            long: -118.2437,
        };

        let sf = GeoCoordinateStruct {
            lat: 37.7749,
            long: -122.4194,
        };

        let distance = harversine_distance(&la, &sf);

        let expected = 559_000.0;
        let tolerance = 5_000.0;

        assert!(
            (distance - expected).abs() < tolerance,
            "Distance should be approximately {} meters, got {} meters",
            expected,
            distance
        );
    }

    #[test]
    fn test_harversine_order_independence() {
        let point_a = GeoCoordinateStruct {
            lat: 33.9326,
            long: -117.6306,
        };

        let point_b = GeoCoordinateStruct {
            lat: 34.0522,
            long: -118.2437,
        };

        let distance_ab = harversine_distance(&point_a, &point_b);
        let distance_ba = harversine_distance(&point_b, &point_a);

        assert_eq!(
            distance_ab, distance_ba,
            "Distance should be the same regardless of order"
        );
    }
}
