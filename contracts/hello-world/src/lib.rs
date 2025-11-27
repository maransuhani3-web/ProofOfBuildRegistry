#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, Env, Symbol, String, symbol_short};

const BUILD_REG: Symbol = symbol_short!("BLDREG");

#[contracttype]
#[derive(Clone)]
pub struct BuildRecord {
    pub build_id: u64,
    pub builder: Symbol,
    pub repo_url: String,
    pub description: String,
    pub submitted_at: u64,
    pub verified: bool,
}

#[contract]
pub struct ProofOfBuildRegistry;

#[contractimpl]
impl ProofOfBuildRegistry {
    // Register a new build proof
    pub fn register_build(env: Env, build_id: u64, builder: Symbol, repo_url: String, description: String) {
        let submitted_at = env.ledger().timestamp();
        let record = BuildRecord {
            build_id,
            builder,
            repo_url,
            description,
            submitted_at,
            verified: false,
        };
        env.storage().instance().set(&build_id, &record);
    }

    // Mark a build as verified (e.g., by an admin or judging contract)
    pub fn verify_build(env: Env, build_id: u64) {
        let mut record: BuildRecord = env.storage().instance().get(&build_id).unwrap();
        record.verified = true;
        env.storage().instance().set(&build_id, &record);
    }

    // Check if a build is verified
    pub fn is_build_verified(env: Env, build_id: u64) -> bool {
        let record: Option<BuildRecord> = env.storage().instance().get(&build_id);
        match record {
            Some(r) => r.verified,
            None => false,
        }
    }

    // Get full build record
    pub fn get_build(env: Env, build_id: u64) -> Option<BuildRecord> {
        env.storage().instance().get(&build_id)
    }
}
