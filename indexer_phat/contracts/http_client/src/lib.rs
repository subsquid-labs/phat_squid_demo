#![cfg_attr(not(feature = "std"), no_std, no_main)]
extern crate alloc;

// pink_extension is short for Phala ink! extension
use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod http_client {
    use super::pink;
    use alloc::{format, string::String};
    use indoc::formatdoc;
    use pink::{http_get, PinkEnvironment};
    use scale::{Decode, Encode};
    //use serde::de::value;

    // you have to use crates with `no_std` support in contract.

    #[derive(Debug, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidEthAddress,
        HttpRequestFailed,
        InvalidResponseBody,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    /// Defines the storage of your contract.
    /// All the fields will be encrypted and stored on-chain.
    /// In this stateless example, we just add a useless field for demo.
    #[ink(storage)]
    pub struct HttpClient {
        demo_field: bool,
        data: Vec<String>,
        last_indexed_block: u64,
    }

    impl HttpClient {
        /// Constructor to initializes your contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                demo_field: true,
                data: Vec::new(),
                last_indexed_block: 0,
            }
        }

        /// A function to handle direct off-chain Query from users.
        /// Such functions use the immutable reference `&self`
        /// so WILL NOT change the contract state.
        ///

        #[ink(message)]
        pub fn start_indexer(&mut self, start: i32, end: i32) -> Result<String> {
            // get account ETH balance with HTTP requests to Etherscan
            // you can send any HTTP requests in Query handler
            for i in start..end {
                let resp = http_get!(
                    "https://v2.archive.subsquid.io/network/ethereum-mainnet/1000/worker"
                );
                if resp.status_code != 200 {
                    return Err(Error::HttpRequestFailed);
                }

                let worker = String::from_utf8(resp.body.to_vec()).unwrap();
                let query = formatdoc!(
                    r#"
    {{
      "logs": [
        {{
          "address": [
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
          ],
          "topic0": [
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
          ],
          "transaction": true
        }}
      ],
      "fields": {{
        "block": {{
          "gasUsed": true
        }},
        "log": {{
          "topics": true,
          "data": true
        }}
      }},
      "fromBlock": {start},
      "toBlock": {end}
    }}
"#,
                    start = i,
                    end = i
                );
                let headers = vec![("Content-Type".into(), "application/json".into())];
                let res = pink::http_post!(worker, query, headers);
                //println!("RES:{:?}", String::from_utf8(res.body.to_vec()).unwrap());
                // self.data
                //     .push(String::from_utf8(res.body.to_vec()).unwrap());
                self.last_indexed_block = i as u64;
                store_s3(i, res.body);
            }

            Ok("ok".to_string())
        }

        pub fn get_last_indexed_block(&self) -> u64 {
            self.last_indexed_block
        }
    }

    pub fn store_s3(index: i32, val: Vec<u8>) {
        use pink_s3 as s3;

        let endpoint = "gateway.storjshare.io";
        let region = "us-east-1";
        let access_key = "<ACCESS_KEY>";
        let secret_key = "<SECRET_KEY>";

        let s3 = s3::S3::new(endpoint, region, access_key, secret_key)
            .unwrap()
            .virtual_host_mode();

        let bucket = "test-phala";
        let key = format!("block-{}", index);
        let object_key = key.as_str();
        let value = val.as_slice();

        s3.put(bucket, object_key, value).unwrap();

        /* let head = s3.head(bucket, object_key).unwrap();
        assert_eq!(head.content_length, value.len() as u64);

        let v = s3.get(bucket, object_key).unwrap();
        assert_eq!(v, value);

        s3.delete(bucket, object_key).unwrap(); */
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            // when your contract is really deployed, the Phala Worker will do the HTTP requests
            // mock is needed for local test
            pink_extension_runtime::mock_ext::mock_all_ext();

            let mut http_client = HttpClient::new();
            let account = String::from("0xD0fE316B9f01A3b5fd6790F88C2D53739F80B464");
            let res = http_client.start_indexer(12000000, 12000050);
            //http_client.store_s3();
            //println!("RES:{:?}", res);
            assert!(res.is_ok());

            // run with `cargo test -- --nocapture`
        }
    }
}
