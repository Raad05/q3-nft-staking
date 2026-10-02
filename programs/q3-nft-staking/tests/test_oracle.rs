mod common;

use {
    anchor_lang::{prelude::Pubkey, solana_program::instruction::Instruction},
    anchor_lang::{system_program, AccountDeserialize, InstructionData, ToAccountMetas},
    common::*,
    q3_nft_staking::{
        accounts, instruction, ExternalValidationResult, OracleValidation, TransferOracle,
        CRANK_REWARD_LAMPORTS, ORACLE_SEED, ORACLE_VAULT_SEED,
    },
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const FEE: u64 = 5_000;

struct OracleEnv {
    env: Env,
    oracle: Pubkey,
    vault: Pubkey,
    cranker: Keypair,
}

impl OracleEnv {
    fn new(start: i64) -> Self {
        let mut env = Env::new();
        env.set_time(start);
        let oracle = pda(&[ORACLE_SEED]);
        let vault = pda(&[ORACLE_VAULT_SEED]);

        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::CreateOracle {
                vault_deposit: 1_000_000_000,
            }
            .data(),
            accounts::CreateOracle {
                admin: env.admin.pubkey(),
                config: env.config,
                oracle,
                vault,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        let admin = env.admin.insecure_clone();
        env.send(ix, &[&admin]).unwrap();

        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::AddOraclePlugin {}.data(),
            accounts::AddOraclePlugin {
                admin: env.admin.pubkey(),
                config: env.config,
                oracle,
                collection: env.collection.pubkey(),
                update_authority: env.update_authority,
                system_program: system_program::ID,
                core_program: mpl_core::ID,
            }
            .to_account_metas(None),
        );
        env.send(ix, &[&admin]).unwrap();

        let cranker = Keypair::new();
        env.svm.airdrop(&cranker.pubkey(), 1_000_000_000).unwrap();
        Self {
            env,
            oracle,
            vault,
            cranker,
        }
    }

    fn transfer_result(&self) -> ExternalValidationResult {
        let data = self.env.svm.get_account(&self.oracle).unwrap().data;
        match TransferOracle::try_deserialize(&mut data.as_slice())
            .unwrap()
            .validation
        {
            OracleValidation::V1 { transfer, .. } => transfer,
            OracleValidation::Uninitialized => panic!("oracle uninitialized"),
        }
    }

    fn crank(&mut self) -> litesvm::types::TransactionResult {
        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::CrankOracle {}.data(),
            accounts::CrankOracle {
                cranker: self.cranker.pubkey(),
                oracle: self.oracle,
                vault: self.vault,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        );
        let cranker = self.cranker.insecure_clone();
        self.env.send(ix, &[&cranker])
    }

    fn cranker_balance(&self) -> u64 {
        self.env.svm.get_balance(&self.cranker.pubkey()).unwrap()
    }

    fn transfer(&mut self, asset: Pubkey, new_owner: Pubkey) -> litesvm::types::TransactionResult {
        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::TransferNft {}.data(),
            accounts::TransferNft {
                owner: self.env.user.pubkey(),
                new_owner,
                asset,
                collection: self.env.collection.pubkey(),
                oracle: self.oracle,
                system_program: system_program::ID,
                core_program: mpl_core::ID,
            }
            .to_account_metas(None),
        );
        let user = self.env.user.insecure_clone();
        self.env.send(ix, &[&user])
    }
}

#[test]
fn test_oracle_blocks_transfer_outside_hours() {
    let mut o = OracleEnv::new(DAY_START + 20 * HOUR);
    assert_eq!(o.transfer_result(), ExternalValidationResult::Rejected);

    let asset = o.env.mint_nft(o.env.user.pubkey()).pubkey();
    let recipient = Keypair::new().pubkey();
    o.env.svm.airdrop(&recipient, 1_000_000_000).unwrap();
    assert!(
        o.transfer(asset, recipient).is_err(),
        "transfer at 20:00 must be rejected"
    );

    let err = o.crank().unwrap_err();
    assert!(format!("{err:?}").contains("OracleAlreadyUpToDate"));

    o.env.set_time(DAY_START + DAY + 9 * HOUR + 60);
    let before = o.cranker_balance();
    o.crank().unwrap();
    assert_eq!(o.transfer_result(), ExternalValidationResult::Pass);
    assert_eq!(o.cranker_balance(), before + CRANK_REWARD_LAMPORTS - FEE);

    o.transfer(asset, recipient).unwrap();
    assert_eq!(o.env.asset(asset).base.owner, recipient);
}

#[test]
fn test_crank_reward_only_near_boundary() {
    let mut o = OracleEnv::new(DAY_START + 12 * HOUR);
    assert_eq!(o.transfer_result(), ExternalValidationResult::Pass);
    assert!(o.crank().is_err(), "no change at noon");

    o.env.set_time(DAY_START + 17 * HOUR + 30 * 60);
    let before = o.cranker_balance();
    o.crank().unwrap();
    assert_eq!(o.transfer_result(), ExternalValidationResult::Rejected);
    assert_eq!(o.cranker_balance(), before - FEE);

    o.env.set_time(DAY_START + DAY + 9 * HOUR - 120);
    assert!(o.crank().is_err());

    o.env.set_time(DAY_START + DAY + 9 * HOUR);
    o.crank().unwrap();
    o.env.set_time(DAY_START + DAY + 17 * HOUR + 120);
    let before = o.cranker_balance();
    o.crank().unwrap();
    assert_eq!(o.transfer_result(), ExternalValidationResult::Rejected);
    assert_eq!(o.cranker_balance(), before + CRANK_REWARD_LAMPORTS - FEE);
}
