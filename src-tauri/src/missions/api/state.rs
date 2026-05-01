/*
Implement helper methods on MissionApiImpl for loading data from 
the database and building/returning mission state 
(MissionsStruct, default data, in-memory state).
*/

use crate::missions::types::*;
use crate::missions::sql::{insert_new_stage, insert_new_mission};
use super::zones::convert_zone_to_json; 
use super::MissionApiImpl;

use sqlx::Row;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::sync::Mutex;

impl MissionApiImpl {
    /// Create new instance with initial state
    pub async fn new() -> Self {
        let mut initial_state = MissionsStruct {
            current_mission: 0,
            missions: vec![],
        };

        let database_connection = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://ngcp:ngcp@localhost:5433/ngcpdb")
            .await
            .expect("Failed to connect to the database");

        let all_mission_ids = sqlx::query("SELECT mission_id FROM missions ")
            .fetch_all(&database_connection)
            .await
            .expect("Failed to execute query");

        println!("Number of mission IDs: {}", all_mission_ids.len());
        if all_mission_ids.len() > 0 {
            for mission_id_row in all_mission_ids {
                let mission_id: i32 = mission_id_row.get("mission_id");
                let mission = sqlx::query(
                    "
                    SELECT 
                        missions.mission_id,
                        missions.mission_name,
                        missions.status,
                        missions.keep_in_zones,
                        missions.keep_out_zones,
                        vehicles.vehicle_name,
                        vehicles.current_stage_id AS current_stage,
                        vehicles.is_auto,
                        vehicles.patient_status,
                        stages.stage_id,
                        stages.stage_name,
                        stages.search_area,
                        stages.target_coordinate,
                        stages.status AS stage_status
                    FROM missions
                    LEFT JOIN vehicles ON missions.mission_id = vehicles.mission_id
                    LEFT JOIN stages ON vehicles.vehicle_id = stages.vehicle_id
                    WHERE missions.mission_id = $1
                    ",
                )
                .bind(mission_id)
                .fetch_all(&database_connection)
                .await
                .expect("Failed to execute query");

                // Set current mission ID if a mission has a status of "Active"
                if mission[0].try_get::<String, _>("status").unwrap_or_else(|_| "Inactive".to_string()) == "Active" {
                    initial_state.current_mission = mission_id;
                }

                let mea_row = mission.iter()
                    .find(|row| row.get::<String, _>("vehicle_name") == "MEA")
                    .expect("Expected MEA row");

                let eru_row = mission.iter()
                    .find(|row| row.get::<String, _>("vehicle_name") == "ERU")
                    .expect("Expected ERU row");

                let mra_row = mission.iter()
                    .find(|row| row.get::<String, _>("vehicle_name") == "MRA")
                    .expect("Expected MRA row");

                initial_state.missions.push(MissionStruct {
                    mission_name: mission[0].get("mission_name"),
                    mission_id: mission[0].get("mission_id"),
                    mission_status: match mission[0]
                        .try_get::<String, _>("status")
                        .unwrap_or_else(|_| "Inactive".to_string())
                        .as_str()
                    {
                        "Active" => MissionStageStatusEnum::Active,
                        "Inactive" => MissionStageStatusEnum::Inactive,
                        "Complete" => MissionStageStatusEnum::Complete,
                        "Failed" => MissionStageStatusEnum::Failed,
                        _ => MissionStageStatusEnum::Inactive,
                    },
                    vehicles: VehiclesStruct {
                        MEA: VehicleStruct {
                            vehicle_name: VehicleEnum::MEA,
                            current_stage: mea_row.get("current_stage"),
                            is_auto: mea_row.get("is_auto"),
                            patient_status: 
                                match mea_row.get::<String, _>("patient_status").as_str() {
                                    "Unsecured" => Some(PatientStatusEnum::Unsecured),
                                    "Secured" => Some(PatientStatusEnum::Secured),
                                    _ => Some(PatientStatusEnum::Unsecured),
                                }, 
                            stages: 
                            if mea_row.get::<i32, _>("current_stage") != -1 {
                                mission.iter()
                                    .filter(|row| row.get::<String, _>("vehicle_name") == "MEA")
                                    .map(|row| StageStruct {
                                        stage_name: row.get("stage_name"),
                                        stage_id: row.get("stage_id"),
                                        stage_status: match row
                                            .try_get::<String, _>("stage_status")
                                            .unwrap_or_else(|_| "Inactive".to_string())
                                            .as_str()
                                        {
                                            "Active" => MissionStageStatusEnum::Active,
                                            "Inactive" => MissionStageStatusEnum::Inactive,
                                            "Complete" => MissionStageStatusEnum::Complete,
                                            "Failed" => MissionStageStatusEnum::Failed,
                                            _ => MissionStageStatusEnum::Inactive,
                                        },
                                        search_area:
                                        match row.try_get::<Vec<String>, _>("search_area").unwrap_or_else(|_| Vec::new()) {
                                            search_areas => search_areas
                                                .into_iter()
                                                .filter_map(|area: String| {
                                                    serde_json::from_str::<Vec<GeoCoordinateStruct>>(convert_zone_to_json(&area).as_str()).ok()
                                                })
                                                .flatten()
                                                .collect::<Vec<GeoCoordinateStruct>>()
                                            }
                                    })
                                    .collect()
                            } else {
                                vec![]
                            }
                        },
                        ERU: VehicleStruct {
                            vehicle_name: VehicleEnum::ERU,
                            current_stage: eru_row.get("current_stage"),
                            is_auto: eru_row.get("is_auto"),
                            patient_status: 
                                match eru_row.get::<String, _>("patient_status").as_str() {
                                    "Unsecured" => Some(PatientStatusEnum::Unsecured),
                                    "Secured" => Some(PatientStatusEnum::Secured),
                                    _ => Some(PatientStatusEnum::Unsecured),
                                },
                            stages: 
                            if eru_row.get::<i32, _>("current_stage") != -1 {
                                mission.iter()
                                    .filter(|row| row.get::<String, _>("vehicle_name") == "ERU")
                                    .map(|row| StageStruct {
                                        stage_name: row.get("stage_name"),
                                        stage_id: row.get("stage_id"),
                                        stage_status: match row
                                            .try_get::<String, _>("stage_status")
                                            .unwrap_or_else(|_| "Inactive".to_string())
                                            .as_str()
                                        {
                                            "Active" => MissionStageStatusEnum::Active,
                                            "Inactive" => MissionStageStatusEnum::Inactive,
                                            "Complete" => MissionStageStatusEnum::Complete,
                                            "Failed" => MissionStageStatusEnum::Failed,
                                            _ => MissionStageStatusEnum::Inactive,
                                        },
                                        search_area: 
                                            match row.try_get::<Vec<String>, _>("search_area").unwrap_or_else(|_| Vec::new()) {
                                            search_areas => search_areas
                                                .into_iter()
                                                .filter_map(|area: String| {
                                                    serde_json::from_str::<Vec<GeoCoordinateStruct>>(convert_zone_to_json(&area).as_str()).ok()
                                                })
                                                .flatten()
                                                .collect::<Vec<GeoCoordinateStruct>>()
                                            }
                                    })
                                    .collect()
                            } else {
                                vec![]
                            }
                        },
                        MRA: VehicleStruct {
                            vehicle_name: VehicleEnum::MRA,
                            current_stage: mra_row.get("current_stage"),
                            is_auto: mra_row.get("is_auto"),
                            patient_status:
                                match mra_row.get::<String, _>("patient_status").as_str() {
                                    "Unsecured" => Some(PatientStatusEnum::Unsecured),
                                    "Secured" => Some(PatientStatusEnum::Secured),
                                    _ => Some(PatientStatusEnum::Unsecured),
                                },
                            stages: 
                            if mra_row.get::<i32, _>("current_stage") != -1 {
                                mission.iter()
                                    .filter(|row| row.get::<String, _>("vehicle_name") == "MRA")
                                    .map(|row| StageStruct {
                                        stage_name: row.get("stage_name"),
                                        stage_id: row.get("stage_id"),
                                        stage_status: match row
                                            .try_get::<String, _>("stage_status")
                                            .unwrap_or_else(|_| "Inactive".to_string())
                                            .as_str()
                                        {
                                            "Active" => MissionStageStatusEnum::Active,
                                            "Inactive" => MissionStageStatusEnum::Inactive,
                                            "Complete" => MissionStageStatusEnum::Complete,
                                            "Failed" => MissionStageStatusEnum::Failed,
                                            _ => MissionStageStatusEnum::Inactive,
                                        },
                                        search_area:
                                            match row.try_get::<Vec<String>, _>("search_area").unwrap_or_else(|_| Vec::new()) {
                                            search_areas => search_areas
                                                .into_iter()
                                                .filter_map(|area: String| {
                                                    serde_json::from_str::<Vec<GeoCoordinateStruct>>(convert_zone_to_json(&area).as_str()).ok()
                                                })
                                                .flatten()
                                                .collect::<Vec<GeoCoordinateStruct>>()
                                            },
                                    })
                                    .collect()
                            } else {
                                vec![]
                            }
                        },
                    },
                    zones: ZonesStruct {
                        keep_in_zones: mission[0]
                            .try_get::<Vec<String>, _>("keep_in_zones")
                            .unwrap_or_else(|_| Vec::new())
                            .into_iter()
                            .map(|zone| {
                                serde_json::from_str::<Vec<GeoCoordinateStruct>>(convert_zone_to_json(&zone).as_str())
                                    .unwrap_or_else(|_| Vec::new())
                            })
                            .collect(),
                        keep_out_zones:
                            mission[0]
                                .try_get::<Vec<String>, _>("keep_out_zones")
                                .unwrap_or_else(|_| Vec::new())
                                .into_iter()
                                .map(|zone| {
                                    serde_json::from_str::<Vec<GeoCoordinateStruct>>(convert_zone_to_json(&zone).as_str())
                                        .unwrap_or_else(|_| Vec::new())
                                })
                                .collect(),
                    },
                });
            }
        } 

        Self {
            state: Arc::new(Mutex::new(initial_state)),
            db: database_connection,
        }
    }

    /// Create default stage configuration
    pub async fn create_default_stage(self, name: &str, id: i32) -> StageStruct {
        let stage_id = insert_new_stage(self.db.clone(), id, name)
            .await
            .expect("Failed to insert new stage into database");

        StageStruct {
            stage_name: name.to_string(),
            stage_id: stage_id,
            stage_status: MissionStageStatusEnum::Inactive,
            search_area: vec![],
        }
    }

    /// Create default mission configuration
    pub async fn create_default_mission(self, name: &str) -> MissionStruct {
        let new_mission_id = insert_new_mission(self.db, name).await.unwrap_or(0);

        MissionStruct {
            mission_name: name.to_string(),
            mission_id: new_mission_id,
            mission_status: MissionStageStatusEnum::Inactive,
            vehicles: VehiclesStruct {
                MEA: VehicleStruct {
                    vehicle_name: VehicleEnum::MEA,
                    current_stage: -1,
                    is_auto: Some(false),
                    patient_status: Some(PatientStatusEnum::Unsecured),
                    stages: vec![],
                },
                ERU: VehicleStruct {
                    vehicle_name: VehicleEnum::ERU,
                    current_stage: -1,
                    is_auto: Some(false),
                    patient_status: Some(PatientStatusEnum::Unsecured),
                    stages: vec![],
                },
                MRA: VehicleStruct {
                    vehicle_name: VehicleEnum::MRA,
                    current_stage: -1,
                    is_auto: None,
                    patient_status: Some(PatientStatusEnum::Unsecured),
                    stages: vec![],
                },
            },
            zones: ZonesStruct {
                keep_in_zones: vec![],
                keep_out_zones: vec![],
            },
        }
    }
}
