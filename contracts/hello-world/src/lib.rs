#![no_std]

use soroban_sdk::{
    auth, contract, contractimpl, contracttype, token, Address, Env, Map, Vec as SVec,
};

#[derive(Clone)]
#[contracttype]
pub enum ShieldFundError {
    InvalidAmount,
    Unauthorized,
    RequestNotFound,
    DeadlinePassed,
    AlreadyFunded,
    Invalidbeneficiary,
}

#[derive(Clone)]
#[contracttype]
pub struct FundRequest {
    pub beneficiary: Address,
    pub university_id: Vec<u8>,
    pub target_amount: i128,
    pub deadline: u64,
    pub funded_amount: i128,
    pub is_active: bool,
    pub donor_count: u32,
}

#[contract]
pub struct ShieldFundContract;

#[contractimpl]
impl ShieldFundContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&"admin".to_val(), &admin);
    }

    pub fn create_request(
        env: Env,
        caller: Address,
        beneficiary: Address,
        university_id: Vec<u8>,
        target_amount: i128,
        deadline_hours: u64,
    ) -> Address {
        // Generate a unique request ID based on beneficiary + timestamp
        let request_id = env
            .current_time()
            .unwrap()
            .wrapping_add(caller.to_val().to_val());

        let deadline = env.current_time().unwrap() + (deadline_hours * 3600 * 1000);

        let request = FundRequest {
            beneficiary,
            university_id,
            target_amount,
            deadline,
            funded_amount: 0,
            is_active: true,
            donor_count: 0,
        };

        let mut requests = env
            .storage()
            .instance()
            .get::<Address, Map<Address, FundRequest>>(&"requests".to_val())
            .unwrap_or_else(|| Map::new(&env));

        requests.set(request_id, request);
        env.storage()
            .instance()
            .set(&"requests".to_val(), &requests);

        request_id
    }

    pub fn contribute(
        env: Env,
        caller: Address,
        request_id: Address,
        amount: i128,
        token_address: Address,
    ) -> Result<(), ShieldFundError> {
        let mut requests = env
            .storage()
            .instance()
            .get::<Address, Map<Address, FundRequest>>(&"requests".to_val())
            .ok_or(ShieldFundError::RequestNotFound)?;

        let request = requests
            .get(&request_id)
            .ok_or(ShieldFundError::RequestNotFound)?;

        // Check deadline not passed
        if env.current_time().unwrap() > request.deadline {
            return Err(ShieldFundError::DeadlinePassed);
        }

        if !request.is_active {
            return Err(ShieldFundError::AlreadyFunded);
        }

        // Transfer USDC from donor to beneficiary
        // Note: In production, would use TokenClient for USDC transfer
        // This is simplified - actual impl would call token.transfer()
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&caller, &request.beneficiary, &amount);

        // Update request
        let mut updated_request = request;
        updated_request.funded_amount += amount;
        updated_request.donor_count += 1;

        // Check if fully funded
        if updated_request.funded_amount >= updated_request.target_amount {
            updated_request.is_active = false;
        }

        requests.set(request_id, updated_request);
        env.storage()
            .instance()
            .set(&"requests".to_val(), &requests);

        Ok(())
    }

    pub fn get_request(env: Env, request_id: Address) -> Option<FundRequest> {
        let requests = env
            .storage()
            .instance()
            .get::<Address, Map<Address, FundRequest>>(&"requests".to_val())?;
        requests.get(&request_id)
    }

    pub fn get_active_requests(env: Env) -> SVec<FundRequest> {
        let requests = env
            .storage()
            .instance()
            .get::<Address, Map<Address, FundRequest>>(&"requests".to_val())
            .unwrap_or_else(|| Map::new(&env));

        let mut active = SVec::new(&env);
        for (_, request) in requests.iter() {
            if request.is_active {
                active.push_back(request);
            }
        }
        active
    }

    pub fn cancel_request(env: Env, caller: Address, request_id: Address) -> Result<(), ShieldFundError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&"admin".to_val())
            .ok_or(ShieldFundError::Unauthorized)?;

        if caller != admin {
            return Err(ShieldFundError::Unauthorized);
        }

        let mut requests = env
            .storage()
            .instance()
            .get::<Address, Map<Address, FundRequest>>(&"requests".to_val())
            .ok_or(ShieldFundError::RequestNotFound)?;

        if let Some(mut request) = requests.get(&request_id) {
            request.is_active = false;
            requests.set(request_id, request);
            env.storage()
                .instance()
                .set(&"requests".to_val(), &requests);
        }

        Ok(())
    }
}

trait ToVal {
    fn to_val(&self) -> u64;
}

impl ToVal for Address {
    fn to_val(&self) -> u64 {
        // Simplified - in practice would use internal conversion
        12345
    }
}