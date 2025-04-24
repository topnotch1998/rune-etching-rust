use log::info;
use bitcoin::{Amount, Transaction, Txid};
use ord_rs::wallet::Utxo;
use std::str::FromStr;

pub async fn broadcast_transaction(
    transaction: &Transaction,
) -> anyhow::Result<Txid> {
    let url = format!("https://blockstream.info/testnet/api/tx");
    let tx_hex = hex::encode(bitcoin::consensus::serialize(&transaction));
    info!("tx_hex ({}): {tx_hex}", tx_hex.len());

    let result = reqwest::Client::new()
        .post(&url)
        .body(tx_hex)
        .send()
        .await?;

    info!("result: {:?}", result);

    if result.status().is_success() {
        let txid = result.text().await?;
        info!("txid: {txid}");
        Ok(Txid::from_str(&txid)?)
    } else {
        Err(anyhow::anyhow!(
            "failed to broadcast transaction: {}",
            result.text().await?
        ))
    }
}

pub async fn sats_amount_from_tx_inputs(
    inputs: &[(Txid, u32)],
) -> anyhow::Result<Vec<Utxo>> {
    let mut output_inputs = Vec::with_capacity(inputs.len());
    for (txid, index) in inputs {
        let tx = get_tx_by_hash(txid).await?;
        let output = tx
            .vout
            .get(*index as usize)
            .ok_or_else(|| anyhow::anyhow!("invalid index {} for txid {}", index, txid))?;

        output_inputs.push(Utxo {
            id: *txid,
            index: *index,
            amount: Amount::from_sat(output.value),
        });
    }
    Ok(output_inputs)
}


pub async fn get_tx_by_hash(txid: &Txid) -> anyhow::Result<ApiTransaction> {
    let url = format!("https://blockstream.info/testnet/api/tx/{}", txid);
    let tx = reqwest::get(&url).await?.json().await?;
    Ok(tx)
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiTransaction {
    vout: Vec<ApiVout>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ApiVout {
    value: u64,
}