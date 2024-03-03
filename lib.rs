#[ic_cdk::query]
fn greet(climacare: String) -> String {
    format!("Hello, {}!", climacare)
}
fn main() {
    let result = greet("Climacare".to_string());
    println!("{}", result);
}
// Data structure that defines climacare
pub struct Climacare {
    website_name: String,
    affected_cities: Vec<String>,
    issue_cause: String,
}
// Implementation for Climacare
impl Climacare {
// New function to create Climacare instances
pub fn new() -> Self {
    Climacare {
        website_name: "Climacare".to_string(),
        affected_cities: Vec::new(),
        issue_cause: "Climate problems caused by rockets damaging the ozone layer.".to_string(),
    }
}
// Function to add affected cities
pub fn add_affected_city(&mut self, city: &str) {
    self.affected_cities.push(city.to_string());
}
// Function to get information about Climacare
pub fn get_info(&self) -> String {
    format!(
        "{} is a website that is at the center of climate problems in the cities of {jakarta, Surabaya}. The problem is caused by the ozone layer {}"
        self.website_climacare, self.affected_cities, self.issue_cause
    )
}
}
// Query function to provide information about climacare
#[ic_cdk::query]
fn get_climacare_info() -> String {
// Creating a Climacare instance
let mut climacare = Climacare::new();
climacare.add_affected_city("Jakarta");
climacare.add_affected_city("Surabaya");
// Return information about climacare
climacare.get_info()
}

#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Message {
    id: u64,
    title: String,
    body: String,
    attachment_url: String,
    created_at: u64,
    updated_at: Option<u64>,
}
    // a trait that must be implemented for a struct that is stored in a stable struct
    impl Storable for Message {
        fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
            Cow::Owned(Encode!(self).unwrap())
        }
    
        fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
            Decode!(bytes.as_ref(), Self).unwrap()
        }
    }
    
    // another trait that must be implemented for a struct that is stored in a stable struct
    impl BoundedStorable for Message {
        const MAX_SIZE: u32 = 1024;
        const IS_FIXED_SIZE: bool = false;
    }
    thread_local! {
        static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
            MemoryManager::init(DefaultMemoryImpl::default())
        );
    
        static ID_COUNTER: RefCell<IdCell> = RefCell::new(
            IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
                .expect("Cannot create a counter")
        );
    
        static STORAGE: RefCell<StableBTreeMap<u64, Message, Memory>> =
            RefCell::new(StableBTreeMap::init(
                MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        ));
    }
    #[derive(candid::CandidType, Serialize, Deserialize, Default)]
    struct MessagePayload {
        title: String,
        body: String,
        attachment_url: String,
    }
    #[ic_cdk::query]
    fn get_message(id: u64) -> Result<Message, Error> {
        match _get_message(&id) {
            Some(message) => Ok(message),
            None => Err(Error::NotFound {
                msg: format!("a message with id={} not found", id),
            }),
        }
    }
    fn _get_message(id: &u64) -> Option<Message> {
        STORAGE.with(|s| s.borrow().get(id))
    }
    #[ic_cdk::update]
    fn add_message(message: MessagePayload) -> Option<Message> {
        let id = ID_COUNTER
            .with(|counter| {
                let current_value = *counter.borrow().get();
                counter.borrow_mut().set(current_value + 1)
            })
            .expect("cannot increment id counter");
        let message = Message {
            id,
            title: message.title,
            body: message.body,
            attachment_url: message.attachment_url,
            created_at: time(),
            updated_at: None,
        };
        do_insert(&message);
        Some(message)
    }
    // helper method to perform insert.
    fn do_insert(message: &Message) {
        STORAGE.with(|service| service.borrow_mut().insert(message.id, message.clone()));
    }
    #[ic_cdk::update]
    fn update_message(id: u64, payload: MessagePayload) -> Result<Message, Error> {
        match STORAGE.with(|service| service.borrow().get(&id)) {
            Some(mut message) => {
                message.attachment_url = payload.attachment_url;
                message.body = payload.body;
                message.title = payload.title;
                message.updated_at = Some(time());
                do_insert(&message);
                Ok(message)
            }
            None => Err(Error::NotFound {
                msg: format!(
                    "couldn't update a message with id={}. message not found",
                    id
                ),
            }),
        }
    }
    #[ic_cdk::update]
    fn delete_message(id: u64) -> Result<Message, Error> {
        match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
            Some(message) => Ok(message),
            None => Err(Error::NotFound {
                msg: format!(
                    "couldn't delete a message with id={}. message not found.",
                    id
                ),
            }),
        }
    }
    #[derive(candid::CandidType, Deserialize, Serialize)]
    enum Error {
        NotFound { msg: String },
    }
    // need this to generate candid
    ic_cdk::export_candid!();
