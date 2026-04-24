/*
Implement helper methods on MissionApiImpl for zone operations 
(add, update, delete zones, apply zone validation rules, 
convert between DB zone format and coordinate types).
*/

use tauri::{AppHandle, Runtime};
use crate::missions::types::{GeofenceType, ZoneType};
use crate::missions::sql::update_zones;
use serde_json::Value;
use crate::telemetry::geos::KEEP_OUT_ZONES;

// We need to import the struct to implement methods on it.
use super::MissionApiImpl;

impl MissionApiImpl {
    pub async fn add_zone_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
    ) -> Result<(), String> {
        println!("Adding zone of type: {:?}", zone_type);
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        match zone_type {
            ZoneType::KeepIn => mission.zones.keep_in_zones.push(GeofenceType::default()),
            ZoneType::KeepOut => mission.zones.keep_out_zones.push(GeofenceType::default()),
        }

        // note: no need for SQL here since its just an empty zone be changed in the rust state
        
        // We need to emit update. The emit_state_update is defined in events2.rs (or locally if we didn't split perfectly).
        // Since we are splitting, `emit_state_update` is a method on MissionApiImpl.
        // It can be called normally.
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn update_zone_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
        zone_index: i32,
        zone_coords: GeofenceType,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        match zone_type {
            ZoneType::KeepIn => {
                // if zone_index >= mission.zones.keep_in_zones.len() as u32 {
                //     return Err("KeepIn index out of range".into());
                // }
                if let Some(zone) = mission.zones.keep_in_zones.get_mut(zone_index as usize) {
                    *zone = zone_coords;
                }
            }
            ZoneType::KeepOut => {
                // if zone_index >= mission.zones.keep_out_zones.len() as u32 {
                //     return Err("KeepOut index out of range".into());
                // }
                if let Some(zone) = mission.zones.keep_out_zones.get_mut(zone_index as usize) {
                    *zone = zone_coords.clone();
                }
                let mut keep_out_zones = KEEP_OUT_ZONES.write().expect("Failed to acquire write lock");
                let key = mission.mission_id;
                keep_out_zones.entry(key).or_default().push(zone_coords);
                println!("Updated structure to showcase {:?}", keep_out_zones);
            }
        }

        let keep_in_zones = mission.zones.keep_in_zones.iter()
            .map(|zone| {
                let json = serde_json::to_string(zone).unwrap();
                convert_zone_format(&json)
            })
            .collect::<Vec<String>>();

        let keep_out_zones = mission.zones.keep_out_zones.iter()
            .map(|zone| {
                let json = serde_json::to_string(zone).unwrap();
                convert_zone_format(&json)
            })
            .collect::<Vec<String>>();


        // update zones
        update_zones(
            self.db.clone(),
            mission.mission_id,
            keep_in_zones.clone(),
            keep_out_zones.clone(),
        ).await.expect("Failed to add zones");

        self.emit_state_update(&app_handle, &state)
    }

    pub async fn delete_zone_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
        zone_index: i32,
    ) -> Result<(), String> {
        println!(
            "Deleting zone of type: {:?} at index: {}",
            zone_type, zone_index
        );
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        match zone_type {
            ZoneType::KeepIn => {
                if zone_index >= mission.zones.keep_in_zones.len() as i32 {
                    return Err("KeepIn index out of range".into());
                }
                mission.zones.keep_in_zones.remove(zone_index as usize);
            }
            ZoneType::KeepOut => {
                if zone_index >= mission.zones.keep_out_zones.len() as i32 {
                    return Err("KeepOut index out of range".into());
                }
                mission.zones.keep_out_zones.remove(zone_index as usize);

                let mut keep_out_zones = KEEP_OUT_ZONES.write().expect("Failed to acquire locks");
                let key = mission.mission_id;
                keep_out_zones.entry(key).or_default().remove(zone_index as usize);
                println!("Updated structure to showcase {:?}", keep_out_zones);

            }
        }

        let keep_in_zones = mission.zones.keep_in_zones.iter()
            .map(|zone| {
                let json = serde_json::to_string(zone).unwrap();
                convert_zone_format(&json)
            })
            .collect::<Vec<String>>();

        let keep_out_zones = mission.zones.keep_out_zones.iter()
            .map(|zone| {
                let json = serde_json::to_string(zone).unwrap();
                convert_zone_format(&json)
            })
            .collect::<Vec<String>>();


        // update zones
        update_zones(
            self.db.clone(),
            mission.mission_id,
            keep_in_zones.clone(),
            keep_out_zones.clone(),
        ).await.expect("Failed to delete zones");

        self.emit_state_update(&app_handle, &state)
    }
}

// helper function for converting JSON string to zone format
pub fn convert_zone_format(json_str: &str) -> String {
    let parsed: Value = serde_json::from_str(json_str).unwrap();

    if let Some(arr) = parsed.as_array() {
        let tuples: Vec<String> = arr.iter().map(|point| {
            let lat = point["lat"].as_f64().unwrap();
            let long = point["long"].as_f64().unwrap();
            format!("({:.5},{:.5})", lat, long)
        }).collect();

        format!("[\n    {}\n]", tuples.join(",\n    "))
    } else {
        String::new()
    }
}

pub fn convert_zone_to_json(zone_str: &str) -> String {
    // Remove brackets and whitespace
    let content = zone_str
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();

    // Parse each coordinate pair
    let coords: Vec<String> = content
        .split(',')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .chunks(2)
        .map(|chunk| {
            let lat = chunk[0]
                .trim()
                .trim_start_matches('(')
                .trim_end_matches(')')
                .trim()  // Trim again after removing parentheses
                .parse::<f64>()
                .unwrap_or(0.0);
            let long = chunk[1]
                .trim()
                .trim_start_matches('(')
                .trim_end_matches(')')
                .trim()  // Trim again after removing parentheses
                .parse::<f64>()
                .unwrap_or(0.0);
            format!(r#"{{"lat":{:.5},"long":{:.5}}}"#, lat, long)
        })
        .collect();

    format!("[{}]", coords.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_zone_format_single_point() {
        // TEST PURPOSE: Verify that a single coordinate point is correctly formatted
        // Input: JSON array with one coordinate object
        // Expected: PostgreSQL-style coordinate format

        let json_input = r#"[{"lat":33.93257,"long":-117.63059}]"#;
        let result = convert_zone_format(json_input);

        // EXPECTED RESULT: Should format as [(lat,long)]
        let expected = "[\n    (33.93257,-117.63059)\n]";
        assert_eq!(result, expected, "Single point should be formatted correctly");
    }

    #[test]
    fn test_convert_zone_format_multiple_points() {
        // TEST PURPOSE: Verify that multiple coordinate points are correctly formatted
        // This is the typical use case for geofence zones (polygons)

        let json_input = r#"[{"lat":33.93257,"long":-117.63059},{"lat":34.05220,"long":-118.24370},{"lat":37.77490,"long":-122.41940}]"#;
        let result = convert_zone_format(json_input);

        // EXPECTED RESULT: Should format as multi-line coordinate list
        let expected = "[\n    (33.93257,-117.63059),\n    (34.05220,-118.24370),\n    (37.77490,-122.41940)\n]";
        assert_eq!(result, expected, "Multiple points should be formatted with proper line breaks");
    }

    #[test]
    fn test_convert_zone_format_precision() {
        // TEST PURPOSE: Verify that coordinate precision is preserved (5 decimal places)
        // GPS coordinates need high precision for accuracy

        let json_input = r#"[{"lat":33.123456789,"long":-117.987654321}]"#;
        let result = convert_zone_format(json_input);

        // EXPECTED RESULT: Should round to 5 decimal places
        let expected = "[\n    (33.12346,-117.98765)\n]";
        assert_eq!(result, expected, "Should preserve 5 decimal places precision");
    }

    #[test]
    fn test_convert_zone_to_json_single_point() {
        // TEST PURPOSE: Verify that a single coordinate in zone format converts to JSON
        // This is the inverse operation of convert_zone_format

        let zone_input = "[(33.93257,-117.63059)]";
        let result = convert_zone_to_json(zone_input);

        // EXPECTED RESULT: Should produce valid JSON array with one coordinate object
        let expected = r#"[{"lat":33.93257,"long":-117.63059}]"#;
        assert_eq!(result, expected, "Single point should convert to JSON correctly");
    }

    #[test]
    fn test_convert_zone_to_json_multiple_points() {
        // TEST PURPOSE: Verify that multiple coordinates in zone format convert to JSON
        // This tests the chunking logic that pairs lat/long values

        let zone_input = "[(33.93257,-117.63059),(34.05220,-118.24370),(37.77490,-122.41940)]";
        let result = convert_zone_to_json(zone_input);

        // EXPECTED RESULT: Should produce valid JSON array with three coordinate objects
        let expected = r#"[{"lat":33.93257,"long":-117.63059},{"lat":34.05220,"long":-118.24370},{"lat":37.77490,"long":-122.41940}]"#;
        assert_eq!(result, expected, "Multiple points should convert to JSON correctly");
    }

    #[test]
    fn test_convert_zone_to_json_with_whitespace() {
        // TEST PURPOSE: Verify that extra whitespace is handled correctly
        // Zone strings from database might have varying whitespace

        let zone_input = "[  ( 33.93257 , -117.63059 )  ]";
        let result = convert_zone_to_json(zone_input);

        // EXPECTED RESULT: Should ignore whitespace and parse correctly
        let expected = r#"[{"lat":33.93257,"long":-117.63059}]"#;
        assert_eq!(result, expected, "Should handle whitespace correctly");
    }

    #[test]
    fn test_convert_zone_roundtrip() {
        // TEST PURPOSE: Verify that converting JSON → Zone → JSON produces equivalent result
        // This ensures the two functions are true inverses of each other

        let original_json = r#"[{"lat":33.93257,"long":-117.63059},{"lat":34.05220,"long":-118.24370}]"#;

        // Convert JSON to zone format
        let zone_format = convert_zone_format(original_json);

        // Convert back to JSON
        let result_json = convert_zone_to_json(&zone_format);

        // EXPECTED RESULT: Should match original JSON
        assert_eq!(result_json, original_json, "Roundtrip conversion should preserve data");
    }
}
