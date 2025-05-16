use crate::manager_mygrid::errors::MyGridError;
use crate::manager_mygrid::models::{BaseData, Block};

pub mod models;
pub mod errors;


/// Reads current schedule from mygrid and returns the block(s)
/// 
/// # Arguments
/// 
/// * 'schedule_path' - full path to the schedule from mygrid
pub async fn get_schedule(schedule_path: &str) -> Result<Vec<Block>, MyGridError> {
    let json = tokio::fs::read_to_string(schedule_path).await?;
    
    let blocks: Vec<Block> = serde_json::from_str(&json)?;
    Ok(blocks)
}

/// Reads base data from mygrid and returns a `BaseData` struct
/// 
/// # Arguments
/// 
/// * 'base_data_path' - full path to the base data file from mygrid
pub async fn get_base_data(base_data_path: &str) -> Result<BaseData, MyGridError> {
    let json = tokio::fs::read_to_string(base_data_path).await?;
    
    let base_data: BaseData = serde_json::from_str(&json)?;
    Ok(base_data)
}