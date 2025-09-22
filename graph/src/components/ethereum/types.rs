use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, sync::Arc};
use web3::types::{
    Action, Address, Block, Bytes, Index, Log, Res, Trace, Transaction, TransactionReceipt, H2048,
    H256, U256, U64,
};

use crate::{
    blockchain::{BlockPtr, BlockTime},
    prelude::{transaction_receipt::LightTransactionReceipt, BlockNumber},
};

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct LightTransaction {
    /// Hash
    pub hash: H256,
    /// Nonce
    pub nonce: U256,
    /// Transaction Index. None when pending.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<Index>,
    /// Sender
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Recipient (None when contract creation)
    pub to: Option<Address>,
    /// Transfered value
    pub value: U256,
    /// Gas Price
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,
    /// Gas amount
    pub gas: U256,
    /// Input data
    pub input: Bytes,
}

impl From<Transaction> for LightTransaction {
    fn from(tx: Transaction) -> Self {
        Self {
            hash: tx.hash,
            nonce: tx.nonce,
            transaction_index: tx.transaction_index,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_price: tx.gas_price,
            gas: tx.gas,
            input: tx.input,
        }
    }
}

impl From<&Transaction> for LightTransaction {
    fn from(tx: &Transaction) -> Self {
        Self {
            hash: tx.hash,
            nonce: tx.nonce,
            transaction_index: tx.transaction_index,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_price: tx.gas_price,
            gas: tx.gas,
            input: tx.input.clone(),
        }
    }
}

pub type LightEthereumBlockV1 = Block<Transaction>;

pub type LightEthereumBlockV2 = Block<LightTransaction>;

pub type LightEthereumBlock = LightEthereumBlockV2;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreTransactionReceipt {
    /// Transaction hash.
    #[serde(rename = "transactionHash")]
    pub transaction_hash: H256,
    /// Index within the block.
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Index,
    /// Hash of the block this transaction was included within.
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,
    /// Number of the block this transaction was included within.
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U64>,
    /// Cumulative gas used within the block after this was executed.
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,
    /// Gas used by this transaction alone.
    ///
    /// Gas used is `None` if the the client is running in light client mode.
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<U256>,
    /// Contract address created, or `None` if not a deployment.
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<Address>,
    /// Logs generated within this transaction.
    pub logs: Vec<Log>,
    /// Status: either 1 (success) or 0 (failure).
    pub status: Option<U64>,
    /// State root.
    pub root: Option<H256>,
    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: H2048,
}

impl From<TransactionReceipt> for StoreTransactionReceipt {
    fn from(receipt: TransactionReceipt) -> StoreTransactionReceipt {
        Self {
            transaction_hash: receipt.transaction_hash,
            transaction_index: receipt.transaction_index,
            block_hash: receipt.block_hash,
            block_number: receipt.block_number,
            cumulative_gas_used: receipt.cumulative_gas_used,
            gas_used: receipt.gas_used,
            contract_address: receipt.contract_address,
            logs: receipt.logs,
            status: receipt.status,
            root: receipt.root,
            logs_bloom: receipt.logs_bloom,
        }
    }
}

pub trait LightEthereumBlockFromV1To<T> {
    fn from_v1(block: LightEthereumBlockV1) -> T;
}

impl LightEthereumBlockFromV1To<LightEthereumBlock> for LightEthereumBlock {
    fn from_v1(block: LightEthereumBlockV1) -> Self {
        LightEthereumBlock {
            hash: block.hash,
            parent_hash: block.parent_hash,
            // sha3_uncles: block.sha3_uncles,
            uncles_hash: block.uncles_hash,
            author: block.author,
            state_root: block.state_root,
            transactions_root: block.transactions_root,
            receipts_root: block.receipts_root,
            number: block.number,
            gas_used: block.gas_used,
            gas_limit: block.gas_limit,
            base_fee_per_gas: block.base_fee_per_gas,
            extra_data: block.extra_data,
            logs_bloom: block.logs_bloom,
            timestamp: block.timestamp,
            difficulty: block.difficulty,
            total_difficulty: block.total_difficulty,
            seal_fields: block.seal_fields,
            uncles: block.uncles,
            transactions: block
                .transactions
                .into_iter()
                .map(LightTransaction::from)
                .collect(),
            size: block.size,
            mix_hash: block.mix_hash,
            nonce: block.nonce,
        }
    }
}

#[derive(Debug)]
struct ConversionError;

pub trait LightEthereumBlockTryFromV1To<T> {
    fn try_from(block: LightEthereumBlockV1) -> T;
}

impl LightEthereumBlockTryFromV1To<Result<LightEthereumBlock, ConversionError>>
    for LightEthereumBlock
{
    fn try_from(block: LightEthereumBlockV1) -> Result<LightEthereumBlock, ConversionError> {
        Ok(<LightEthereumBlock as LightEthereumBlockFromV1To<
            LightEthereumBlock,
        >>::from_v1(block))
    }
}

pub trait LightEthereumBlockExt {
    fn number(&self) -> BlockNumber;
    fn transaction_for_log(&self, log: &Log) -> Option<LightTransaction>;
    fn transaction_for_call(&self, call: &EthereumCall) -> Option<LightTransaction>;
    fn parent_ptr(&self) -> Option<BlockPtr>;
    fn format(&self) -> String;
    fn block_ptr(&self) -> BlockPtr;
    fn timestamp(&self) -> BlockTime;
}

impl LightEthereumBlockExt for LightEthereumBlock {
    fn number(&self) -> BlockNumber {
        BlockNumber::try_from(self.number.unwrap().as_u64()).unwrap()
    }

    fn transaction_for_log(&self, log: &Log) -> Option<LightTransaction> {
        log.transaction_hash
            .and_then(|hash| self.transactions.iter().find(|tx| tx.hash == hash))
            .cloned()
    }

    fn transaction_for_call(&self, call: &EthereumCall) -> Option<LightTransaction> {
        call.transaction_hash
            .and_then(|hash| self.transactions.iter().find(|tx| tx.hash == hash))
            .cloned()
    }

    fn parent_ptr(&self) -> Option<BlockPtr> {
        match self.number() {
            0 => None,
            n => Some(BlockPtr::from((self.parent_hash, n - 1))),
        }
    }

    fn format(&self) -> String {
        format!(
            "{} ({})",
            self.number
                .map_or(String::from("none"), |number| format!("#{}", number)),
            self.hash
                .map_or(String::from("-"), |hash| format!("{:x}", hash))
        )
    }

    fn block_ptr(&self) -> BlockPtr {
        BlockPtr::from((self.hash.unwrap(), self.number.unwrap().as_u64()))
    }

    fn timestamp(&self) -> BlockTime {
        let ts = i64::try_from(self.timestamp.as_u64()).unwrap();
        BlockTime::since_epoch(ts, 0)
    }
}

#[derive(Clone, Debug)]
pub struct EthereumBlockWithCalls {
    pub ethereum_block: EthereumBlock,
    /// The calls in this block; `None` means we haven't checked yet,
    /// `Some(vec![])` means that we checked and there were none
    pub calls: Option<Vec<EthereumCall>>,
}

impl EthereumBlockWithCalls {
    /// Given an `EthereumCall`, check within receipts if that transaction was successful.
    pub fn transaction_for_call_succeeded(&self, call: &EthereumCall) -> anyhow::Result<bool> {
        let call_transaction_hash = call.transaction_hash.ok_or(anyhow::anyhow!(
            "failed to find a transaction for this call"
        ))?;

        let receipt = self
            .ethereum_block
            .transaction_receipts
            .iter()
            .find(|txn| txn.transaction_hash == call_transaction_hash)
            .ok_or(anyhow::anyhow!(
                "failed to find the receipt for this transaction"
            ))?;

        Ok(evaluate_transaction_status(receipt.status))
    }
}

/// Evaluates if a given transaction was successful.
///
/// Returns `true` on success and `false` on failure.
/// If a receipt does not have a status value (EIP-658), assume the transaction was successful.
pub fn evaluate_transaction_status(receipt_status: Option<U64>) -> bool {
    receipt_status
        .map(|status| !status.is_zero())
        .unwrap_or(true)
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct EthereumBlockV1 {
    pub block: Arc<LightEthereumBlock>,
    pub transaction_receipts: Vec<Arc<TransactionReceipt>>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct EthereumBlockV2 {
    pub block: Arc<LightEthereumBlock>,
    pub transaction_receipts: Vec<Arc<StoreTransactionReceipt>>,
}

impl From<EthereumBlockV1> for EthereumBlockV2 {
    fn from(b: EthereumBlockV1) -> Self {
        Self {
            block: b.block,
            transaction_receipts: b
                .transaction_receipts
                .into_iter()
                .map(|arc_receipt| StoreTransactionReceipt::from((*arc_receipt).clone()))
                .map(Arc::new).collect(),
        }
    }
}

pub type EthereumBlock = EthereumBlockV2;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EthereumCall {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub gas_used: U256,
    pub input: Bytes,
    pub output: Bytes,
    pub block_number: BlockNumber,
    pub block_hash: H256,
    pub transaction_hash: Option<H256>,
    pub transaction_index: u64,
}

impl EthereumCall {
    pub fn try_from_trace(trace: &Trace) -> Option<Self> {
        // The parity-ethereum tracing api returns traces for operations which had execution errors.
        // Filter errorful traces out, since call handlers should only run on successful CALLs.
        if trace.error.is_some() {
            return None;
        }
        // We are only interested in traces from CALLs
        let call = match &trace.action {
            // Contract to contract value transfers compile to the CALL opcode
            // and have no input. Call handlers are for triggering on explicit method calls right now.
            Action::Call(call) if call.input.0.len() >= 4 => call,
            _ => return None,
        };
        let (output, gas_used) = match &trace.result {
            Some(Res::Call(result)) => (result.output.clone(), result.gas_used),
            _ => return None,
        };

        // The only traces without transactions are those from Parity block reward contracts, we
        // don't support triggering on that.
        let transaction_index = trace.transaction_position? as u64;

        Some(EthereumCall {
            from: call.from,
            to: call.to,
            value: call.value,
            gas_used,
            input: call.input.clone(),
            output,
            block_number: trace.block_number as BlockNumber,
            block_hash: trace.block_hash,
            transaction_hash: trace.transaction_hash,
            transaction_index,
        })
    }
}

impl From<EthereumBlock> for BlockPtr {
    fn from(b: EthereumBlock) -> BlockPtr {
        BlockPtr::from((b.block.hash.unwrap(), b.block.number.unwrap().as_u64()))
    }
}

impl<'a> From<&'a EthereumBlock> for BlockPtr {
    fn from(b: &'a EthereumBlock) -> BlockPtr {
        BlockPtr::from((b.block.hash.unwrap(), b.block.number.unwrap().as_u64()))
    }
}

impl<'a> From<&'a EthereumCall> for BlockPtr {
    fn from(call: &'a EthereumCall) -> BlockPtr {
        BlockPtr::from((call.block_hash, call.block_number))
    }
}
