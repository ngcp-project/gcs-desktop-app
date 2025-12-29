use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;
use crate::missions::types::*;

lazy_static! {
    //  [ [] [] [] ] [ [1,2] [2,1] [1,4] [1,5] ] [ [1,2] [] [] ]
    //Use the mission id and store a vector of vector of coordinates  [ [] [][] []]
    pub static ref KEEP_OUT_ZONES: RwLock<HashMap<i32, Vec<Vec<GeoCoordinateStruct>>>> =
        RwLock::new(HashMap::new());
}

fn harversine_distance(a: &GeoCoordinateStruct, b: &GeoCoordinateStruct) -> f64 {
    let r = 6371000.0;
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = (b.long - a.long).to_radians();

    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

pub fn is_near_keep_out_zone(mission_id: i32 , point: &GeoCoordinateStruct, threshold_m: f64) -> bool {
    let zones = KEEP_OUT_ZONES.read().unwrap();
    println!("Checking zones for all vehicles");
    println!(
        "Current position: ({}, {})",
        point.lat, point.long
    );
    
        if let Some(polygons) = zones.get(&mission_id) {
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


