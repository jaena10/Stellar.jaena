#![cfg(test)]

mod test {
    use super::*;
    use soroban_sdk::testutils::Address as TestAddress;
    use soroban_sdk::{vec, Address, Env};

    fn create_address(id: u32) -> Address {
        Address::from_account_id(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, id as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0,
        ])
    }

    #[test]
    fn test_create_fund_request() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let beneficiary = create_address(2);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        let request_id = contract.create_request(
            &env,
            admin.clone(),
            beneficiary.clone(),
            b"CU2024001".to_vec(),
            5000_0000000, // 5000 USDC (7 decimal places)
            48, // 48 hours deadline
        );

        let request = contract.get_request(&env, request_id.clone()).unwrap();
        assert_eq!(request.beneficiary, beneficiary);
        assert_eq!(request.target_amount, 5000_0000000);
        assert_eq!(request.is_active, true);
        assert_eq!(request.donor_count, 0);
    }

    #[test]
    fn test_contribute_to_request() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let donor = create_address(3);
        let beneficiary = create_address(2);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        let request_id = contract.create_request(
            &env,
            admin.clone(),
            beneficiary.clone(),
            b"CU2024001".to_vec(),
            5000_0000000,
            48,
        );

        // Mock USDC token address
        let usdc_token = create_address(100);

        let result = contract.contribute(
            &env,
            donor.clone(),
            request_id.clone(),
            2500_0000000, // 2500 USDC
            usdc_token.clone(),
        );

        assert!(result.is_ok());

        let request = contract.get_request(&env, request_id.clone()).unwrap();
        assert_eq!(request.funded_amount, 2500_0000000);
        assert_eq!(request.donor_count, 1);
        assert_eq!(request.is_active, true); // Not fully funded yet
    }

    #[test]
    fn test_request_fully_funded() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let donor1 = create_address(3);
        let donor2 = create_address(4);
        let beneficiary = create_address(2);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        let request_id = contract.create_request(
            &env,
            admin.clone(),
            beneficiary.clone(),
            b"CU2024001".to_vec(),
            5000_0000000,
            48,
        );

        let usdc_token = create_address(100);

        // First contribution - 3000 USDC
        contract.contribute(&env, donor1.clone(), request_id.clone(), 3000_0000000, usdc_token.clone());

        // Second contribution - 2000 USDC (completes target)
        contract.contribute(&env, donor2.clone(), request_id.clone(), 2000_0000000, usdc_token.clone());

        let request = contract.get_request(&env, request_id.clone()).unwrap();
        assert_eq!(request.funded_amount, 5000_0000000);
        assert_eq!(request.is_active, false); // Should be marked inactive
    }

    #[test]
    fn test_deadline_passed_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let donor = create_address(3);
        let beneficiary = create_address(2);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        // Create request with 1 hour deadline
        let request_id = contract.create_request(
            &env,
            admin.clone(),
            beneficiary.clone(),
            b"CU2024001".to_vec(),
            5000_0000000,
            1,
        );

        // Fast-forward time by 2 hours (deadline passes)
        env.ledger().set_timestamp(7200);

        let usdc_token = create_address(100);
        let result = contract.contribute(
            &env,
            donor.clone(),
            request_id.clone(),
            1000_0000000,
            usdc_token.clone(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_get_active_requests() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        // Create 3 fund requests
        contract.create_request(
            &env,
            admin.clone(),
            create_address(2),
            b"UNI001".to_vec(),
            1000_0000000,
            24,
        );
        contract.create_request(
            &env,
            admin.clone(),
            create_address(3),
            b"UNI002".to_vec(),
            2000_0000000,
            24,
        );
        contract.create_request(
            &env,
            admin.clone(),
            create_address(4),
            b"UNI003".to_vec(),
            3000_0000000,
            24,
        );

        let active = contract.get_active_requests(&env);
        assert_eq!(active.len(), 3);
    }

    #[test]
    fn test_cancel_request_by_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        let request_id = contract.create_request(
            &env,
            admin.clone(),
            create_address(2),
            b"UNI001".to_vec(),
            1000_0000000,
            24,
        );

        let result = contract.cancel_request(&env, admin.clone(), request_id.clone());
        assert!(result.is_ok());

        let request = contract.get_request(&env, request_id.clone()).unwrap();
        assert_eq!(request.is_active, false);
    }

    #[test]
    fn test_cancel_request_by_non_admin_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = create_address(1);
        let non_admin = create_address(99);
        let contract = ShieldFundContract;

        contract.initialize(&env, admin.clone());

        let request_id = contract.create_request(
            &env,
            admin.clone(),
            create_address(2),
            b"UNI001".to_vec(),
            1000_0000000,
            24,
        );

        let result = contract.cancel_request(&env, non_admin.clone(), request_id.clone());
        assert!(result.is_err());
    }
}