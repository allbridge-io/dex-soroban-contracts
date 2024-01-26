#![no_main]

use libfuzzer_sys::fuzz_target;
use soroban_sdk::testutils::arbitrary::{arbitrary, fuzz_catch_panic, Arbitrary};
use soroban_sdk::Env;
use tests::contracts::pool::Direction;
use tests::utils::{TestingEnvConfig, TestingEnvironment};

#[derive(Arbitrary, Debug)]
pub enum SwapDirection {
    A2B,
    B2A,
}

#[derive(Arbitrary, Debug)]
pub enum UserID {
    Alice,
    Bob,
}

#[derive(Arbitrary, Debug)]
pub struct Swap {
    direction: SwapDirection,
    amount: u16,
    sender: UserID,
    recipient: UserID,
}

impl Into<Direction> for SwapDirection {
    fn into(self) -> Direction {
        match self {
            SwapDirection::A2B => Direction::A2B,
            SwapDirection::B2A => Direction::B2A,
        }
    }
}

#[derive(Arbitrary, Debug)]
struct Input {
    deposit_amount: (u16, u16),
    swaps: [Swap; 5],
}

fuzz_target!(|input: Input| {
    let env = Env::default();
    let testing_env = TestingEnvironment::create(
        &env,
        TestingEnvConfig::default()
            .with_yaro_admin_deposit(410_000.0)
            .with_yusd_admin_deposit(440_000.0),
    );

    let TestingEnvironment {
        ref pool,
        ref alice,
        ref bob,
        ref yusd_token,
        ref yaro_token,
        ..
    } = testing_env;

    yusd_token.airdrop_amount(alice.as_ref(), f64::MAX);
    yaro_token.airdrop_amount(alice.as_ref(), f64::MAX);

    let get_user = |user_id: &UserID| match user_id {
        UserID::Alice => alice,
        UserID::Bob => bob,
    };

    let _ = fuzz_catch_panic(|| {
        let _ = pool
            .deposit(
                alice,
                (input.deposit_amount.0 as f64, input.deposit_amount.1 as f64),
                0.0,
            )
            .unwrap();

        for swap in input.swaps {
            let sender = get_user(&swap.sender);
            let recipient = get_user(&swap.recipient);

            let amount = swap.amount as f64;
            let receive_amount_min = 0.0;

            pool.swap(
                sender,
                recipient,
                amount,
                receive_amount_min,
                swap.direction.into(),
            )
            .unwrap();
        }

        pool.assert_total_lp_less_or_equal_d();
    });
});
