# Data Extraction Project

## Overview

This project is designed to extract Liquidity Pool (LP) information from the Uniswap decentralized exchange on the Ethereum network. The extracted data includes details such as LP ID, creation block, LP address, and the addresses of the two tokens in the pool.

## Project Structure

- **Main Script (`main.rs`):** The main script orchestrates the data extraction process. It utilizes the Uniswap 2 contract to interact with the Ethereum network, fetch LP details, and store the information in a CSV file. The script runs in a loop, attempting to recover from errors and resume data extraction.

- **Uniswap Contract (`univ2_contract`):** This module defines the Uniswap 2 contract using the `abigen` macro. It includes methods for querying the Ethereum blockchain to retrieve information about LPs.

- **CSV Handling (`csv`):** The project uses the `csv` crate to read and write CSV files. The `univ2lps.csv` file is used to store information about extracted LPs.

- **Environment Variables (`dotenv`):** The project uses the `dotenv` crate to load environment variables. The Ethereum RPC URL is read from the environment or defaults to a predefined URL.

## Dependencies

- **ethers:** A library for interacting with Ethereum, used for contract interaction.
- **eyre:** A crate for concise error handling.
- **serde:** A data serialization library for converting structs to and from CSV.
- **tokio:** A runtime for writing asynchronous code.
- **dotenv:** A library for reading environment variables from a file.

## How to Run

1. Clone the repository: `git clone <repository_url>`
2. Navigate to the project directory: `cd <project_directory>`
3. Create a .env file and set the RPC_URL variable to your Ethereum node's RPC URL. you may choose not to set it, the script will use the default RPC URL.
4. Run the project: `cargo run`

The script will continuously extract LP information from Uniswap until all LPs have been processed.

## Configuration

- **RPC_URL:** The Ethereum node's RPC URL. Set this in the `.env` file.

## Troubleshooting

- If errors occur during execution, the script will attempt to recover and resume data extraction.
- Check the error messages for details on any encountered issues.

## Notes

- The project assumes the existence of a CSV file (`univ2lps.csv`) to store and recover the progress of data extraction.
