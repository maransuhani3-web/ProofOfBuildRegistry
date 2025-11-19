#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, Address, Env, String, Symbol,
};

// Structure to store scholarship application details
#[contracttype]
#[derive(Clone)]
pub struct Scholarship {
    pub app_id: u64,
    pub applicant: Address,
    pub title: String,
    pub descrip: String,
    pub amount_requested: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub is_approved: bool,
    pub is_distributed: bool,
    pub timestamp: u64,
}

// Structure to track DAO statistics
#[contracttype]
#[derive(Clone)]
pub struct DaoStats {
    pub total_applications: u64,
    pub approved_applications: u64,
    pub distributed_scholarships: u64,
    pub total_funds_distributed: u64,
}

// Storage keys
const APP_COUNT: Symbol = symbol_short!("APP_CNT");
const DAO_STATS: Symbol = symbol_short!("DAO_STAT");

// Mapping application_id to Scholarship
#[contracttype]
pub enum ScholarBook {
    Application(u64),
}

// Mapping to track if an address has voted on a specific application
#[contracttype]
pub enum VoteTracker {
    Vote(u64, Address), // (app_id, voter_address)
}

#[contract]
pub struct CryptoScholarHub;

#[contractimpl]
impl CryptoScholarHub {
    // Function for students to submit scholarship applications
    pub fn submit_application(
        env: Env,
        applicant: Address,
        title: String,
        descrip: String,
        amount_requested: u64,
    ) -> u64 {
        applicant.require_auth();

        let mut app_count: u64 = env.storage().instance().get(&APP_COUNT).unwrap_or(0);
        app_count += 1;

        let timestamp = env.ledger().timestamp();

        let new_application = Scholarship {
            app_id: app_count,
            applicant: applicant.clone(),
            title,
            descrip,
            amount_requested,
            votes_for: 0,
            votes_against: 0,
            is_approved: false,
            is_distributed: false,
            timestamp,
        };

        // Update DAO statistics
        let mut stats = Self::get_dao_stats(env.clone());
        stats.total_applications += 1;

        // Store the application and update counters
        env.storage()
            .instance()
            .set(&ScholarBook::Application(app_count), &new_application);
        env.storage().instance().set(&APP_COUNT, &app_count);
        env.storage().instance().set(&DAO_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Scholarship Application Submitted - ID: {}, Amount: {}",
            app_count,
            amount_requested
        );

        app_count
    }

    // Function for DAO members to vote on scholarship applications
    pub fn vote_on_application(env: Env, voter: Address, app_id: u64, vote_for: bool) {
        voter.require_auth();

        // Check if voter has already voted
        let vote_key = VoteTracker::Vote(app_id, voter.clone());
        let has_voted: bool = env.storage().instance().get(&vote_key).unwrap_or(false);

        if has_voted {
            log!(&env, "Voter has already voted on this application!");
            panic!("Already voted!");
        }

        let mut scholarship = Self::get_application(env.clone(), app_id);

        if scholarship.app_id == 0 {
            log!(&env, "Application not found!");
            panic!("Application not found!");
        }

        if scholarship.is_approved {
            log!(&env, "Application already approved!");
            panic!("Application already approved!");
        }

        // Record the vote
        if vote_for {
            scholarship.votes_for += 1;
        } else {
            scholarship.votes_against += 1;
        }

        // Check if application should be approved (simple majority: votes_for > votes_against and at least 3 votes)
        let total_votes = scholarship.votes_for + scholarship.votes_against;
        if total_votes >= 3 && scholarship.votes_for > scholarship.votes_against {
            scholarship.is_approved = true;

            // Update DAO statistics
            let mut stats = Self::get_dao_stats(env.clone());
            stats.approved_applications += 1;
            env.storage().instance().set(&DAO_STATS, &stats);

            log!(&env, "Application ID: {} has been APPROVED by DAO!", app_id);
        }

        // Mark that this voter has voted
        env.storage().instance().set(&vote_key, &true);
        env.storage()
            .instance()
            .set(&ScholarBook::Application(app_id), &scholarship);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Vote recorded - App ID: {}, Vote For: {}",
            app_id,
            vote_for
        );
    }

    // Function to distribute scholarship funds to approved applicants
    pub fn distribute_scholarship(env: Env, app_id: u64) {
        let mut scholarship = Self::get_application(env.clone(), app_id);

        if scholarship.app_id == 0 {
            log!(&env, "Application not found!");
            panic!("Application not found!");
        }

        if !scholarship.is_approved {
            log!(&env, "Application not approved by DAO!");
            panic!("Not approved!");
        }

        if scholarship.is_distributed {
            log!(&env, "Scholarship already distributed!");
            panic!("Already distributed!");
        }

        scholarship.is_distributed = true;

        // Update DAO statistics
        let mut stats = Self::get_dao_stats(env.clone());
        stats.distributed_scholarships += 1;
        stats.total_funds_distributed += scholarship.amount_requested;

        env.storage()
            .instance()
            .set(&ScholarBook::Application(app_id), &scholarship);
        env.storage().instance().set(&DAO_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "Scholarship Distributed - App ID: {}, Amount: {}",
            app_id,
            scholarship.amount_requested
        );
    }

    // Function to view a specific scholarship application
    pub fn get_application(env: Env, app_id: u64) -> Scholarship {
        let key = ScholarBook::Application(app_id);

        env.storage().instance().get(&key).unwrap_or(Scholarship {
            app_id: 0,
            applicant: Address::from_string(&String::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )),
            title: String::from_str(&env, "Not_Found"),
            descrip: String::from_str(&env, "Not_Found"),
            amount_requested: 0,
            votes_for: 0,
            votes_against: 0,
            is_approved: false,
            is_distributed: false,
            timestamp: 0,
        })
    }

    // Function to view DAO statistics
    pub fn get_dao_stats(env: Env) -> DaoStats {
        env.storage()
            .instance()
            .get(&DAO_STATS)
            .unwrap_or(DaoStats {
                total_applications: 0,
                approved_applications: 0,
                distributed_scholarships: 0,
                total_funds_distributed: 0,
            })
    }
}
