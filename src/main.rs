use clap::Parser;
use ethers::utils::{keccak256, hex};
use rand::Rng;
use std::io::{self, Write};
use rayon::prelude::*;
use std::sync::Mutex;
use num_cpus;

const PROXY_BYTECODE: &str = "0x67363d3d37363d34f03d5260086018f3";
lazy_static::lazy_static! {
    static ref PROXY_BYTECODE_HASH: [u8; 32] = keccak256(hex::decode(&PROXY_BYTECODE[2..]).unwrap());
}
const FACTORY_ADDRESS: &str = "0x93FEC2C00BfE902F733B57c5a6CeeD7CD1384AE1";

#[derive(Debug)]
pub struct FindSaltOptions {
    creator: String,
    starts_with: Option<String>,
    ends_with: Option<String>,
    silent: bool,
    max_attempts: u64,
    parallel: bool,
}

#[derive(Debug)]
pub struct SaltResult {
    salt: String,
    address: String,
}

fn get_deployed(salt: &[u8]) -> String {
    let mut packed = Vec::with_capacity(1 + 20 + 32 + 32);
    packed.extend_from_slice(&[0xff]);
    packed.extend_from_slice(&hex::decode(&FACTORY_ADDRESS[2..]).unwrap());
    packed.extend_from_slice(salt);
    packed.extend_from_slice(&PROXY_BYTECODE_HASH[..]);
    
    let encode1 = keccak256(packed);
    let proxy = format!("0x{}", hex::encode(&encode1[12..]));
    
    let mut packed2 = Vec::with_capacity(22);
    packed2.extend_from_slice(&[0xd6, 0x94]);
    packed2.extend_from_slice(&hex::decode(&proxy[2..]).unwrap());
    packed2.extend_from_slice(&[0x01]);
    
    let encoded2 = keccak256(packed2);
    format!("0x{}", hex::encode(&encoded2[12..]))
}

fn is_valid_address(address: &str, starts_with: &Option<String>, ends_with: &Option<String>) -> bool {
    if starts_with.is_none() && ends_with.is_none() {
        return true;
    }

    let address_lower = address.to_lowercase();
    match (starts_with, ends_with) {
        (Some(prefix), Some(suffix)) => {
            address_lower.starts_with(&prefix.to_lowercase()) && 
            address_lower.ends_with(&suffix.to_lowercase())
        },
        (Some(prefix), None) => address_lower.starts_with(&prefix.to_lowercase()),
        (None, Some(suffix)) => address_lower.ends_with(&suffix.to_lowercase()),
        (None, None) => true,
    }
}

pub fn find_salt(options: FindSaltOptions) -> Option<SaltResult> {
    if options.parallel {
        find_salt_parallel(options)
    } else {
        find_salt_sequential(options)
    }
}

pub fn find_salt_sequential(options: FindSaltOptions) -> Option<SaltResult> {
    let mut attempts = 0;
    let mut rng = rand::thread_rng();

    while attempts < options.max_attempts {
        attempts += 1;
        let salt: [u8; 32] = rng.gen();
        
        let mut packed = Vec::new();
        packed.extend_from_slice(&hex::decode(&options.creator[2..]).unwrap());
        packed.extend_from_slice(&salt);
        
        let hex_salt = keccak256(packed);
        let address = get_deployed(&hex_salt);

        if !options.silent {
            print!("\rAttempt {}: {}", attempts, address);
            io::stdout().flush().unwrap();
        }

        if is_valid_address(&address, &options.starts_with, &options.ends_with) {
            if !options.silent {
                println!("\nFound matching address!");
                println!("Salt: 0x{}", hex::encode(salt));
                println!("Address: {}", address);
                println!("Attempts: {}", attempts);
            }
            return Some(SaltResult {
                salt: format!("0x{}", hex::encode(salt)),
                address,
            });
        }
    }

    if !options.silent {
        println!("\nNo matching address found after {} attempts", attempts);
    }
    None
}

pub fn find_salt_parallel(options: FindSaltOptions) -> Option<SaltResult> {
    let chunk_size = 10000;
    let num_threads = num_cpus::get();
    let attempts_per_thread = options.max_attempts / num_threads as u64;
    let progress = Mutex::new(0u64);
    
    let creator_bytes = hex::decode(&options.creator[2..]).unwrap();

    (0..num_threads).into_par_iter()
        .find_map_any(|_| {
            let mut attempts = 0;
            let mut rng = rand::thread_rng();
            let mut packed = Vec::with_capacity(20 + 32);
            packed.extend_from_slice(&creator_bytes);
            packed.resize(packed.len() + 32, 0);

            while attempts < attempts_per_thread {
                for _ in 0..chunk_size {
                    rng.fill(&mut packed[creator_bytes.len()..]);
                    
                    let hex_salt = keccak256(&packed);
                    let address = get_deployed(&hex_salt);

                    attempts += 1;

                    if !options.silent {
                        let mut total = progress.lock().unwrap();
                        *total += 1;
                        if *total % 1000 == 0 {
                            print!("\rAttempt {}", total);
                            io::stdout().flush().unwrap();
                        }
                    }

                    if is_valid_address(&address, &options.starts_with, &options.ends_with) {
                        if !options.silent {
                            println!("\nFound matching address!");
                            println!("Salt: 0x{}", hex::encode(&packed[creator_bytes.len()..]));
                            println!("Address: {}", address);
                            println!("Attempts: {}", attempts);
                        }
                        return Some(SaltResult {
                            salt: format!("0x{}", hex::encode(&packed[creator_bytes.len()..])),
                            address,
                        });
                    }
                }
            }
            None
        })
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    creator: String,
    
    #[arg(short, long)]
    starts_with: Option<String>,
    
    #[arg(short, long)]
    ends_with: Option<String>,
    
    #[arg(short, long, default_value_t = u64::MAX)]
    max_attempts: u64,
    
    #[arg(long, default_value_t = false)]
    silent: bool,
    
    #[arg(short = 'p', long, default_value_t = false)]
    parallel: bool,
}

fn main() {
    let args = Args::parse();
    
    let starts_with = args.starts_with.map(|s| format!("0x{}", s));
    
    if let Some(result) = find_salt(FindSaltOptions {
        creator: args.creator,
        starts_with,
        ends_with: args.ends_with,
        max_attempts: args.max_attempts,
        silent: args.silent,
        parallel: args.parallel,
    }) {
        println!("Found result - Salt: {}, Address: {}", result.salt, result.address);
    }
}
