mod common;

use common::*;
use solana_signer::Signer;

#[test]
fn test_stake_claim_unstake() {
    let mut env = Env::new();
    let asset = env.mint_nft(env.user.pubkey()).pubkey();
    assert_eq!(env.total_staked(), 0);

    env.stake(asset).unwrap();
    let staked = env.asset(asset);
    assert!(is_frozen(&staked));
    assert!(staked.plugin_list.burn_delegate.is_some());
    assert_eq!(env.total_staked(), 1);
    assert!(env.stake(asset).is_err(), "double stake must fail");

    env.set_time(env.now() + DAY);
    env.claim(asset).unwrap();
    assert_eq!(env.rewards_balance(), REWARDS_PER_DAY);
    assert!(is_frozen(&env.asset(asset)));
    assert_eq!(env.total_staked(), 1);

    let err = env.claim(asset).unwrap_err();
    assert!(format!("{err:?}").contains("NothingToClaim"));

    env.set_time(env.now() + DAY / 2);
    env.unstake(asset).unwrap();
    assert_eq!(env.rewards_balance(), REWARDS_PER_DAY + REWARDS_PER_DAY / 2);
    let unstaked = env.asset(asset);
    assert!(unstaked.plugin_list.freeze_delegate.is_none());
    assert!(unstaked.plugin_list.burn_delegate.is_none());
    assert_eq!(env.total_staked(), 0);
    assert!(env.claim(asset).is_err(), "claim on unstaked NFT must fail");

    env.stake(asset).unwrap();
    assert_eq!(env.total_staked(), 1);
}

#[test]
fn test_total_staked_tracks_multiple_assets() {
    let mut env = Env::new();
    let a = env.mint_nft(env.user.pubkey()).pubkey();
    let b = env.mint_nft(env.user.pubkey()).pubkey();
    env.stake(a).unwrap();
    env.stake(b).unwrap();
    assert_eq!(env.total_staked(), 2);
    env.unstake(a).unwrap();
    assert_eq!(env.total_staked(), 1);
}

#[test]
fn test_burn_staked_nft() {
    let mut env = Env::new();
    let asset = env.mint_nft(env.user.pubkey()).pubkey();

    assert!(
        env.burn(asset).is_err(),
        "burning an unstaked NFT must fail"
    );

    env.stake(asset).unwrap();
    env.set_time(env.now() + DAY);
    env.burn(asset).unwrap();

    assert_eq!(env.rewards_balance(), REWARDS_PER_DAY + BURN_BONUS);
    assert_eq!(env.total_staked(), 0);
    let burned = env.svm.get_account(&asset);
    assert!(
        burned.is_none_or(|a| a.data.len() <= 1),
        "asset should be burned"
    );
}
