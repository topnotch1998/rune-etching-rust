mod utils;

use log::info;
use std::str::FromStr;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, Network, PrivateKey, Txid};
use ord_rs::wallet::{
    CreateCommitTransactionArgsV2, SignCommitTransactionArgs, EtchingTransactionArgs, Runestone
};
use ord_rs::{Nft, OrdTransactionBuilder};
use ordinals::{Etching, Rune, Terms};
use utils::rpc_client;
use crate::utils::{calc_fees, Fees};

pub fn parse_inputs(input: Vec<String>) -> Vec<(Txid, u32)> {
    input
        .into_iter()
        .map(|input| {
            let mut parts = input.split(':');
            let txid = Txid::from_str(parts.next().unwrap()).unwrap();
            let vout = parts.next().unwrap().parse::<u32>().unwrap();
            (txid, vout)
        })
        .collect()
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let user_utxos:Vec<String> = vec!["77b28bec4e4ec7d43d792225b2d6222e57bbbcf3ad37308e0c88906ed91a729e:1".to_string()];
    let inputs = parse_inputs(user_utxos);
    let amount = 1_000_000_000;
    let limit = 1_000;

    let network = Network::Testnet;

    let private_key = PrivateKey::from_wif("cNfPNUCLMdcSM4aJhuEiKEK44YoziFVD3EYh9tVgc4rjSTeaYwHP")?;
    let public_key = private_key.public_key(&Secp256k1::new());
    let sender_address = Address::p2wpkh(&public_key, network).unwrap();

    info!("sender address: {sender_address}",);

    let Fees {
        commit_fee,
        reveal_fee,
        ..
    } = calc_fees(network);

    info!("Commit fee: {commit_fee}, reveal fee: {reveal_fee}",);

    let inputs = rpc_client::sats_amount_from_tx_inputs(&inputs).await?;

    let mut builder = OrdTransactionBuilder::p2tr(private_key);

    let etching = Etching {
        rune: Some(Rune::from_str("CUSTOMRUNETOKENTEST").unwrap()),
        divisibility: Some(2),
        premine: Some(1_000_000),
        spacers: None,
        symbol: Some('$'),
        terms: Some(Terms {
            amount: Some(amount as u128),
            cap: Some(limit as u128),
            height: (None, None),
            offset: (None, None),
        }),
        turbo: true,
    };

    let mut inscription = Nft::new(
        Some("text/plain;charset=utf-8".as_bytes().to_vec()),
        Some(etching.rune.unwrap().to_string().as_bytes().to_vec()),
    );
    inscription.pointer = Some(vec![]);
    inscription.rune = Some(
        etching
            .rune
            .ok_or(anyhow::anyhow!("Invalid etching data; rune is missing"))?
            .commitment(),
    );
    
    let commit_tx = builder
        .build_commit_transaction_with_fixed_fees(
            network,
            CreateCommitTransactionArgsV2 {
                inputs: inputs.clone(),
                inscription,
                txin_script_pubkey: sender_address.script_pubkey(),
                leftovers_recipient: sender_address.clone(),
                commit_fee,
                reveal_fee,
                derivation_path: None,
            },
        )
        .await?;
    let signed_commit_tx = builder
        .sign_commit_transaction(
            commit_tx.unsigned_tx,
            SignCommitTransactionArgs {
                inputs,
                txin_script_pubkey: sender_address.script_pubkey(),
                derivation_path: None,
            },
        )
        .await?;

    let commit_txid = rpc_client::broadcast_transaction(&signed_commit_tx).await?;
    info!("Commit transaction broadcasted: {}", commit_txid);

    // make runestone
    let runestone = Runestone {
        etching: Some(etching),
        edicts: vec![],
        mint: None,
        pointer: Some(1),
    };

    info!("getting reveal transaction...");
    let reveal_transaction = builder
        .build_etching_transaction(EtchingTransactionArgs {
            input: ord_rs::wallet::Utxo {
                id: commit_txid,
                index: 0,
                amount: commit_tx.reveal_balance,
            },
            recipient_address: sender_address,
            redeem_script: commit_tx.redeem_script,
            runestone,
            derivation_path: None,
        })
        .await?;
    info!("reveal transaction: {reveal_transaction:?}");

    let txid = rpc_client::broadcast_transaction(&reveal_transaction).await?;
    info!("Reveal transaction broadcasted: {}", txid);
    
    Ok(())
}