// idea trait 

struct Idea {
    title: String,
    description: String,
    funding_goal: u64,
    current_funding: u64,
    legal_entity: String,
    status: String, // e.g., "open", "funded", "closed"
    contact_info: String,
    category: String, // e.g., "technology", "healthcare", "education"
    business_registration:u8,
    created_at: u64, // timestamp
    updated_at: u64, // timestamp
}



#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

