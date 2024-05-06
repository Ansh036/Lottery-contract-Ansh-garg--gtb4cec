 
 

#![no_std]

use core::ops::Add;

use soroban_sdk::storage::Persistent;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, map, panic_with_error, token, vec,
    Address, Env, Map, Symbol, Vec
};

use shared::rand::*;


#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq)]
enum LotteryState {
    Initialized = 1,
    Active = 2,
    Finished = 3,
}

 
#[derive(Clone, Copy)]
#[contracttype]
enum DataKey {
    Admin = 1,
    Tickets = 2,
    TicketPrice = 4,
    LotteryNumber = 5,
    LotteryResults = 6,
    NumberOfNumbers = 7,
    MaxRange = 8,
    Thresholds = 9,
    MinPlayersCount = 10,
    Token = 11,
    LotteryState = 12,
}

 
#[contracterror]
#[derive(Clone, Debug, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
   
    AlreadyInitialized = 1,
   
    InsufficientFunds = 2,
 
    NotInitialized = 3, 
    MinParticipantsNotSatisfied = 4,
  
    MaxRangeTooLow = 5,
    
    NumberOfNumbersTooLow = 6,
    
    NumberOfThresholdsTooLow = 7,
     
    NotEnoughOrTooManyNumbers = 8,
   
    InvalidNumbers = 9,
     
    WrongLotteryNumber = 10, 
    NoLotteryResultsAvailable = 11,
     
    AlreadyActive = 12,
    
    NotActive = 13,
    
    InvalidThresholds = 14,
     
    InvalidTicketPrice = 15
}

 
type LotteryTicket = Vec<u32>;
type LotteryResult = Vec<u32>;

#[contract]
pub struct LotteryContract;

#[contractimpl]
impl LotteryContract {
    
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        env: Env,
        admin: Address,
        token: Address,
        ticket_price: i128,
        number_of_numbers: u32,
        max_range: u32,
        thresholds: Map<u32, u32>,
        min_players_count: u32,
    ) {
        admin.require_auth();
        let storage = env.storage().persistent();

        if storage
            .get::<_, LotteryState>(&DataKey::LotteryState)
            .is_some() {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        storage.set(&DataKey::Admin, &admin);
        storage.set(&DataKey::Token, &token);
        storage.set(&DataKey::LotteryState, &LotteryState::Initialized);
        Self::create_lottery(
            env,
            ticket_price,
            number_of_numbers,
            max_range,
            thresholds,
            min_players_count,
        );
    }

    
    pub fn create_lottery(
        env: Env,
        ticket_price: i128,
        number_of_numbers: u32,
        max_range: u32,
        thresholds: Map<u32, u32>,
        min_players_count: u32,
    ) -> u32 {
        let storage = env.storage().persistent();
        if storage
            .get::<_, LotteryState>(&DataKey::LotteryState)
            .is_none() {
            panic_with_error!(&env, Error::NotInitialized);
        }

        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let lottery_state = storage
            .get::<_, LotteryState>(&DataKey::LotteryState)
            .unwrap();
        if lottery_state == LotteryState::Active {
            panic_with_error!(&env, Error::AlreadyActive);
        }

        if max_range < number_of_numbers {
            panic_with_error!(&env, Error::MaxRangeTooLow);
        }

        if number_of_numbers < 2 {
            panic_with_error!(&env, Error::NumberOfNumbersTooLow);
        }

        if thresholds.is_empty() {
            panic_with_error!(&env, Error::NumberOfThresholdsTooLow);
        }

        if ticket_price <= 0 {
            panic_with_error!(&env, Error::InvalidTicketPrice);
        }

        let sum_of_percentages = thresholds.values()
            .iter()
            .fold(0u32, |acc, percentage| acc.add(percentage));

        if !(1..=100).contains(&sum_of_percentages) {
            panic_with_error!(&env, Error::InvalidThresholds);
        }

        for threshold_number in thresholds.keys() {
            if threshold_number < 1 || threshold_number > number_of_numbers {
                panic_with_error!(&env, Error::InvalidThresholds);
            }
        }

        let lottery_number = storage
            .get::<_, u32>(&DataKey::LotteryNumber)
            .unwrap_or_default()
            + 1;

        storage.set(&DataKey::NumberOfNumbers, &number_of_numbers);
        storage.set(&DataKey::MaxRange, &max_range);
        storage.set(&DataKey::Thresholds, &thresholds);
        storage.set(&DataKey::MinPlayersCount, &min_players_count);
        storage.set(&DataKey::TicketPrice, &ticket_price);
        storage.set(
            &DataKey::Tickets,
            &Map::<Address, Vec<LotteryTicket>>::new(&env),
        );
        storage.set(&DataKey::LotteryState, &LotteryState::Active);
        storage.set(&DataKey::LotteryNumber, &lottery_number);

        let topic = (Symbol::new(&env, "new_lottery_created"), lottery_number);
        env.events().publish(topic, (number_of_numbers, max_range, thresholds, ticket_price));

        lottery_number
    }

    
    pub fn register(_: Env, by: Address) {
        by.require_auth();
    }

     
    pub fn buy_ticket(env: Env, by: Address, ticket: Vec<u32>) -> Result<u32, Error> {
        by.require_auth();

        let storage = env.storage().persistent();

        lottery_must_be_active(&storage)?;

        let number_of_elements = storage.get::<_, u32>(&DataKey::NumberOfNumbers).unwrap();
        let max_range = storage.get::<_, u32>(&DataKey::MaxRange).unwrap();

        if ticket.len() != number_of_elements {
            return Err(Error::NotEnoughOrTooManyNumbers);
        }

         
        for number in ticket.iter() {
            if number == 0 || number > max_range {
                return Err(Error::InvalidNumbers);
            }
        }

        let price = storage.get::<_, i128>(&DataKey::TicketPrice).unwrap();
        let token = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        if token_client.balance(&by) <= price {
            return Err(Error::InsufficientFunds);
        }

        token_client.transfer(&by, &env.current_contract_address(), &price);

        let mut tickets = storage
            .get::<_, Map<Address, Vec<LotteryTicket>>>(&DataKey::Tickets)
            .unwrap();

        let mut player_selection = tickets.get(by.clone()).unwrap_or(vec![&env]);
        player_selection.push_back(ticket);
        tickets.set(by, player_selection);

        storage.set(&DataKey::Tickets, &tickets);
        Ok(tickets.values().len())
    }

    
    pub fn pool_balance(env: Env) -> Result<i128, Error> {
        let storage = env.storage().persistent();
        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        Ok(token_client.balance(&env.current_contract_address()))
    }

    
    pub fn check_lottery_results(env: Env, lottery_number: u32) -> Result<Vec<u32>, Error> {
        let storage = env.storage().persistent();
        
        let lottery_results = storage
            .get::<_, Map<u32, LotteryResult>>(&DataKey::LotteryResults)
            .ok_or(Error::NoLotteryResultsAvailable)?;

        if !lottery_results.contains_key(lottery_number) {
            return Err(Error::WrongLotteryNumber);
        }
        Ok(lottery_results.get(lottery_number).unwrap())
    }

  
    pub fn play_lottery(env: Env, random_seed: u64) -> Result<(), Error> {
        let storage = env.storage().persistent();

        lottery_must_be_active(&storage)?;

        let admin = storage.get::<_, Address>(&DataKey::Admin).unwrap();
        admin.require_auth();

        let token: Address = storage.get::<_, Address>(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token);

        let tickets = storage
            .get::<_, Map<Address, Vec<LotteryTicket>>>(&DataKey::Tickets)
            .unwrap();

        let min_players_count = storage.get::<_, u32>(&DataKey::MinPlayersCount).unwrap();

        if tickets.keys().len() < min_players_count {
            return Err(Error::MinParticipantsNotSatisfied);
        }

        let pool = token_client.balance(&env.current_contract_address());
        let max_range = storage.get::<_, u32>(&DataKey::MaxRange).unwrap();
        let number_of_elements = storage.get::<_, u32>(&DataKey::NumberOfNumbers).unwrap();
        let mut thresholds = storage
            .get::<_, Map<u32, u32>>(&DataKey::Thresholds)
            .unwrap();

        let drawn_numbers = draw_numbers::<RandomNumberGenerator>(&env, max_range, number_of_elements, random_seed);
        let winners = get_winners(&env, &drawn_numbers, &tickets, &thresholds);
        let prizes = calculate_prizes(&env, &winners, &mut thresholds, pool);
        payout_prizes(&env, &token_client, &prizes);

        // store numbers drawn in this lottery
        let lottery_number = storage.get::<_, u32>(&DataKey::LotteryNumber).unwrap();
        let mut lottery_results = storage
            .get::<_, Map<u32, LotteryResult>>(&DataKey::LotteryResults)
            .unwrap_or(map![&env]);

        lottery_results.set(lottery_number, drawn_numbers);
        storage.set(&DataKey::LotteryResults, &lottery_results);

        storage.set(&DataKey::LotteryState, &LotteryState::Finished);

        // emit events with won prizes
        prizes.iter().for_each(|(address, prize)| {
            let topic = (Symbol::new(&env, "won_prize"), &address);
            env.events().publish(topic, prize);
        });
        Ok(())
    }
}

 
fn  lottery_must_be_active(storage: &Persistent) -> Result<(), Error> {
    let lottery_state_opt = storage
        .get::<_, LotteryState>(&DataKey::LotteryState);

    if lottery_state_opt.is_none() {
        return Err(Error::NotInitialized);
    }

    if lottery_state_opt.unwrap() != LotteryState::Active {
        return Err(Error::NotActive);
    }
    Ok(())
}

 
fn get_winners(
    env: &Env,
    drawn_numbers: &Vec<u32>,
    tickets: &Map<Address, Vec<LotteryTicket>>,
    thresholds: &Map<u32, u32>,
) -> Map<u32, Vec<Address>> {
    let mut winners = Map::<u32, Vec<Address>>::new(env);

    tickets
        .iter()
        .for_each(|(ticket_address, tickets)|
            tickets
                .iter()
                .map(|ticket| count_matches(drawn_numbers, &ticket))
                .filter(|count, | thresholds.contains_key(*count))
                .for_each(|count| {
                    let mut addresses = winners.get(count).unwrap_or(Vec::<Address>::new(env));
                    addresses.push_back(ticket_address.clone());
                    winners.set(count, addresses);
                })
        );
    winners
}

 
fn calculate_prizes(
    env: &Env,
    winners: &Map<u32, Vec<Address>>,
    thresholds: &mut Map<u32, u32>,
    pool: i128,
) -> Map<Address, i128> {
    let mut prizes = Map::<Address, i128>::new(env);

   
    let total_prizes_percentage = count_total_prizes_percentage(winners, thresholds);
    recalculate_new_thresholds(winners, thresholds, total_prizes_percentage);

    
    winners.iter().for_each(|(threshold_number, addresses)| {
        let pool_percentage = thresholds.get(threshold_number).unwrap();
        let prize = pool * pool_percentage as i128 / 100i128;
        addresses.iter().for_each(|address| {
            let current_player_prize = prizes.get(address.clone()).unwrap_or_default();
            prizes.set(address, current_player_prize + prize)
        });
    });
    prizes
}

 
fn payout_prizes(env: &Env, token_client: &token::Client, prizes: &Map<Address, i128>) {
    prizes.iter().for_each(|(address, prize)| {
        token_client.transfer(&env.current_contract_address(), &address, &prize);
    });
}

 
fn count_total_prizes_percentage(
    winners: &Map<u32, Vec<Address>>,
    thresholds: &Map<u32, u32>,
) -> u32 {
    winners
        .iter()
        .fold(0u32, |acc, (threshold_number, _)| {
            let threshold_percentage = thresholds.get(threshold_number).unwrap();
            let winners_count = winners.get(threshold_number).unwrap().len();
            acc.add(threshold_percentage * winners_count)
        })
}

 
fn recalculate_new_thresholds(
    winners: &Map<u32, Vec<Address>>,
    thresholds: &mut Map<u32, u32>,
    total_prizes_percentage: u32,
) {
    if total_prizes_percentage > 100 {
        for threshold_number in thresholds.keys() {
            if winners.contains_key(threshold_number) {
                let winners_count = winners.get(threshold_number).unwrap().len();
                let threshold_percentage = thresholds.get(threshold_number).unwrap();
                let val =
                    winners_count * threshold_percentage * 100 / total_prizes_percentage;
                thresholds.set(threshold_number, val / winners_count);
            } else {
                thresholds.remove(threshold_number);
            }
        }
    }
}

 
fn count_matches(drawn_numbers: &Vec<u32>, player_ticket: &LotteryTicket) -> u32 {
    drawn_numbers
        .iter()
        .filter(|x| player_ticket.contains(x))
        .count() as u32
}


 
 
fn draw_numbers<T: RandomNumberGeneratorTrait>(env: &Env, max_range: u32, number_of_numbers: u32, random_seed: u64) -> Vec<u32> {
    let mut numbers = Vec::new(env);
    for n in 0..number_of_numbers {
        let new_seed = random_seed + n as u64;
        let mut random_generator = T::new(env, new_seed);
        loop {
             
            let drawn_number = random_generator.number(env, max_range);
            if !numbers.contains(drawn_number) {
                numbers.push_back(drawn_number);
                break;
            }
        }
    }
    numbers
}

#[cfg(test)]
mod test;
