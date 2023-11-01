use crate::sov_coin::DABlock;
use bitcoincore_rpc::{Auth, Client};
use crypto::Digest;
use ord::inscription_id::InscriptionId;
use std::str::FromStr;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct Verifier {
    rx_block: Receiver<Digest>,
    tx_verified: Sender<(Digest, bool)>,
    btc_client: Client,
}

impl Verifier {
    // TODO add params for port and Auth
    pub fn spawn(rx_block: Receiver<Digest>, tx_verified: Sender<(Digest, bool)>) {
        tokio::spawn(async move {
            Self {
                rx_block,
                tx_verified,
                btc_client: Client::new(
                    "http://localhost:18332",
                    Auth::UserPass("tim".to_string(), "tim".to_string()),
                )
                .expect("Wrong URL provided"),
            }
            .run()
            .await;
        });
    }

    async fn run(&mut self) {
        while let Some(blk_hash) = self.rx_block.recv().await {
            // This really has to be done in a better way
            let inscription_id: InscriptionId = InscriptionId::from_str(
                &(reqwest::get(
                    String::from("http://localhost:4005/get_inscription_id_by_block_hash/")
                        + &hex::encode(blk_hash.0),
                )
                .await
                .expect("Failed to get an answer from the watcher_prover")
                .json::<serde_json::Value>()
                .await
                .expect("Failed to parse inscription id"))["block_hash"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
            let da_block =
                DABlock::from_inscription_id(&self.btc_client, &inscription_id.to_string())
                    .expect("Block not found from inscription_id");
            let _ = self.tx_verified.send((blk_hash, da_block.verify())).await;
        }
    }
}
