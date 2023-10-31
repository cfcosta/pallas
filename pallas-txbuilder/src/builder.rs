use std::time::Instant;

use indexmap::IndexMap;

use pallas_primitives::babbage::{
    AddrKeyhash, Certificate, NativeScript, NetworkId, PlutusData, PlutusV1Script, PlutusV2Script,
    TransactionBody, TransactionInput, TransactionOutput, WitnessSet,
};

use crate::{
    asset::MultiAsset,
    fee::Fee,
    native_script::{BuildNativeScript, NativeScriptBuilder},
    plutus_script::{V1Script, V2Script},
    transaction::{self, OutputExt},
    util::*,
    NetworkParams, ValidationError,
};

pub struct TransactionBuilder {
    inputs: IndexMap<TransactionInput, TransactionOutput>,
    outputs: Vec<TransactionOutput>,

    reference_inputs: IndexMap<TransactionInput, TransactionOutput>,
    collateral: IndexMap<TransactionInput, TransactionOutput>,
    collateral_return: Option<TransactionOutput>,
    network_params: NetworkParams,
    mint: Option<MultiAsset<i64>>,
    required_signers: Vec<AddrKeyhash>,
    valid_from_slot: Option<u64>,
    valid_until_slot: Option<u64>,
    certificates: Vec<Certificate>,
    plutus_data: Vec<PlutusData>,
    native_scripts: Vec<NativeScript>,
    plutus_v1_scripts: Vec<PlutusV1Script>,
    plutus_v2_scripts: Vec<PlutusV2Script>,
}

impl TransactionBuilder {
    pub fn new(network_params: NetworkParams) -> TransactionBuilder {
        TransactionBuilder {
            network_params,

            inputs: Default::default(),
            outputs: Default::default(),
            reference_inputs: Default::default(),
            collateral: Default::default(),
            collateral_return: Default::default(),
            mint: Default::default(),
            required_signers: Default::default(),
            valid_from_slot: Default::default(),
            valid_until_slot: Default::default(),
            certificates: Default::default(),
            plutus_data: Default::default(),
            native_scripts: Default::default(),
            plutus_v1_scripts: Default::default(),
            plutus_v2_scripts: Default::default(),
        }
    }

    pub fn input(mut self, input: TransactionInput, resolved: TransactionOutput) -> Self {
        self.inputs.insert(input, resolved);
        self
    }

    pub fn reference_input(mut self, input: TransactionInput, resolved: TransactionOutput) -> Self {
        self.reference_inputs.insert(input, resolved);
        self
    }

    pub fn collateral(mut self, input: TransactionInput, resolved: TransactionOutput) -> Self {
        self.collateral.insert(input, resolved);
        self
    }

    pub fn collateral_return(mut self, output: TransactionOutput) -> Self {
        self.collateral_return = Some(output);
        self
    }

    pub fn output(mut self, output: TransactionOutput) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn mint(mut self, assets: MultiAsset<i64>) -> Self {
        self.mint = Some(assets);

        self
    }

    pub fn require_signer(mut self, signer: AddrKeyhash) -> Self {
        self.required_signers.push(signer);
        self
    }

    pub fn valid_from(mut self, timestamp: Instant) -> Result<Self, ValidationError> {
        self.valid_from_slot = Some(
            self.network_params
                .timestamp_to_slot(timestamp)
                .ok_or(ValidationError::InvalidTimestamp)?,
        );

        Ok(self)
    }

    pub fn valid_from_slot(mut self, slot: u64) -> Self {
        self.valid_from_slot = Some(slot);
        self
    }

    pub fn valid_until(mut self, timestamp: Instant) -> Result<Self, ValidationError> {
        self.valid_until_slot = Some(
            self.network_params
                .timestamp_to_slot(timestamp)
                .ok_or(ValidationError::InvalidTimestamp)?,
        );

        Ok(self)
    }

    pub fn valid_until_slot(mut self, slot: u64) -> Self {
        self.valid_until_slot = Some(slot);
        self
    }

    pub fn certificate(mut self, cert: Certificate) -> Self {
        self.certificates.push(cert);
        self
    }

    pub fn plutus_data(mut self, data: impl Into<PlutusData>) -> Self {
        self.plutus_data.push(data.into());
        self
    }

    pub fn native_script<T: BuildNativeScript>(mut self, script: NativeScriptBuilder<T>) -> Self {
        self.native_scripts.push(script.build());
        self
    }

    pub fn plutus_v1_script(mut self, script: V1Script) -> Self {
        self.plutus_v1_scripts.push(script.build());
        self
    }

    pub fn plutus_v2_script(mut self, script: V2Script) -> Self {
        self.plutus_v2_scripts.push(script.build());
        self
    }

    pub fn build(self) -> Result<transaction::Transaction, ValidationError> {
        if self.inputs.is_empty() {
            return Err(ValidationError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(ValidationError::NoOutputs);
        }

        if self.collateral.iter().any(|(_, txo)| txo.is_multiasset()) {
            return Err(ValidationError::InvalidCollateralInput);
        }

        if self
            .collateral_return
            .as_ref()
            .map(|i| i.is_multiasset())
            .unwrap_or(false)
        {
            return Err(ValidationError::InvalidCollateralReturn);
        }

        let inputs = self.inputs.iter().map(|(k, _)| k.clone()).collect();
        let reference_inputs = self
            .reference_inputs
            .iter()
            .map(|(k, _)| k.clone())
            .collect();
        let collaterals = self.collateral.iter().map(|(k, _)| k.clone()).collect();
        let outputs = self.outputs.clone();

        let mut tx = transaction::Transaction {
            body: TransactionBody {
                inputs,
                outputs,
                ttl: self.valid_until_slot,
                validity_interval_start: self.valid_from_slot,
                fee: 0,
                certificates: opt_if_empty(self.certificates),
                withdrawals: None,
                update: None,
                auxiliary_data_hash: None,
                mint: self.mint.map(|x| x.build()),
                script_data_hash: None,
                collateral: opt_if_empty(collaterals),
                required_signers: opt_if_empty(self.required_signers),
                network_id: NetworkId::from_u64(self.network_params.network_id()),
                collateral_return: self.collateral_return,
                total_collateral: None,
                reference_inputs: opt_if_empty(reference_inputs),
            },
            witness_set: WitnessSet {
                vkeywitness: None,
                native_script: opt_if_empty(self.native_scripts),
                bootstrap_witness: None,
                plutus_v1_script: opt_if_empty(self.plutus_v1_scripts),
                plutus_v2_script: opt_if_empty(self.plutus_v2_scripts),
                plutus_data: opt_if_empty(self.plutus_data),
                redeemer: None,
            },
            is_valid: true,
            auxiliary_data: None,
        };

        tx.body.auxiliary_data_hash = tx.auxiliary_data.clone().map(hash_to_bytes);
        tx = Fee::linear().with_fee(tx)?;

        Ok(tx)
    }

    pub fn build_hex(self) -> Result<String, ValidationError> {
        Ok(self.build()?.hex_encoded()?)
    }
}
