# Hostel Manager

A hostel manager canister developed for the Dacade Internet Computer Rust Smart Contract 101 Challenge

To view and test this project canister, Click [here](https://dashboard.internetcomputer.org/canister/ys34e-baaaa-aaaal-adeua-cai)

## Running the project locally
You must have:
- [DFX SDK](https://internetcomputer.org/docs/current/developer-docs/build/install-upgrade-remove)
- Rust and Cargo



If you want to test your project locally, you can use the following commands:
```bash
# CLone the repo
git clone https://github.com/dacade-icp-rust-challenge

# Change current directory
cd dacade-icp-rust-challenge/

# Starts the replica, running in the background
dfx start --background --clean

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

If you want to participate in this challenge check the  [link](https://dacade.org/communities/icp/courses/rust-smart-contract-101/)