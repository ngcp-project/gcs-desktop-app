/*
Define all mission-related database functions (mission CRUD, vehicle selection and auto-mode, stage CRUD and transition, zone updates).
*/
use sqlx::{query, PgPool, Row};

pub async fn insert_new_mission(db_conn: PgPool, mission_name: &str) -> Result<i32, sqlx::Error> {
    let new_mission = query(
        "
        INSERT INTO missions(mission_name, keep_in_zones, keep_out_zones) 
        VALUES ($1, $2, $3) RETURNING mission_id
    ",
    )
    .bind(mission_name)
    .bind(&Vec::<String>::new())
    .bind(&Vec::<String>::new())
    .fetch_one(&db_conn)
    .await
    .expect("Failed to insert dummy data into missions");

    let new_mission_id: i32 = new_mission.get::<i32, _>("mission_id");

    let _insert_mra_vehicle = query(
        "
        INSERT INTO vehicles(mission_id, vehicle_name, current_stage_id)
        VALUES ($1, $2, $3) RETURNING vehicle_id
    ",
    )
    .bind(new_mission_id)
    .bind("MRA")
    .bind(-1)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to insert dummy data into vehicles");

    let _insert_eru_vehicle = query(
        "
        INSERT INTO vehicles(mission_id, vehicle_name, current_stage_id)
        VALUES ($1, $2, $3) RETURNING vehicle_id
    ",
    )
    .bind(new_mission_id)
    .bind("ERU")
    .bind(-1)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to insert dummy data into vehicles");

    let _insert_mea_vehicle = query(
        "
        INSERT INTO vehicles(mission_id, vehicle_name, current_stage_id)
        VALUES ($1, $2, $3) RETURNING vehicle_id
    ",
    )
    .bind(new_mission_id)
    .bind("MEA")
    .bind(-1)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to insert dummy data into vehicles");

    return Ok(new_mission_id);
}

pub async fn update_mission_name(
    db_conn: PgPool,
    mission_id: i32,
    new_mission_name: &str,
) -> Result<(), sqlx::Error> {
    query(
        "
        UPDATE missions SET mission_name = $1 WHERE mission_id = $2
    ",
    )
    .bind(new_mission_name)
    .bind(mission_id)
    .execute(&db_conn)
    .await
    .expect("Failed to update mission name");

    Ok(())
}

pub async fn delete_mission(db_conn: PgPool, mission_id: i32) -> Result<(), sqlx::Error> {
    query(
        "
        DELETE FROM missions WHERE mission_id = $1
    ",
    )
    .bind(mission_id)
    .execute(&db_conn)
    .await
    .expect("Failed to delete mission");

    Ok(())
}

pub async fn select_vehicle_from_mission(
    db_conn: PgPool,
    mission_id: i32,
    vehicle_name: String,
) -> Result<i32, sqlx::Error> {
    let vehicle = query(
        "
        SELECT vehicle_id FROM vehicles WHERE mission_id = $1 AND vehicle_name = $2
    ",
    )
    .bind(mission_id)
    .bind(vehicle_name)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to find vehicle in mission");

    let vehicle_id: i32 = vehicle.get::<i32, _>("vehicle_id");

    return Ok(vehicle_id);
}

pub async fn insert_new_stage(
    db_conn: PgPool,
    vehicle_id: i32,
    stage_name: &str,
) -> Result<i32, sqlx::Error> {
    let new_stage = query(
        "
        INSERT INTO stages(vehicle_id, stage_name) 
        VALUES ($1, $2) RETURNING stage_id
    ",
    )
    .bind(vehicle_id)
    .bind(stage_name)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to insert dummy data into stages");

    let new_stage_id: i32 = new_stage.get::<i32, _>("stage_id");

    // set current stage id if previously didn't exist (-1)
    let current_stage = query(
        "
        SELECT current_stage_id FROM vehicles WHERE vehicle_id = $1
    ",
    )
    .bind(vehicle_id)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to find vehicle in mission");

    if current_stage.get::<i32, _>("current_stage_id") == -1 {
        query(
            "
            UPDATE vehicles SET current_stage_id = $1 WHERE vehicle_id = $2
        ",
        )
        .bind(new_stage_id)
        .bind(vehicle_id)
        .execute(&db_conn)
        .await
        .expect("Failed to update vehicle current stage id");

        println!("Updated vehicle current stage id to {}", new_stage_id);
    }

    return Ok(new_stage_id);
}

pub async fn delete_stage(db_conn: PgPool, stage_id: i32) -> Result<(), sqlx::Error> {
    query(
        "
        DELETE FROM stages WHERE stage_id = $1
    ",
    )
    .bind(stage_id)
    .execute(&db_conn)
    .await
    .expect("Failed to delete stage");

    Ok(())
}

pub async fn reset_vehicle_current_stage(
    db_conn: PgPool,
    mission_id: i32,
    vehicle_name: String,
) -> Result<(), sqlx::Error> {
    query(
        "
        UPDATE vehicles SET current_stage_id = -1
        WHERE mission_id = $1 AND vehicle_name = $2
    ",
    )
    .bind(mission_id)
    .bind(vehicle_name)
    .execute(&db_conn)
    .await
    .expect("Failed to reset vehicle current stage");

    Ok(())
}

pub async fn update_stage_name(
    db_conn: PgPool,
    stage_id: i32,
    new_stage_name: &str,
) -> Result<(), sqlx::Error> {
    query(
        "
        UPDATE stages SET stage_name = $1 WHERE stage_id = $2
    ",
    )
    .bind(new_stage_name)
    .bind(stage_id)
    .execute(&db_conn)
    .await
    .expect("Failed to update stage name");

    Ok(())
}

pub async fn update_stage_status(
    db_conn: PgPool,
    stage_id: i32,
    status: &str,
) -> Result<(), sqlx::Error> {
    query(
        "
        UPDATE stages SET status = $1 WHERE stage_id = $2
    ",
    )
    .bind(status)
    .bind(stage_id)
    .execute(&db_conn)
    .await
    .expect("Failed to update stage status");

    Ok(())
}

pub async fn update_stage_area(
    db_conn: PgPool,
    stage_id: i32,
    area: Vec<String>,
    vehicle_id: i32,
) -> Result<i32, sqlx::Error> {
    query(
        "
        UPDATE stages SET search_area = $1 WHERE stage_id = $2
    ",
    )
    .bind(area)
    .bind(stage_id)
    .execute(&db_conn)
    .await
    .expect("Failed to update stage area");

    // set status to active if current_stage_id matches stage_id
    let current_stage = query(
        "
        SELECT current_stage_id FROM vehicles WHERE vehicle_id = $1
    ",
    )
    .bind(vehicle_id)
    .fetch_one(&db_conn)
    .await
    .expect("Failed to find vehicle in mission");

    let current_stage_id = current_stage.get::<i32, _>("current_stage_id");

    if current_stage_id == stage_id {
        query(
            "
            UPDATE stages
            SET status = 'Active'
            WHERE stage_id = $1
        ",
        )
        .bind(stage_id)
        .execute(&db_conn)
        .await
        .expect("Failed to update stage status");
        println!("Updated current stage status to 'Active'");
    }

    Ok(current_stage_id)
}

pub async fn update_auto_mode_vehicle(
    db_conn: PgPool,
    mission_id: i32,
    vehicle_name: String,
    is_auto: bool,
) -> Result<(), sqlx::Error> {
    query(
        "
        UPDATE vehicles SET auto_mode = $1 WHERE vehicle_name = $2 AND mission_id = $3
    ",
    )
    .bind(is_auto)
    .bind(vehicle_name)
    .bind(mission_id)
    .execute(&db_conn)
    .await
    .expect("Failed to update vehicle auto mode");

    Ok(())
}

pub async fn transition_stage(
    db_conn: PgPool,
    mission_id: i32,
    vehicle_name: String,
    current_stage_id: i32,
) -> Result<Option<i32>, sqlx::Error> {
    let rows = query(
        "
        SELECT stage_id
        FROM stages
        WHERE vehicle_id = (
            SELECT vehicle_id
            FROM vehicles
            WHERE mission_id = $1 AND vehicle_name = $2
        )
        ORDER BY stage_id
    ",
    )
    .bind(mission_id)
    .bind(vehicle_name.clone())
    .fetch_all(&db_conn)
    .await
    .expect("Failed to select stage_id");

    let stage_ids: Vec<i32> = rows.iter().map(|row| row.get("stage_id")).collect();

    // get next index
    if let Some(pos) = stage_ids.iter().position(|&id| id == current_stage_id) {
        if let Some(&next_stage_id) = stage_ids.get(pos + 1) {
            query(
                "
                UPDATE vehicles
                SET current_stage_id = $1
                WHERE mission_id = $2 AND vehicle_name = $3
            ",
            )
            .bind(next_stage_id)
            .bind(mission_id)
            .bind(vehicle_name)
            .execute(&db_conn)
            .await
            .expect("Failed to transition next stage");

            // query to update completed stage status
            query(
                "
                UPDATE stages
                SET status = 'Complete'
                WHERE stage_id = $1
            ",
            )
            .bind(current_stage_id)
            .execute(&db_conn)
            .await
            .expect("Failed to update stage status");

            // query to update active stage status
            query(
                "
                UPDATE stages
                SET status = 'Active'
                WHERE stage_id = $1
            ",
            )
            .bind(next_stage_id)
            .execute(&db_conn)
            .await
            .expect("Failed to update stage status");

            return Ok(Some(next_stage_id));
        }
    }

    Ok(None)
}

// pub async fn update_complete_stage(
//     db_conn: PgPool,
//     mission_id: i32,
//     vehicle_name: String,
//     stage_id: i32,
// ) -> Result<(), sqlx::Error> {
//     query("
//         UPDATE stages SET completed = TRUE WHERE stage_id = $1 AND vehicle_id = (
//             SELECT vehicle_id FROM vehicles WHERE mission_id = $2 AND vehicle_name = $3
//         )
//     ")
//     .bind(stage_id)
//     .bind(mission_id)
//     .bind(vehicle_name)
//     .execute(&db_conn)
//     .await
//     .expect("Failed to update stage completion");

//     Ok(())
// }

pub async fn update_mission_status(
    db_conn: PgPool,
    mission_id: i32,
    status: &str,
) -> Result<(), sqlx::Error> {
    query(
        "
        UPDATE missions SET status = $1 WHERE mission_id = $2
    ",
    )
    .bind(status)
    .bind(mission_id)
    .execute(&db_conn)
    .await
    .expect("Failed to update mission completion");

    Ok(())
}

pub async fn update_zones(
    db_conn: PgPool,
    mission_id: i32,
    keep_in_zones: Vec<String>,
    keep_out_zones: Vec<String>,
) -> Result<(), sqlx::Error> {
    query(
        "
        INSERT INTO missions (mission_id, keep_in_zones, keep_out_zones)
        VALUES ($1, $2, $3)
        ON CONFLICT (mission_id) DO UPDATE
        SET keep_in_zones = EXCLUDED.keep_in_zones,
            keep_out_zones = EXCLUDED.keep_out_zones
    ",
    ) // waaah where is my UPSERT  T^T  ~tho this is basically an upsert
    .bind(mission_id)
    .bind(keep_in_zones)
    .bind(keep_out_zones)
    .execute(&db_conn)
    .await
    .expect("Failed to upsert mission zones");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    // Test database URL - using a separate test database
    const TEST_DB_URL: &str = "postgres://ngcp:ngcp@localhost:5433/ngcpdb_test";

    // Helper function to setup test database
    async fn setup_test_db() -> PgPool {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(TEST_DB_URL)
            .await
            .expect("Failed to connect to test database");

        // Drop tables first to ensure clean state
        query("DROP TABLE IF EXISTS stages CASCADE")
            .execute(&pool)
            .await
            .expect("Failed to drop stages table");

        query("DROP TABLE IF EXISTS vehicles CASCADE")
            .execute(&pool)
            .await
            .expect("Failed to drop vehicles table");

        query("DROP TABLE IF EXISTS missions CASCADE")
            .execute(&pool)
            .await
            .expect("Failed to drop missions table");

        // Create tables
        query(
            "CREATE TABLE missions (
                mission_id SERIAL PRIMARY KEY,
                mission_name VARCHAR(255),
                keep_in_zones TEXT[] NOT NULL,
                keep_out_zones TEXT[] NOT NULL,
                status TEXT DEFAULT 'Inactive'
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create missions table");

        query(
            "CREATE TABLE vehicles (
                mission_id INTEGER REFERENCES missions ON DELETE CASCADE,
                vehicle_id SERIAL UNIQUE,
                vehicle_name VARCHAR(255) NOT NULL,
                current_stage_id INTEGER NOT NULL,
                auto_mode BOOLEAN DEFAULT FALSE,
                patient_status VARCHAR(255) DEFAULT 'Unsecured',
                PRIMARY KEY (mission_id, vehicle_id)
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create vehicles table");

        query(
            "CREATE TABLE stages (
                stage_id SERIAL PRIMARY KEY,
                vehicle_id INTEGER REFERENCES vehicles(vehicle_id) ON DELETE CASCADE,
                search_area TEXT[],
                stage_name VARCHAR(255) NOT NULL,
                target_coordinate TEXT,
                status TEXT DEFAULT 'Inactive'
            )",
        )
        .execute(&pool)
        .await
        .expect("Failed to create stages table");

        pool
    }

    // Helper function to cleanup test database
    async fn cleanup_test_db(pool: &PgPool) {
        query("DROP TABLE IF EXISTS stages CASCADE")
            .execute(pool)
            .await
            .expect("Failed to drop stages table");

        query("DROP TABLE IF EXISTS vehicles CASCADE")
            .execute(pool)
            .await
            .expect("Failed to drop vehicles table");

        query("DROP TABLE IF EXISTS missions CASCADE")
            .execute(pool)
            .await
            .expect("Failed to drop missions table");
    }

    #[tokio::test]
    async fn test_insert_new_mission() {
        // TEST PURPOSE: Verify that insert_new_mission creates a mission with 3 vehicles

        // ARRANGE: Set up test database
        let pool = setup_test_db().await;

        // ACT: Insert a new mission
        let mission_id = insert_new_mission(pool.clone(), "Test Mission")
            .await
            .expect("Failed to insert mission");

        // ASSERT: Verify mission was created
        let mission = query("SELECT mission_name FROM missions WHERE mission_id = $1")
            .bind(mission_id)
            .fetch_one(&pool)
            .await
            .expect("Mission not found");

        let mission_name: String = mission.get("mission_name");
        assert_eq!(mission_name, "Test Mission", "Mission name should match");

        // ASSERT: Verify 3 vehicles were created (MRA, ERU, MEA)
        let vehicle_count = query("SELECT COUNT(*) as count FROM vehicles WHERE mission_id = $1")
            .bind(mission_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count vehicles");

        let count: i64 = vehicle_count.get("count");
        assert_eq!(count, 3, "Should create exactly 3 vehicles");

        // ASSERT: Verify vehicle names
        let vehicles =
            query("SELECT vehicle_name FROM vehicles WHERE mission_id = $1 ORDER BY vehicle_name")
                .bind(mission_id)
                .fetch_all(&pool)
                .await
                .expect("Failed to fetch vehicles");

        assert_eq!(vehicles.len(), 3);
        assert_eq!(vehicles[0].get::<String, _>("vehicle_name"), "ERU");
        assert_eq!(vehicles[1].get::<String, _>("vehicle_name"), "MEA");
        assert_eq!(vehicles[2].get::<String, _>("vehicle_name"), "MRA");

        // PAUSE HERE - Check pgAdmin to see the data!
        println!("Test passed! Mission ID: {}", mission_id);
        println!("Open pgAdmin and refresh the ngcpdb_test database to see:");
        println!("   - missions table (1 row)");
        println!("   - vehicles table (3 rows: ERU, MEA, MRA)");
        println!("  Sleeping for 30 seconds... Check pgAdmin now!");
        std::thread::sleep(std::time::Duration::from_secs(30));

        // CLEANUP
        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_update_mission_name() {
        // TEST PURPOSE: Verify that update_mission_name changes the mission name correctly

        // ARRANGE: Set up test database and create a mission
        let pool = setup_test_db().await;
        let mission_id = insert_new_mission(pool.clone(), "Original Name")
            .await
            .expect("Failed to insert mission");

        // ACT: Update the mission name
        update_mission_name(pool.clone(), mission_id, "Updated Name")
            .await
            .expect("Failed to update mission name");

        // ASSERT: Verify the name was updated
        let mission = query("SELECT mission_name FROM missions WHERE mission_id = $1")
            .bind(mission_id)
            .fetch_one(&pool)
            .await
            .expect("Mission not found");

        let mission_name: String = mission.get("mission_name");
        assert_eq!(
            mission_name, "Updated Name",
            "Mission name should be updated"
        );

        // CLEANUP
        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_delete_mission() {
        // TEST PURPOSE: Verify that delete_mission removes the mission and cascades to vehicles

        // ARRANGE: Set up test database and create a mission
        let pool = setup_test_db().await;
        let mission_id = insert_new_mission(pool.clone(), "Mission to Delete")
            .await
            .expect("Failed to insert mission");

        // ACT: Delete the mission
        delete_mission(pool.clone(), mission_id)
            .await
            .expect("Failed to delete mission");

        // ASSERT: Verify mission was deleted
        let mission_result = query("SELECT mission_id FROM missions WHERE mission_id = $1")
            .bind(mission_id)
            .fetch_optional(&pool)
            .await
            .expect("Query failed");

        assert!(mission_result.is_none(), "Mission should be deleted");

        // ASSERT: Verify vehicles were also deleted (CASCADE)
        let vehicle_count = query("SELECT COUNT(*) as count FROM vehicles WHERE mission_id = $1")
            .bind(mission_id)
            .fetch_one(&pool)
            .await
            .expect("Failed to count vehicles");

        let count: i64 = vehicle_count.get("count");
        assert_eq!(count, 0, "All vehicles should be deleted via CASCADE");

        // CLEANUP
        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_select_vehicle_from_mission() {
        // TEST PURPOSE: Verify that select_vehicle_from_mission finds the correct vehicle

        // ARRANGE: Set up test database and create a mission
        let pool = setup_test_db().await;
        let mission_id = insert_new_mission(pool.clone(), "Test Mission")
            .await
            .expect("Failed to insert mission");

        // ACT: Select the MRA vehicle
        let vehicle_id = select_vehicle_from_mission(pool.clone(), mission_id, "MRA".to_string())
            .await
            .expect("Failed to select vehicle");

        // ASSERT: Verify we got a valid vehicle_id
        assert!(vehicle_id > 0, "Should return a valid vehicle_id");

        // ASSERT: Verify the vehicle exists with the correct name
        let vehicle = query("SELECT vehicle_name FROM vehicles WHERE vehicle_id = $1")
            .bind(vehicle_id)
            .fetch_one(&pool)
            .await
            .expect("Vehicle not found");

        let vehicle_name: String = vehicle.get("vehicle_name");
        assert_eq!(vehicle_name, "MRA", "Vehicle name should match");

        // CLEANUP
        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_update_mission_status() {
        // TEST PURPOSE: Verify that update_mission_status changes the mission status

        // ARRANGE: Set up test database and create a mission
        let pool = setup_test_db().await;
        let mission_id = insert_new_mission(pool.clone(), "Test Mission")
            .await
            .expect("Failed to insert mission");

        // ACT: Update mission status to Active
        update_mission_status(pool.clone(), mission_id, "Active")
            .await
            .expect("Failed to update mission status");

        // ASSERT: Verify the status was updated
        let mission = query("SELECT status FROM missions WHERE mission_id = $1")
            .bind(mission_id)
            .fetch_one(&pool)
            .await
            .expect("Mission not found");

        let status: String = mission.get("status");
        assert_eq!(status, "Active", "Mission status should be Active");

        // CLEANUP
        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    async fn test_update_zones() {
        // TEST PURPOSE: Verify that update_zones performs an UPSERT correctly

        // ARRANGE: Set up test database and create a mission
        let pool = setup_test_db().await;
        let mission_id = insert_new_mission(pool.clone(), "Test Mission")
            .await
            .expect("Failed to insert mission");

        let keep_in = vec!["[(33.93257,-117.63059),(34.05220,-118.24370)]".to_string()];
        let keep_out = vec!["[(37.77490,-122.41940),(40.71280,-74.00600)]".to_string()];

        // ACT: Update zones
        update_zones(pool.clone(), mission_id, keep_in.clone(), keep_out.clone())
            .await
            .expect("Failed to update zones");

        // ASSERT: Verify zones were updated
        let mission =
            query("SELECT keep_in_zones, keep_out_zones FROM missions WHERE mission_id = $1")
                .bind(mission_id)
                .fetch_one(&pool)
                .await
                .expect("Mission not found");

        let db_keep_in: Vec<String> = mission.get("keep_in_zones");
        let db_keep_out: Vec<String> = mission.get("keep_out_zones");

        assert_eq!(db_keep_in, keep_in, "Keep-in zones should match");
        assert_eq!(db_keep_out, keep_out, "Keep-out zones should match");

        // ACT: Update zones again (testing UPSERT)
        let new_keep_in = vec!["[(40.00000,-120.00000)]".to_string()];
        update_zones(
            pool.clone(),
            mission_id,
            new_keep_in.clone(),
            keep_out.clone(),
        )
        .await
        .expect("Failed to update zones second time");

        // ASSERT: Verify zones were updated (not inserted as new row)
        let mission_count = query("SELECT COUNT(*) as count FROM missions")
            .fetch_one(&pool)
            .await
            .expect("Failed to count missions");

        let count: i64 = mission_count.get("count");
        assert_eq!(
            count, 1,
            "Should still have only 1 mission (UPSERT, not INSERT)"
        );

        // CLEANUP
        cleanup_test_db(&pool).await;
    }
}
