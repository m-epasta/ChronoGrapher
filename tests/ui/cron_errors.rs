use chronographer::prelude::*;

fn main() {
    // Too few fields
    cron!("* * * * *");
    
    // Too many fields
    cron!("* * * * * * *");
    
    // Invalid range
    cron!("60 * * * * *");
    
    // Invalid range (month)
    cron!("* * * * 13 *");
    
    // Invalid range (day of month)
    cron!("* * * 32 * *");
    
    // Invalid range (day of week)
    cron!("* * * * * 8");
    
    // Invalid range (hour)
    cron!("* * 24 * * *");
    
    // Invalid names
    cron!("* * * * * BLA");
    
    // Invalid names (month)
    cron!("* * * * BLU *");
    
    // Invalid step
    cron!("*/0 * * * * *");
    
    // Invalid range bounds
    cron!("10-5 * * * * *");
    
    // Invalid characters
    cron!("& * * * * *");
    
    // Missing number after slash
    cron!("*/ * * * * *");
    
    // Missing number after minus
    cron!("1- * * * * *");
}
