#![allow(dead_code)]

use {
    anchor_lang::{
        prelude::Pubkey, solana_program::instruction::Instruction, system_program, InstructionData,
        ToAccountMetas,
    },
    anchor_spl::{associated_token, token},
    litesvm::{types::TransactionResult, LiteSVM},
    mpl_core::{Asset, Collection},
    q3_nft_staking::{accounts, instruction},
    solana_clock::Clock,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

pub const DAY_START: i64 = 1_767_225_600;
pub const HOUR: i64 = 3_600;
pub const DAY: i64 = 86_400;
pub const REWARDS_PER_DAY: u64 = 1_000_000;
pub const BURN_BONUS: u64 = 100_000_000;

pub struct Env {
    pub svm: LiteSVM,
    pub admin: Keypair,
    pub user: Keypair,
    pub collection: Keypair,
    pub config: Pubkey,
    pub rewards_mint: Pubkey,
    pub update_authority: Pubkey,
    pub user_ata: Pubkey,
}

pub fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &q3_nft_staking::id()).0
}

impl Env {
    pub fn new() -> Self {
        let mut svm = LiteSVM::new();
        svm.add_program(
            q3_nft_staking::id(),
            include_bytes!(concat!(
                env!("CARGO_TARGET_TMPDIR"),
                "/../deploy/q3_nft_staking.so"
            )),
        )
        .unwrap();
        svm.add_program(mpl_core::ID, include_bytes!("../fixtures/mpl_core.so"))
            .unwrap();

        let admin = Keypair::new();
        let user = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();
        svm.airdrop(&user.pubkey(), 10_000_000_000).unwrap();

        let config = pda(&[q3_nft_staking::CONFIG_SEED]);
        let rewards_mint = pda(&[q3_nft_staking::REWARDS_MINT_SEED, config.as_ref()]);
        let update_authority = pda(&[q3_nft_staking::UPDATE_AUTHORITY_SEED]);
        let user_ata =
            associated_token::get_associated_token_address(&user.pubkey(), &rewards_mint);

        let mut env = Self {
            svm,
            admin,
            user,
            collection: Keypair::new(),
            config,
            rewards_mint,
            update_authority,
            user_ata,
        };
        env.set_time(DAY_START + 10 * HOUR);

        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::Initialize {
                rewards_per_day: REWARDS_PER_DAY,
                burn_bonus: BURN_BONUS,
            }
            .data(),
            accounts::Initialize {
                admin: env.admin.pubkey(),
                config,
                rewards_mint,
                system_program: system_program::ID,
                token_program: token::ID,
            }
            .to_account_metas(None),
        );
        env.send(ix, &[&env.admin.insecure_clone()]).unwrap();

        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::CreateCollection {
                name: "Stakers".into(),
                uri: "https://x.y/c".into(),
            }
            .data(),
            accounts::CreateCollection {
                admin: env.admin.pubkey(),
                config,
                collection: env.collection.pubkey(),
                update_authority,
                system_program: system_program::ID,
                core_program: mpl_core::ID,
            }
            .to_account_metas(None),
        );
        let (admin, collection) = (env.admin.insecure_clone(), env.collection.insecure_clone());
        env.send(ix, &[&admin, &collection]).unwrap();
        env
    }

    pub fn send(&mut self, ix: Instruction, signers: &[&Keypair]) -> TransactionResult {
        self.svm.expire_blockhash();
        let blockhash = self.svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&signers[0].pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
        self.svm.send_transaction(tx)
    }

    pub fn set_time(&mut self, unix_timestamp: i64) {
        let mut clock = self.svm.get_sysvar::<Clock>();
        clock.unix_timestamp = unix_timestamp;
        self.svm.set_sysvar(&clock);
    }

    pub fn now(&self) -> i64 {
        self.svm.get_sysvar::<Clock>().unix_timestamp
    }

    pub fn mint_nft(&mut self, owner: Pubkey) -> Keypair {
        let asset = Keypair::new();
        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::MintNft {
                name: "Staker #1".into(),
                uri: "https://x.y/a".into(),
            }
            .data(),
            accounts::MintNft {
                admin: self.admin.pubkey(),
                config: self.config,
                owner,
                asset: asset.pubkey(),
                collection: self.collection.pubkey(),
                update_authority: self.update_authority,
                system_program: system_program::ID,
                core_program: mpl_core::ID,
            }
            .to_account_metas(None),
        );
        let admin = self.admin.insecure_clone();
        self.send(ix, &[&admin, &asset]).unwrap();
        asset
    }

    pub fn stake(&mut self, asset: Pubkey) -> TransactionResult {
        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &instruction::Stake {}.data(),
            accounts::Stake {
                user: self.user.pubkey(),
                asset,
                collection: self.collection.pubkey(),
                update_authority: self.update_authority,
                system_program: system_program::ID,
                core_program: mpl_core::ID,
            }
            .to_account_metas(None),
        );
        let user = self.user.insecure_clone();
        self.send(ix, &[&user])
    }

    pub fn reward_ix_accounts(&self, asset: Pubkey) -> accounts::ClaimRewards {
        accounts::ClaimRewards {
            user: self.user.pubkey(),
            config: self.config,
            rewards_mint: self.rewards_mint,
            user_rewards_ata: self.user_ata,
            asset,
            collection: self.collection.pubkey(),
            update_authority: self.update_authority,
            system_program: system_program::ID,
            token_program: token::ID,
            associated_token_program: associated_token::ID,
            core_program: mpl_core::ID,
        }
    }

    pub fn send_reward_ix(&mut self, data: Vec<u8>, asset: Pubkey) -> TransactionResult {
        let ix = Instruction::new_with_bytes(
            q3_nft_staking::id(),
            &data,
            self.reward_ix_accounts(asset).to_account_metas(None),
        );
        let user = self.user.insecure_clone();
        self.send(ix, &[&user])
    }

    pub fn claim(&mut self, asset: Pubkey) -> TransactionResult {
        self.send_reward_ix(instruction::ClaimRewards {}.data(), asset)
    }

    pub fn unstake(&mut self, asset: Pubkey) -> TransactionResult {
        self.send_reward_ix(instruction::Unstake {}.data(), asset)
    }

    pub fn burn(&mut self, asset: Pubkey) -> TransactionResult {
        self.send_reward_ix(instruction::BurnStakedNft {}.data(), asset)
    }

    pub fn asset(&self, asset: Pubkey) -> Box<Asset> {
        Asset::from_bytes(&self.svm.get_account(&asset).unwrap().data).unwrap()
    }

    pub fn total_staked(&self) -> u64 {
        let data = self
            .svm
            .get_account(&self.collection.pubkey())
            .unwrap()
            .data;
        Collection::from_bytes(&data)
            .unwrap()
            .plugin_list
            .attributes
            .unwrap()
            .attributes
            .attribute_list
            .iter()
            .find(|a| a.key == q3_nft_staking::TOTAL_STAKED_KEY)
            .unwrap()
            .value
            .parse()
            .unwrap()
    }

    pub fn rewards_balance(&self) -> u64 {
        match self.svm.get_account(&self.user_ata) {
            Some(account) => u64::from_le_bytes(account.data[64..72].try_into().unwrap()),
            None => 0,
        }
    }
}

pub fn is_frozen(asset: &Asset) -> bool {
    asset
        .plugin_list
        .freeze_delegate
        .as_ref()
        .is_some_and(|p| p.freeze_delegate.frozen)
}
