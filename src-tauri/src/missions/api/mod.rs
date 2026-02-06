/*
Define the public missions API surface: MissionApi trait, MissionApiImpl 
struct, and the macro-decorated impl MissionApi for MissionApiImpl.

Keep trait methods as thin wrappers that call helper methods
implemented in the other api/ files. 
*/

use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::PgPool;
use tauri::{AppHandle, Runtime};
use crate::missions::types::*;

pub mod events;
pub mod missions;
pub mod stages;
pub mod state;
pub mod zones;

#[derive(Clone)]
pub struct MissionApiImpl {
    state: Arc<Mutex<MissionsStruct>>,
    db: PgPool,
}

#[taurpc::procedures(
    event_trigger = MissionEventTrigger,
    path = "mission"
)]
pub trait MissionApi {
    // ----------------------------
    // Event Handlers
    // ----------------------------
    #[taurpc(event)]
    async fn on_updated(new_data: MissionsStruct);

    // ----------------------------
    // State Management
    // ----------------------------
    async fn get_default_data() -> MissionsStruct;
    async fn get_all_missions() -> MissionsStruct;
    
    // ----------------------------
    // Mission Operations
    // ----------------------------
    async fn rename_mission(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        mission_name: String,
    ) -> Result<(), String>;
    async fn get_mission_data(mission_id: i32) -> MissionStruct;
    async fn create_mission(
        app_handle: AppHandle<impl Runtime>,
        mission_name: String,
    ) -> Result<(), String>;
    async fn delete_mission(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
    ) -> Result<(), String>;
    async fn start_mission(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
    ) -> Result<(), String>;

    
    // ----------------------------
    // Vehicle Operations
    // ----------------------------
    async fn set_auto_mode(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        is_auto: bool,
    ) -> Result<(), String>;

    // ----------------------------
    // Stage Operations
    // ----------------------------
    async fn add_stage(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_name: String,
    ) -> Result<(), String>;

    async fn delete_stage(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
    ) -> Result<(), String>;

    async fn rename_stage(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
        stage_name: String,
    ) -> Result<(), String>;

    async fn transition_stage(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
    ) -> Result<(), String>;

    async fn update_stage_area(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
        area: GeofenceType,
    ) -> Result<(), String>;

    // ----------------------------
    // Zone Operations
    // ----------------------------
    async fn add_zone(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
    ) -> Result<(), String>;
    async fn update_zone(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
        zone_index: i32,
        zone_coords: GeofenceType,
    ) -> Result<(), String>;
    async fn delete_zone(
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
        zone_index: i32,
    ) -> Result<(), String>;
}

/*==============================================================================
 * MissionApi Trait Implementation
 *============================================================================*/

#[taurpc::resolvers]
impl MissionApi for MissionApiImpl {
    // ----------------------------------
    // State Management Implementations
    // ----------------------------------
    async fn get_default_data(self) -> MissionsStruct {
        Self::new().await.state.lock().await.clone()
    }

    async fn get_all_missions(self) -> MissionsStruct {
        self.state.lock().await.clone()
    }

    // ----------------------------------
    // Mission Operations Implementations
    // ----------------------------------
    async fn get_mission_data(self, mission_id: i32) -> MissionStruct {
        self.get_mission_data_helper(mission_id).await
    }

    async fn rename_mission(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        mission_name: String,
    ) -> Result<(), String> {
        self.rename_mission_helper(app_handle, mission_id, mission_name).await
    }

    async fn create_mission(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_name: String,
    ) -> Result<(), String> {
        self.create_mission_helper(app_handle, mission_name).await
    }

    async fn delete_mission(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
    ) -> Result<(), String> {
        self.delete_mission_helper(app_handle, mission_id).await
    }

    async fn start_mission(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
    ) -> Result<(), String> {
        self.start_mission_helper(app_handle, mission_id).await
    }

    // ----------------------------------
    // Vehicle Operations Implementations
    // ----------------------------------
    async fn set_auto_mode(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        is_auto: bool,
    ) -> Result<(), String> {
        self.set_auto_mode_helper(app_handle, mission_id, vehicle_name, is_auto).await
    }

    // ----------------------------------
    // Stage Operations Implementations
    // ----------------------------------
    async fn add_stage(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_name: String,
    ) -> Result<(), String> {
        self.add_stage_helper(app_handle, mission_id, vehicle_name, stage_name).await
    }

    async fn update_stage_area(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
        area: GeofenceType,
    ) -> Result<(), String> {
        self.update_stage_area_helper(app_handle, mission_id, vehicle_name, stage_id, area).await
    }

    async fn delete_stage(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
    ) -> Result<(), String> {
        self.delete_stage_helper(app_handle, mission_id, vehicle_name, stage_id).await
    }

    async fn rename_stage(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
        stage_id: i32,
        stage_name: String,
    ) -> Result<(), String> {
        self.rename_stage_helper(app_handle, mission_id, vehicle_name, stage_id, stage_name).await
    }

    async fn transition_stage(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        vehicle_name: VehicleEnum,
    ) -> Result<(), String> {
        self.transition_stage_helper(app_handle, mission_id, vehicle_name).await
    }

    // ----------------------------------
    // Zone Operations Implementations
    // ----------------------------------
    async fn add_zone(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
    ) -> Result<(), String> {
        self.add_zone_helper(app_handle, mission_id, zone_type).await
    }

    async fn update_zone(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
        zone_index: i32,
        zone_coords: GeofenceType,
    ) -> Result<(), String> {
        self.update_zone_helper(app_handle, mission_id, zone_type, zone_index, zone_coords).await
    }

    async fn delete_zone(
        self,
        app_handle: AppHandle<impl Runtime>,
        mission_id: i32,
        zone_type: ZoneType,
        zone_index: i32,
    ) -> Result<(), String> {
        self.delete_zone_helper(app_handle, mission_id, zone_type, zone_index).await
    }
}


