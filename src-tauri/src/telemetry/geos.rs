use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize)]
pub struct PolygonDTO {
    pub vehicle_id: String,
    pub polygon: Vec<(f64, f64)>,
}

lazy_static! {
    pub static ref KEEP_OUT_ZONES: RwLock<HashMap<String, Vec<Vec<Coordinate>>>> =
        RwLock::new(HashMap::new());
}

#[tauri::command]
pub fn update_keep_out_zone(data: Vec<PolygonDTO>) {
    let mut zones = KEEP_OUT_ZONES.write().unwrap();
    zones.clear();
    println!("📥 Received {} polygons to update", data.len());

    for dto in data {
        let key = dto.vehicle_id.to_lowercase();
        println!("🔧 Vehicle: {}, Points: {}", key, dto.polygon.len());

        if dto.polygon.len() >= 3 {
            let polygon = dto
                .polygon
                .iter()
                .map(|(lat, lon)| Coordinate {
                    latitude: *lat,
                    longitude: *lon,
                })
                .collect::<Vec<_>>();

            println!(
                "✅ Storing polygon for {}: {:?}",
                key,
                polygon
                    .iter()
                    .map(|c| (c.latitude, c.longitude))
                    .collect::<Vec<_>>()
            );

            zones.entry(key).or_default().push(polygon);
        } else {
            println!("⚠️ Skipped polygon for {}: too few points", key);
        }
    }
}

fn harversine_distance(a: &Coordinate, b: &Coordinate) -> f64 {
    let r = 6371000.0;
    let dlat = (b.latitude - a.latitude).to_radians();
    let dlon = (b.longitude - a.longitude).to_radians();

    let lat1 = a.latitude.to_radians();
    let lat2 = b.latitude.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

pub fn is_near_keep_out_zone(vehicle_id: &str, point: &Coordinate, threshold_m: f64) -> bool {
    let zones = KEEP_OUT_ZONES.read().unwrap();
    println!("🔍 Checking zones for vehicle: {}", vehicle_id);
    println!(
        "📍 Current position: ({}, {})",
        point.latitude, point.longitude
    );
    if let Some(polygons) = zones.get(&vehicle_id.to_lowercase()) {
        for polygon in polygons {
            for coord in polygon {
                let dist = harversine_distance(point, coord);
                if dist <= threshold_m {
                    return true;
                }
            }
        }
    } else {
        println!("⚠️ No zones registered for vehicle {}", vehicle_id);
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harversine_same_point() {
        // TEST PURPOSE: Make sure calculating distance from a point to itself returns 0
        // This is the simplest possible test - if this fails, something is very wrong!

        // This is a location in Orange County, CA (could be any coordinate)
        // Latitude: 33.9326° N (north of equator)
        // Longitude: -117.6306° W (west of prime meridian)
        let coord = Coordinate {
            latitude: 33.9326,
            longitude: -117.6306,
        };

        // Calculate distance from this point to itself
        let distance = harversine_distance(&coord, &coord);

        // EXPECTED RESULT: 0.0 meters (no distance between same location)
        assert_eq!(distance, 0.0, "Distance between same point should be 0");
    }

    #[test]
    fn test_harversine_known_distance_la_to_sf() {
        // TEST PURPOSE: Validate that the math is correct by checking a known real-world distance
        // We know LA to San Francisco is about 559 km, so the function should return close to that

        // Los Angeles downtown coordinates
        // Latitude: 34.0522° N
        // Longitude: -118.2437° W
        let la = Coordinate {
            latitude: 34.0522,
            longitude: -118.2437,
        };

        // San Francisco downtown coordinates
        // Latitude: 37.7749° N
        // Longitude: -122.4194° W
        let sf = Coordinate {
            latitude: 37.7749,
            longitude: -122.4194,
        };

        // Calculate the straight-line distance between LA and SF
        let distance = harversine_distance(&la, &sf);

        // EXPECTED RESULT: Approximately 559,000 meters (559 km)
        // We allow 5km tolerance because:
        // - This is straight-line distance (not driving distance)
        // - Small rounding differences are acceptable
        let expected = 559_000.0; // meters (convert km to meters: 559 * 1000)
        let tolerance = 5_000.0;   // allow +/- 5km error

        assert!(
            (distance - expected).abs() < tolerance,
            "Distance should be approximately {} meters, got {} meters",
            expected,
            distance
        );
    }

    #[test]
    fn test_harversine_order_independence() {
        // TEST PURPOSE: Ensure that distance(A→B) equals distance(B→A)
        // This is a fundamental property of distance - it shouldn't matter which point is first

        // Point A: Location in Orange County, CA
        let point_a = Coordinate {
            latitude: 33.9326,    // Latitude (north/south position)
            longitude: -117.6306, // Longitude (east/west position)
        };

        // Point B: Downtown Los Angeles, CA
        let point_b = Coordinate {
            latitude: 34.0522,    // Latitude
            longitude: -118.2437, // Longitude
        };

        // Calculate distance from A to B
        let distance_ab = harversine_distance(&point_a, &point_b);

        // Calculate distance from B to A
        let distance_ba = harversine_distance(&point_b, &point_a);

        // EXPECTED RESULT: Both distances should be exactly equal
        // The distance from LA to Orange County = distance from Orange County to LA
        assert_eq!(
            distance_ab, distance_ba,
            "Distance should be the same regardless of order"
        );
    }
}
