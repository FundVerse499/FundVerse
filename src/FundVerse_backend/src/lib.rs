// FundVerse_backend/src/lib.rs

//! FundVerse Backend: Ideas + Campaigns with a foreign-key relation (campaign.idea_id -> ideas)

use std::{borrow::Cow, cell::RefCell};

use candid::{CandidType, Decode, Encode, Deserialize};
use ic_cdk::{self};
use ic_cdk_macros::{init, query, update};

// ---- Stable storage (Ideas) ----
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, storable::Bound , Storable};
use std::collections::HashMap;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 2000;

// Global memory manager + stable map for ideas
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static DOCS: std::cell::RefCell<HashMap<u64, Doc>> = Default::default();
    static IDEA_COUNTER: std::cell::RefCell<u64> = std::cell::RefCell::new(0);
    static DOC_COUNTER: std::cell::RefCell<u64> = std::cell::RefCell::new(0);


    static IDEAS: RefCell<StableBTreeMap<u64, Idea, Memory>> = RefCell::new(
        // Use memory 0 for ideas map
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|mm| mm.borrow().get(MemoryId::new(0)))
        )
    );

    // In-heap vector for campaigns (simple MVP). You can move this to stable later if needed.
    static CAMPAIGNS: RefCell<Vec<Campaign>> = RefCell::new(Vec::new());
}

// ------------- Data Models -------------

#[derive(CandidType, Deserialize, serde::Serialize, Clone, Debug)]
pub struct Idea {
    pub title: String,
    pub description: String,
    pub funding_goal: u64,
    pub current_funding: u64,
    pub legal_entity: String,
    pub status: Option<String>, // e.g., "pending", "approved", "rejected"
    pub contact_info: String,
    pub category: String,       // e.g., "technology", "healthcare", "education"
    pub business_registration: u8,
    pub created_at: u64,        // ns since epoch
    pub updated_at: u64,        // ns since epoch
    pub doc_ids: Vec<u64>,      // IDs of uploaded documents
}

#[derive(CandidType, Deserialize, Clone)]
pub struct Doc {
    pub id: u64,
    pub idea_id: u64,       // which idea this belongs to
    pub name: String,       // original filename
    pub content_type: String, // e.g., "application/pdf"
    pub data: Vec<u8>,        // raw file bytes
    pub uploaded_at: u64,
}

// Store Idea in stable memory by encoding/decoding with candid.
impl Storable for Idea {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).expect("encode Idea"))
    }


    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).expect("decode Idea")
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Campaign {
    pub id: u64,
    pub idea_id: u64,      // 🔗 foreign key to Idea
    pub amount_raised: u64,
    pub goal: u64,
    pub end_date: u64,     // seconds since Unix epoch
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CampaignCard {
    pub id: u64,
    pub idea_id: u64,      // 🔗
    pub title: String,     // from Idea
    pub category: String,  // from Idea
    pub amount_raised: u64,
    pub goal: u64,
    pub end_date: u64,
    pub days_left: i64,    // negative => ended
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum CampaignStatus {
    Active,
    Ended,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct CampaignWithIdea {
    pub campaign: CampaignCard,
    pub idea: Idea,
}

// ------------- Helpers -------------

fn now_secs() -> u64 {
    // ic_cdk::api::time returns ns
    ic_cdk::api::time() / 1_000_000_000
}

fn to_card(c: &Campaign, idea: &Idea) -> CampaignCard {
    let now = now_secs() as i64;
    let days_left = ((c.end_date as i64) - now) / 86_400; // 86400 secs/day
    CampaignCard {
        id: c.id,
        idea_id: c.idea_id,
        title: idea.title.clone(),
        category: idea.category.clone(),
        amount_raised: c.amount_raised,
        goal: c.goal,
        end_date: c.end_date,
        days_left,
    }
}

fn get_idea(id: u64) -> Option<Idea> {
    IDEAS.with(|map| map.borrow().get(&id))
}

/// Upload a document for an Idea. Returns the new doc_id or None if idea doesn't exist.
#[update]
fn upload_doc(idea_id: u64, name: String, content_type: String, data: Vec<u8>, uploaded_at: u64) -> Option<u64> {
    if !IDEAS.with(|ideas| ideas.borrow().contains_key(&idea_id)) {
        return None; // idea doesn’t exist
    }

    DOC_COUNTER.with(|c| {
        let mut c = c.borrow_mut();
        *c += 1;
        let doc_id = *c;

        let doc = Doc {
            id: doc_id,
            idea_id,
            name,
            content_type,
            data,
            uploaded_at,
        };

        DOCS.with(|docs| docs.borrow_mut().insert(doc_id, doc));

        // attach to idea
        IDEAS.with(|ideas| {
            if let Some(mut idea) = ideas.borrow().get(&idea_id) {
                idea.doc_ids.push(doc_id);
                ideas.borrow_mut().insert(idea_id, idea);
            }
        });

        Some(doc_id)
    })
}




// ------------- Public API -------------

/// Create an Idea and persist it in stable storage. Returns the new idea_id.
#[update]
fn create_idea(
    title: String,
    description: String,
    funding_goal: u64,
    legal_entity: String,
    contact_info: String,
    category: String,
    business_registration: u8,
) -> u64 {
    if title.is_empty()
        || description.is_empty()
        || funding_goal == 0
        || legal_entity.is_empty()
        || contact_info.is_empty()
        || category.is_empty()
    {
        ic_cdk::trap(
            "Invalid input: all fields must be provided and funding_goal must be > 0.",
        );
    }

    let now = ic_cdk::api::time();
    let idea = Idea {
        title,
        description,
        funding_goal,
        current_funding: 0,
        legal_entity,
        status: Some("pending".to_string()),
        contact_info,
        doc_ids : vec![],
        category,
        business_registration,
        created_at: now,
        updated_at: now,
    };

    // naive id generation = len + 1 (OK for MVP)
    // consider a StableCell counter for production.
    IDEAS.with(|ideas| {
        let mut ideas = ideas.borrow_mut();
        let id = (ideas.len() as u64) + 1;
        ideas.insert(id, idea);
        id
    })
}

/// Create a Campaign linked to an existing Idea. Returns new campaign_id (Ok) or error (Err).
#[update]
fn create_campaign(idea_id: u64, goal: u64, end_date: u64) -> Result<u64, String> {
    if goal == 0 {
        return Err("goal must be > 0".into());
    }
    // ensure idea exists
    let Some(_idea) = get_idea(idea_id) else {
        return Err("idea_id not found".into());
    };

    let id = CAMPAIGNS.with(|store| {
        let mut vec = store.borrow_mut();
        let new_id = (vec.len() as u64) + 1;
        vec.push(Campaign {
            id: new_id,
            idea_id,
            amount_raised: 0,
            goal,
            end_date,
        });
        new_id
    });

    Ok(id)
}

/// Return all campaign cards (title/category pulled from linked Idea).
#[query]
fn get_campaign_cards() -> Vec<CampaignCard> {
    CAMPAIGNS.with(|store| {
        store
            .borrow()
            .iter()
            .filter_map(|c| get_idea(c.idea_id).map(|idea| to_card(c, &idea)))
            .collect()
    })
}

///return docs with idea_id
#[query]
fn get_doc(doc_id: u64) -> Option<Doc> {
    DOCS.with(|docs| docs.borrow().get(&doc_id).cloned())
}

/// Return cards filtered by status (Active/Ended).
#[query]
fn get_campaign_cards_by_status(status: CampaignStatus) -> Vec<CampaignCard> {
    let now = now_secs() as i64;
    CAMPAIGNS.with(|store| {
        store
            .borrow()
            .iter()
            .filter_map(|c| get_idea(c.idea_id).map(|idea| to_card(c, &idea)))
            .filter(|card| match status {
                CampaignStatus::Active => card.days_left >= 0 && (card.end_date as i64) >= now,
                CampaignStatus::Ended => card.days_left < 0 || (card.end_date as i64) < now,
            })
            .collect()
    })
}

/// Fetch a single campaign joined with its Idea.
#[query]
fn get_campaign_with_idea(campaign_id: u64) -> Option<CampaignWithIdea> {
    CAMPAIGNS.with(|store| {
        store
            .borrow()
            .iter()
            .find(|c| c.id == campaign_id)
            .and_then(|c| get_idea(c.idea_id).map(|idea| CampaignWithIdea {
                campaign: to_card(c, &idea),
                idea,
            }))
    })
}

/// Convenience: fetch an idea by id
#[query]
fn get_idea_by_id(idea_id: u64) -> Option<Idea> {
    get_idea(idea_id)
}

// ------------- Demo Seed -------------

#[init]
fn init() {
    let now = ic_cdk::api::time(); // nanoseconds since epoch

    // ---- Seed Idea 1 ----
    let idea1_id = create_idea(
        "Eco-Friendly Water Bottles".into(),
        "Reusable bottles made from recycled materials".into(),
        100_000,
        "EcoCorp LLC".into(),
        "contact@ecocorp.example".into(),
        "Environment".into(),
        1,
    );

    // ---- Seed Idea 2 ----
    let idea2_id = create_idea(
        "Indie Pixel Art Game".into(),
        "A cozy RPG with retro pixel art".into(),
        100_000,
        "IndieStudio Ltd".into(),
        "hello@indiestudio.example".into(),
        "Gaming".into(),
        1,
    );

    // ---- Seed Campaigns ----
    // Active (+7 days)
    let _ = create_campaign(idea1_id, 100_000, now + 7 * 86_400 * 1_000_000_000);

    // Ended (-5 days)
    let _ = create_campaign(idea2_id, 100_000, now - 5 * 86_400 * 1_000_000_000);
}


// Export Candid for tooling & UI integration
ic_cdk::export_candid!();
