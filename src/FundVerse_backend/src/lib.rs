// idea trait 
use candid::{CandidType, Deserialize};
use ic_cdk_macros::{init, query, update};


use ic_cdk::{self};
use std::cell::RefCell;

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


// ========== Data Models ==========
// ========== Data Models ==========
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Campaign {
    pub id: u64,
    pub title: String,
    pub category: String,
    pub amount_raised: u64, // in EGP smallest unit (for MVP just treat as integer)
    pub goal: u64,          // EGP
    pub end_date: u64,      // seconds since Unix epoch
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct CampaignCard {
    pub id: u64,
    pub title: String,
    pub category: String,
    pub amount_raised: u64,
    pub goal: u64,
    pub end_date: u64,
    pub days_left: i64, // negative => ended
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum CampaignStatus { Active, Ended }

thread_local! {
    static CAMPAIGNS: RefCell<Vec<Campaign>> = RefCell::new(Vec::new());
}

fn now_secs() -> u64 {
    // ic_cdk::api::time returns nanoseconds
    ic_cdk::api::time() / 1_000_000_000
}

fn to_card(c: &Campaign) -> CampaignCard {
    let now = now_secs() as i64;
    let days_left = ((c.end_date as i64) - now) / 86_400; // 86400 = secs per day
    CampaignCard {
        id: c.id,
        title: c.title.clone(),
        category: c.category.clone(),
        amount_raised: c.amount_raised,
        goal: c.goal,
        end_date: c.end_date,
        days_left,
    }
}

// ========== Queries ==========

#[query]
fn get_campaign_cards() -> Vec<CampaignCard> {
    CAMPAIGNS.with(|store| store.borrow().iter().map(to_card).collect())
}

#[query]
fn get_campaign_cards_by_status(status: CampaignStatus) -> Vec<CampaignCard> {
    let now = now_secs() as i64;
    CAMPAIGNS.with(|store| {
        store
            .borrow()
            .iter()
            .map(to_card)
            .filter(|card| match status {
                CampaignStatus::Active => card.days_left >= 0 && (card.end_date as i64) >= now,
                CampaignStatus::Ended => card.days_left < 0 || (card.end_date as i64) < now,
            })
            .collect()
    })
}

// ========== Demo Seed (optional) ==========

#[init]
fn init() {
    // Add two demo campaigns so the grid shows data on first deploy.
    CAMPAIGNS.with(|store| {
        let mut s = store.borrow_mut();
        if s.is_empty() {
            let now = now_secs();
            s.push(Campaign {
                id: 1,
                title: "Eco-Friendly Water Bottles".into(),
                category: "Environment".into(),
                amount_raised: 25_000,
                goal: 100_000,
                end_date: now + 7 * 86_400, // 7 days from now => Active
            });
            s.push(Campaign {
                id: 2,
                title: "Indie Pixel Art Game".into(),
                category: "Gaming".into(),
                amount_raised: 120_000,
                goal: 100_000,
                end_date: now - 5 * 86_400, // 5 days ago => Ended
            });
        }
    });
}

// Export Candid for tooling (optional if you maintain .did by hand)
ic_cdk::export_candid!();

