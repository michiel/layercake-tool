use crate::plan::Plan;
use anyhow::Result;

pub fn execute_plan(plan: Plan) -> Result<()> {
    println!("Executing plan: {:?}", plan);

    plan.import.profiles.iter().for_each(|profile| {
        println!("Importing file: {}", profile.filename);
    });

    plan.export.profiles.iter().for_each(|profile| {
        println!("Exporting file: {}", profile.filename);
    });

    Ok(())
}
