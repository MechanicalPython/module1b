# JHUB module 1b - interactive website. 

## How to build. 
1. Download rust from https://www.rust-lang.org/learn/get-started
2. `git clone git@github.com:MechanicalPython/JHUB.git`
3. `cd ./module1b`
4. Optional: Generate and download NASA API key from https://api.nasa.gov and copy/paste it into a file named `api_key`, 
with no whitespace. If no key is generated, then the rate limit for the demo key is 30 requests per hour, 50 per day. 
5. Build project with `cargo build`
6. Run project with `cargo run`
7. Go to http://127.0.0.1:8080 to interact with the website. 

## Source API
This project uses the NASA Near Earth Object Web Service, found here: https://api.nasa.gov 

## Definitions.
NEO_lookup -> details of a NEO. Derived from API 
NEO_feed -> list of NEOs from a date range. 

