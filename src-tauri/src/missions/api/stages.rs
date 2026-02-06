/*
Implement helper methods on MissionApiImpl for stage-level operations
(add, delete, rename stages, transition stages, update search area).
*/

use tauri::{AppHandle, Runtime};
use crate::missions::types::*;
use crate::missions::sql::{select_vehicle_from_mission, update_stage_area, delete_stage, update_stage_name, transition_stage};
use crate::commands::commands::{CommandsApiImpl, GeoCoordinate};
use crate::commands::CommandsApi;
use super::MissionApiImpl;

impl MissionApiImpl {
    pub async fn add_stage_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_name: String,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        let vehicle = match vehicle_name {
            VehicleEnum::MEA => &mut mission.vehicles.MEA,
            VehicleEnum::ERU => &mut mission.vehicles.ERU,
            VehicleEnum::MRA => &mut mission.vehicles.MRA,
        };
        let vehicle_id = select_vehicle_from_mission(
            self.db.clone(),
            mission.mission_id,
            vehicle.vehicle_name.to_string(),
        )
        .await
        .expect("Failed to find vehicle mission");

        // Clone self to call async method that takes self
        let default_stage = self.clone().create_default_stage(
            &stage_name,
            vehicle_id
        ).await;
        
        println!("Default stage created: {:?}", &default_stage);
        let stage_id = default_stage.stage_id;
        vehicle.stages.push(default_stage);

        if vehicle.current_stage == -1 {
            vehicle.current_stage = stage_id;
        }

        self.emit_state_update(&app_handle, &state)
    }

    pub async fn update_stage_area_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
        area: GeofenceType,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        let vehicle = match vehicle_name {
            VehicleEnum::MEA => &mut mission.vehicles.MEA,
            VehicleEnum::ERU => &mut mission.vehicles.ERU,
            VehicleEnum::MRA => &mut mission.vehicles.MRA,
        };

        let stage = vehicle
            .stages
            .iter_mut()
            .find(|s| s.stage_id == stage_id)
            .ok_or("Stage not found")?;

        stage.search_area = area;

        let search_area_string = format!(
            "[\n    {}\n]",
            stage.search_area
                .iter()
                .map(|coord| format!("({}, {})", coord.lat, coord.long))
                .collect::<Vec<String>>()
                .join(",\n    ")
        );
        
        let search_area_array: Vec<String> = vec![search_area_string.clone()];
        
        let vehicle_id = select_vehicle_from_mission(
            self.db.clone(),
            mission.mission_id,
            vehicle.vehicle_name.to_string(),
        ).await.expect("Failed to find vehicle mission");

        let _ = update_stage_area(
            self.db.clone(),
            stage.stage_id,
            search_area_array,
            vehicle_id,
        ).await.expect("Failed to update stage area");

        self.emit_state_update(&app_handle, &state)
    }

    pub async fn delete_stage_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
    ) -> Result<(), String> {
        println!("Deleting stage with ID: {}", stage_id);
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;

        let vehicle = match vehicle_name {
            VehicleEnum::MEA => &mut mission.vehicles.MEA,
            VehicleEnum::ERU => &mut mission.vehicles.ERU,
            VehicleEnum::MRA => &mut mission.vehicles.MRA,
        };

        let stage_index = vehicle
            .stages
            .iter()
            .position(|s| s.stage_id == stage_id)
            .ok_or("Stage not found")?;

        let stage = &vehicle.stages[stage_index];
        if matches!(stage.stage_status, MissionStageStatusEnum::Active | MissionStageStatusEnum::Complete) {
            return Err("Cannot delete current/completed stage".into());
        }
        delete_stage(self.db.clone(), stage_id)
            .await
            .expect("Failed to delete stage from database");

        vehicle.stages.remove(stage_index);
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn rename_stage_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
        stage_name: String,
    ) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;
        let vehicle = match vehicle_name {
            VehicleEnum::MEA => &mut mission.vehicles.MEA,
            VehicleEnum::ERU => &mut mission.vehicles.ERU,
            VehicleEnum::MRA => &mut mission.vehicles.MRA,
        };
        let stage = vehicle
            .stages
            .iter_mut()
            .find(|s| s.stage_id == stage_id)
            .ok_or("Stage not found")?;

        update_stage_name(self.db.clone(), stage.stage_id, &stage_name)
            .await
            .expect("Failed to update stage name");

        stage.stage_name = stage_name;
        self.emit_state_update(&app_handle, &state)
    }

    pub async fn transition_stage_helper(
        &self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
    ) -> Result<(), String> {
        println!("Transitioning stage for vehicle: {:?}", vehicle_name);
        let mut state = self.state.lock().await;
        let commands_api = CommandsApiImpl::default();
        let mission = state
            .missions
            .iter_mut()
            .find(|m| m.mission_id == mission_id)
            .ok_or("Mission not found")?;
        let vehicle = match vehicle_name {
            VehicleEnum::MEA => &mut mission.vehicles.MEA,
            VehicleEnum::ERU => &mut mission.vehicles.ERU,
            VehicleEnum::MRA => &mut mission.vehicles.MRA,
        };

        println!("Current Stage: {:?}", vehicle.current_stage);

        // Mark current stage as complete
        if let Some(stage) = vehicle.stages.iter_mut().find(|s| s.stage_id == vehicle.current_stage) {
            stage.stage_status = MissionStageStatusEnum::Complete;
        } else {
            println!("Stage with ID not found");
        }

        // Transition to next stage if available
        let transitioned_stage = transition_stage(
            self.db.clone(),
            mission.mission_id,
            vehicle.vehicle_name.to_string(),
            vehicle.current_stage,
        )
        .await
        .expect("Failed to transition stage");

        println!(
            "After Transition Stage: {:?}",
            transitioned_stage.unwrap_or(vehicle.current_stage)
        );

        if let Some(stage) = vehicle.stages.iter_mut().find(|s| s.stage_id == transitioned_stage.unwrap_or(vehicle.current_stage)) {
            vehicle.current_stage = transitioned_stage.unwrap_or(vehicle.current_stage);
            stage.stage_status = MissionStageStatusEnum::Active;

            // Send search area for the new active stage if it has valid coordinates
            if stage.search_area.len() >= 3 {  // Only send if we have at least 3 coordinates
                let coords: Vec<GeoCoordinate> = stage.search_area.iter()
                    .take(6) // Limit to 6 points
                    .map(|coord| GeoCoordinate {
                        lat: coord.lat,
                        long: coord.long,
                    })
                    .collect();
                
                // Send search area (commandID: 4) to the specific vehicle
                commands_api.clone().send_zone_update(
                    vehicle.vehicle_name.to_string(),
                    "4".to_string(),
                    coords
                ).await?;
            }
        } else {
            println!("No next stage available");
        }

        self.emit_state_update(&app_handle, &state)
    }
}

