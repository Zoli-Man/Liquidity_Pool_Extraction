use csv::Reader;
use csv::WriterBuilder;
use ethers::prelude::abigen;
use ethers::{
    providers::{Http, Provider},
    types::Address,
};
use eyre::Result;
use serde::Serialize;
use std::fs::File;
use std::fs::OpenOptions;
use std::str::FromStr;
use std::sync::Arc;

use dotenv::dotenv;
use std::env;
abigen!(univ2_contract, r"src\contract\univ2.json");
/// Struct to represent Liquidity Pool (LP) information.
#[derive(Debug, Serialize)]
struct LpId {
    id: u64,
    block: i32,
    address: Address,
    token0: Address,
    token1: Address,
}

#[tokio::main]
#[tracing::instrument]
/// Main function that handles errors due to a bad connection to the Ethereum server.
/// If an error is encountered, `data_extraction` will reset and start from the last retrieved LP.

async fn main() {
    loop {
        let result1 = data_extraction().await;
        match result1 {
            Ok(_) => {
                println!("all good, all data was extracted");
                break;
            }
            Err(err) => {
                println!("Error while running univ2: {}", err);
                // try to rerun DataExtract.
            }
        }
    }
}
#[tracing::instrument]

/// Main logic for fetching LP of Uniswap from Ethereum network.
async fn data_extraction() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let univ2_address = "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f";

    // ! get a connection to a ETH node and create a contract instant of uniswap 2 factory

    // take RPC_URL for .env or use default
    let rpc_url: &str =
        &env::var("RPC_URL").unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());
    let provider: Provider<Http> = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider.clone());

    let contract: univ2_contract<Provider<Http>> =
        univ2_contract::new(Address::from_str(univ2_address).unwrap(), client);
    let all_univ2_pairs = contract.all_pairs_length().call().await?.as_u128();
    let mut found_lps = 0;
    let mut start_block = 0;

    // ! csv settings
    // * recover the last found lp
    // Open the CSV file for reading

    let file = File::open(r"csvs\univ2lps.csv")?;
    let mut rdr = Reader::from_reader(file);
    // get the last record to recover work
    for result in rdr.records() {
        let record = result?;
        found_lps = record.get(0).unwrap().parse::<u128>()?;
        start_block = record.get(1).unwrap().parse::<i32>()? + 1;
    }
    // * open the csv for append writing
    let file2 = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(r"csvs\univ2lps.csv")?;
    let mut writer = WriterBuilder::new().has_headers(false).from_writer(file2);

    // ! start main loop
    let step = 1000;
    while found_lps < all_univ2_pairs {
        let num_of_lps_in_interval =
            how_many_lps_in_interval(start_block, start_block + step, contract.clone()).await?;
        if num_of_lps_in_interval == 0 {
            // no new lps
            start_block += step;
            continue;
        } else {
            // in this block we know there are X number of LPs in the interval
            for _i in 0..num_of_lps_in_interval {
                // find the creation_block number of the LP
                let creation_block =
                    find_lp(start_block, start_block + step, contract.clone()).await?;
                // looking for more LPs in the same creation_block number
                let lp_events = contract
                    .pair_created_filter()
                    .from_block(creation_block)
                    .to_block(creation_block)
                    .query()
                    .await?;
                // save the founded LPs to the CSV
                for eve in lp_events {
                    // Serialize the struct to a CSV string
                    writer.serialize(LpId {
                        id: eve.p3.low_u64() - 1,
                        block: creation_block,
                        address: eve.pair,
                        token0: eve.token_0,
                        token1: eve.token_1,
                    })?;
                    found_lps += 1;
                }
                start_block = creation_block + 1;
            }
            writer.flush()?;
            println!("flushed");
        }
    }
    Ok(())
}
/// Helper function to count the number of LPs in a given interval, including edges [start,end]
/// *Arguments
/// start: i32, the start of the interval
/// end: i32, the end of the interval
/// contract: univ2_contract<Provider<Http>>, the uniswap2 contract
///
/// *Result
/// Result<u32> , the number of LPs in the interval
async fn how_many_lps_in_interval(
    start: i32,
    end: i32,
    contract: univ2_contract<Provider<Http>>,
) -> Result<u32> {
    let events = contract
        .pair_created_filter()
        .from_block(start)
        .to_block(end)
        .query()
        .await?;
    Ok(events.len().try_into().unwrap())
}

/// Helper function to find the first LP's creation block within a given interval using binary search
/// *Arguments
/// start: i32, the start of the interval
/// end: i32, the end of the interval
/// contract: univ2_contract<Provider<Http>>, the uniswap2 contract
///
/// *Result
/// Result<u32> , the first LP's creation block
async fn find_lp(
    mut start: i32,
    mut end: i32,
    contract: univ2_contract<Provider<Http>>,
) -> Result<i32> {
    let mut mid = (start + end) / 2;
    loop {
        //look at the right half: [start,mid]
        let num_of_lps = how_many_lps_in_interval(start, mid, contract.clone()).await?;

        if num_of_lps == 0 {
            // go to left side: [mid,end]
            start = mid;
            mid = (start + end) / 2;
        } else {
            //go to right side: [start,mid]
            end = mid;
            mid = (start + end) / 2;
        }

        if start == mid {
            // the answer is start or start+1
            //cheek if answer is start
            if how_many_lps_in_interval(start, start, contract.clone()).await? == 0 {
                start += 1;
            }
            break;
        }
    }
    Ok(start)
}
