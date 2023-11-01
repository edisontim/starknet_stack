use bitcoincore_rpc::{Client, RpcApi};
use cairo_platinum_prover::air::{verify_cairo_proof, PublicInputs};
use lambdaworks_math::field::fields::fft_friendly::stark_252_prime_field::Stark252PrimeField;
use ord::inscription::Inscription;
use ord::inscription_id::InscriptionId;
use stark_platinum_prover::proof::{options::ProofOptions, stark::StarkProof};
use std::str::FromStr;

pub struct DABlock {
    pub prev_inscription_id: Vec<u8>,
    pub proof: Vec<u8>,
    pub pub_inputs: Vec<u8>,
}

pub fn search_vec(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        // special case: `haystack.windows(0)` will panic, so this case
        // needs to be handled separately in whatever way you feel is
        // appropriate
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|subslice| subslice == needle)
}

impl DABlock {
    pub fn from_inscription(inscription: &[u8]) -> Self {
        let inscription_id_size = 66;
        let prev_inscription_id = inscription[0..inscription_id_size].to_vec();
        let delimiter = "public_inputs_end";
        let delimiter_start_idx = search_vec(&inscription, delimiter.as_bytes()).unwrap();

        let pub_inputs = inscription[inscription_id_size..delimiter_start_idx].to_vec();

        let proof_start_idx = delimiter_start_idx + delimiter.len();
        let proof_end_idx =
            search_vec(&inscription, "nonce".as_bytes()).unwrap() + "nonce".len() + 1;
        let proof = inscription[proof_start_idx..proof_end_idx].to_vec();
        DABlock {
            prev_inscription_id,
            pub_inputs,
            proof,
        }
    }

    pub fn from_inscription_id(
        btc_rpc_client: &Client,
        inscription_id: &str,
    ) -> Result<Self, eyre::Error> {
        let tx_inscriptions = Inscription::from_transaction(
            &btc_rpc_client
                .get_raw_transaction_info(
                    &InscriptionId::from_str(inscription_id)?.txid,
                    Option::None,
                )?
                .transaction()?,
        );
        // Assume for now our inscription is the first in the transaction
        Ok(DABlock::from_inscription(
            &tx_inscriptions[0]
                .inscription
                .body
                .as_ref()
                .ok_or_else(|| eyre::eyre!("Missing inscription body"))?,
        ))
    }

    pub fn verify(self) -> bool {
        let stark_proof: StarkProof<Stark252PrimeField> =
            serde_cbor::from_slice(&self.proof).unwrap();
        let proof_options = ProofOptions::default_test_options();
        let public_inputs: PublicInputs = serde_cbor::from_slice(&self.pub_inputs).unwrap();

        verify_cairo_proof(&stark_proof, &public_inputs, &proof_options)
    }
}
