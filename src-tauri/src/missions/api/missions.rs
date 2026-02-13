/*
Implement helper methods on MissionApiImpl for mission-level 
operations (create, rename, delete missions, get mission data, 
update mission status, start mission flow).

*/

use tauri::{AppHandle, Runtime};
use crate::missions::types::*;
use crate::missions::sql::{update_mission_name, delete_mission, update_mission_status, update_stage_status, update_auto_mode_vehicle};
use crate::commands::commands::{CommandsApiImpl, GeoCoordinate};
use crate::commands::CommandsApi;
use super::MissionApiImpl;

impl MissionApiImpl {
    pub async fn get_mission_data_helper(&self, mission_id: i32) -> MissionStruct {
        let state = self.state.lock().await;
        state
            .missions
            .iter()
            .find(|m| m.mission_id == mission_id)
            .map(|m| m.clone())
            .unwrap_or_else(|| panic!("Mission not found"))
    }

    pub async fn rename_mission_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        mission_name: String,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        update_mission_name(self.db.clone(), mission.mission_id, &mission_name)
            .await
            .expect("Failed to update mission name");
        mission.mission_name = mission_name;
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn create_mission_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_name: String,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        // self.clone() requires self to be Clone, which it is (Arc and PgPool are Clone)
        let new_mission = self.clone().create_default_mission(&mission_name).await;
        state.missions.push(new_mission);
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn delete_mission_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
    ) -> Result<(), String> {
        println!("Deleting mission with ID: {}", mission_id);
        let mut state = self.state.lock().await;
        let mission_index = state
            .missions
            .iter()
            .position(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        if !matches!(
            state.missions[mission_index].mission_status,
            MissionStageStatusEnum::Inactive
        ) {
            return Err("Cannot delete active/past missions".into());
        }
        delete_mission(self.db.clone(), state.missions[mission_index].mission_id)
            .await
            .expect("Failed to delete mission from database");

        state.missions.remove(mission_index);
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn start_mission_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let commands_api = CommandsApiImpl::default();

        // First, handle the previous mission if it exists
        if let Some(prev_mission_index) = state.missions.iter().position(|m| m.mission_id == state.current_mission) {
            state.missions[prev_mission_index].mission_status = MissionStageStatusEnum::Complete;
            update_mission_status(self.db.clone(), state.missions[prev_mission_index].mission_id, "Complete").await.expect("Failed to update mission status");
        }

        // Find and update the new mission
        let start_mission_index = state.missions.iter().position(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;
        
        // Update mission status first
        state.missions[start_mission_index].mission_status = MissionStageStatusEnum::Active;
        state.current_mission = mission_id;
        update_mission_status(self.db.clone(), mission_id, "Active").await.expect("Failed to update mission status");

        // Emit state update to ensure frontend reflects the change
        self.emit_state_update(&app_handle, &state)?;

        // Now handle the zone updates
        let mission = &state.missions[start_mission_index];
        
        // Send keep-in zones (commandID: 2) only if there are valid zones
        for zone in &mission.zones.keep_in_zones {
            if zone.len() >= 3 {  // Only send if we have at least 3 coordinates
                let coords: Vec<GeoCoordinate> = zone.iter()
                    .take(6) // Limit to 6 points
                    .map(|coord| GeoCoordinate {
                        lat: coord.lat,
                        long: coord.long,
                    })
                    .collect();
                
                // Send to ALL vehicles at once
                commands_api.clone().send_zone_update("ALL".to_string(), "2".to_string(), coords).await?;
            }
        }

        // Send keep-out zones (commandID: 3) only if there are valid zones
        for zone in &mission.zones.keep_out_zones {
            if zone.len() >= 3 {  // Only send if we have at least 3 coordinates
                let coords: Vec<GeoCoordinate> = zone.iter()
                    .take(6) // Limit to 6 points
                    .map(|coord| GeoCoordinate {
                        lat: coord.lat,
                        long: coord.long,
                    })
                    .collect();
                
                // Send to ALL vehicles at once
                commands_api.clone().send_zone_update("ALL".to_string(), "3".to_string(), coords).await?;
            }
        }

        // Update vehicle stages and send search areas
        let vehicles = &mut state.missions[start_mission_index].vehicles;
        
        // Set the first stage of each vehicle to active if they have stages
        if !vehicles.MEA.stages.is_empty() {
            vehicles.MEA.stages[0].stage_status = MissionStageStatusEnum::Active;
            update_stage_status(
                self.db.clone(),
                vehicles.MEA.stages[0].stage_id,
                "Active",
            ).await.expect("Failed to update stage status");

            // Send search area for MEA only if it has valid coordinates
            let search_area = &vehicles.MEA.stages[0].search_area;
            if search_area.len() >= 3 {  // Only send if we have at least 3 coordinates
                let coords: Vec<GeoCoordinate> = search_area.iter()
                    .take(6)
                    .map(|coord| GeoCoordinate {
                        lat: coord.lat,
                        long: coord.long,
                    })
                    .collect();
                
                commands_api.clone().send_zone_update("MEA".to_string(), "4".to_string(), coords).await?;
            }
        }
        
        if !vehicles.ERU.stages.is_empty() {
            vehicles.ERU.stages[0].stage_status = MissionStageStatusEnum::Active;
            update_stage_status(
                self.db.clone(),
                vehicles.ERU.stages[0].stage_id,
                "Active",
            ).await.expect("Failed to update stage status");

            // Send search area for ERU only if it has valid coordinates
            let search_area = &vehicles.ERU.stages[0].search_area;
            if search_area.len() >= 3 {  // Only send if we have at least 3 coordinates
                let coords: Vec<GeoCoordinate> = search_area.iter()
                    .take(6)
                    .map(|coord| GeoCoordinate {
                        lat: coord.lat,
                        long: coord.long,
                    })
                    .collect();
                
                commands_api.clone().send_zone_update("ERU".to_string(), "4".to_string(), coords).await?;
            }
        }
        
        if !vehicles.MRA.stages.is_empty() {
            vehicles.MRA.stages[0].stage_status = MissionStageStatusEnum::Active;
            update_stage_status(
                self.db.clone(),
                vehicles.MRA.stages[0].stage_id,
                "Active",
            ).await.expect("Failed to update stage status");

            // Send search area for MRA only if it has valid coordinates
            let search_area = &vehicles.MRA.stages[0].search_area;
            if search_area.len() >= 3 {  // Only send if we have at least 3 coordinates
                let coords: Vec<GeoCoordinate> = search_area.iter()
                    .take(6)
                    .map(|coord| GeoCoordinate {
                        lat: coord.lat,
                        long: coord.long,
                    })
                    .collect();
                
                commands_api.clone().send_zone_update("MRA".to_string(), "4".to_string(), coords).await?;
            }
        }
        
        // Final state update after all changes
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn set_auto_mode_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        is_auto: bool,
    ) -> Result<(), String> {
        println!("Setting auto mode for vehicle: {:?}", vehicle_name);
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        let vehicle = match vehicle_name {
            VehicleEnum::MEA => &mut mission.vehicles.MEA,
            VehicleEnum::ERU => &mut mission.vehicles.ERU,
            VehicleEnum::MRA => return Err("MRA auto mode unsupported".into()),
        };

        update_auto_mode_vehicle(
            self.db.clone(),
            mission.mission_id,
            vehicle.vehicle_name.to_string(),
            is_auto,
        )
        .await
        .expect("Failed to update auto mode in database");

        vehicle.is_auto = Some(is_auto);
        self.emit_state_update(&app_handle, &state)
    }
}
