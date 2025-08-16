use ic_cdk::{update, query};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use candid::{CandidType, Deserialize, Serialize, Nat, Principal , Decode};


type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static IDEAS: RefCell<StableBTreeMap<u64, Idea, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|mm| mm.borrow_mut().get(MemoryId::new(0))), "ideas")
    );

}

// idea trait 
#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Idea {
    title: String,
    description: String,
    funding_goal: u64,
    current_funding: u64,
    legal_entity: String,
    status: Option<String>, // e.g., "pending", "approved", "rejected"
    contact_info: String,
    category: String, // e.g., "technology", "healthcare", "education"
    business_registration:u8,
    created_at: u64, // timestamp
    updated_at: u64, // timestamp
}

impl Storable for Idea {
        fn to_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn into_bytes(self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}



#[update]fn create_idea(
    title: String,
    description: String,
    funding_goal: u64,
    legal_entity: String,
    contact_info: String,
    category: String,
    business_registration: u8,
) -> u64 {
    // Validate inputs
    if title.is_empty() || description.is_empty() || funding_goal == 0 || legal_entity.is_empty() || contact_info.is_empty() || category.is_empty() {
        ic_cdk::trap("Invalid input: All fields must be provided and funding goal must be greater than zero.");
    }
    let now = ic_cdk::api::time();
    let idea = Idea {
        title,
        description,
        funding_goal,
        current_funding: 0,
        legal_entity,
        status: "pending".to_string(),
        contact_info,
        category,
        business_registration,
        created_at: now,
        updated_at: now,
    };  

    let id = IDEAS.with(|ideas| {
        let mut ideas = ideas.borrow_mut();
        let id = ideas.len() as u64 + 1; // simple ID generation
        ideas.insert(id, idea);
        id
    });

    id
}



