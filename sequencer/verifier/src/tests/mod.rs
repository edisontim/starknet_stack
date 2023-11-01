#[cfg(test)]
mod tests {
    // TODO add a test that proves a program and posts the content to the Bitcoin blockchain directly, annoying to do because need to connect a wallet, and actually do the inscription, etc
    use crate::sov_coin::*;
    use crate::verifier;
    use bitcoincore_rpc::{Auth, Client};
    use cairo_platinum_prover::{
        air::generate_cairo_proof, cairo_layout::CairoLayout, runner::run::generate_prover_args,
    };
    use crypto::Digest;
    use stark_platinum_prover::proof::options::ProofOptions;
    use std::{env, fs};
    use tokio::sync::mpsc::channel;

    const CAIRO0_PROGRAM_PATH: &str = "./programs/cairo0.json";

    fn run_program_and_get_proof(program_content: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let (main_trace, pub_inputs) =
            generate_prover_args(program_content, &None, CairoLayout::Plain).unwrap();
        let proof_options_prover = ProofOptions::default_test_options();
        let proof = generate_cairo_proof(&main_trace, &pub_inputs, &proof_options_prover).unwrap();

        let proof_bytes = serde_cbor::to_vec(&proof).unwrap();
        let public_inputs_bytes = serde_cbor::to_vec(&pub_inputs).unwrap();

        (proof_bytes, public_inputs_bytes)
    }

    #[test]
    fn test_search_vec() {
        let haystack = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".as_bytes();
        let needle_1 = "laborum".as_bytes();
        let needle_2 = "adipiscing".as_bytes();

        assert!(search_vec(haystack, needle_1).unwrap() == 437);
        assert!(search_vec(haystack, needle_2).unwrap() == 40);
    }

    #[test]
    fn test_parse_inscription() {
        let path = String::from(env!("CARGO_MANIFEST_DIR")) + "/src/tests/utils/inscription";
        let inscription = fs::read(path).unwrap();
        let da_block = DABlock::from_inscription(&inscription);
        assert!(
            da_block.prev_inscription_id
                == "0000000000000000000000000000000000000000000000000000000000000000i0".as_bytes()
        );
        assert!(
				da_block.pub_inputs
					== "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. ".as_bytes()
			);
        assert!(
				da_block.proof
					== "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.noncex".as_bytes()
			);
    }

    #[test]
    fn test_valid_proof() {
        let program_content = std::fs::read(CAIRO0_PROGRAM_PATH).unwrap();
        let (proof_bytes, public_inputs_bytes) = run_program_and_get_proof(&program_content);
        let da_block = DABlock {
            proof: proof_bytes,
            pub_inputs: public_inputs_bytes,
            prev_inscription_id: "0x0".as_bytes().to_vec(),
        };
        assert!(da_block.verify());
    }

    #[test]
    fn test_valid_proof_from_file() {
        let program_content = std::fs::read(CAIRO0_PROGRAM_PATH).unwrap();
        let (proof_bytes, public_inputs_bytes) = run_program_and_get_proof(&program_content);

        let temp_dir = String::from("./temp/");
        let _ = fs::create_dir(temp_dir.clone());
        let proof_path = temp_dir.clone() + "proof";
        let input_path = temp_dir.clone() + "pub_in";

        let _ = fs::write(proof_path.clone(), &proof_bytes);
        let _ = fs::write(input_path.clone(), &public_inputs_bytes);

        drop(proof_bytes);
        drop(public_inputs_bytes);

        let proof_bytes = fs::read(proof_path.clone()).unwrap();
        let pub_in_bytes = fs::read(input_path.clone()).unwrap();

        let _ = fs::remove_file(proof_path);
        let _ = fs::remove_file(input_path);
        let _ = fs::remove_dir(temp_dir);

        let da_block = DABlock {
            proof: proof_bytes,
            pub_inputs: pub_in_bytes,
            prev_inscription_id: "0x0".as_bytes().to_vec(),
        };

        assert!(da_block.verify());
    }

    #[test]
    fn test_valid_proof_from_inscription() {
        let btc_rpc_client = Client::new(
            "http://localhost:18332",
            Auth::UserPass("tim".to_string(), "tim".to_string()),
        )
        .unwrap();

        assert!(DABlock::from_inscription_id(
            &btc_rpc_client,
            "ff06f3370b5393c990533baefd80bef08d47cdaff8088246c1359db1366d60fei0"
        )
        .unwrap()
        .verify());
    }

    #[tokio::test]
    async fn test_verify_proof_from_runner() {
        let (tx_block, rx_block) = channel(1000);
        let (tx_verified, mut rx_verified) = channel(1000);
        verifier::Verifier::spawn(rx_block, tx_verified);

        let digest_bytes: [u8; 32] = [
            0xff, 0x06, 0xf3, 0x37, 0x0b, 0x53, 0x93, 0xc9, 0x90, 0x53, 0x3b, 0xae, 0xfd, 0x80,
            0xbe, 0xf0, 0x8d, 0x47, 0xcd, 0xaf, 0xf8, 0x08, 0x82, 0x46, 0xc1, 0x35, 0x9d, 0xb1,
            0x36, 0x6d, 0x60, 0xfe,
        ];
        let digest = Digest(digest_bytes);
        let _ = tx_block.send(digest.clone()).await;
        let (blk_hash, proof_is_valid) = rx_verified.recv().await.unwrap();
        assert!(blk_hash == digest);
        assert!(proof_is_valid);
    }
}
