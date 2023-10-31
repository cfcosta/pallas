use std::time::Instant;

use pallas_txbuilder::prelude::*;

macro_rules! assert_transaction {
    ($code:expr) => {{
        let bytes = hex::decode($code).expect("Failed to decode transaction CBOR");
        let cbor: serde_cbor::Value =
            serde_cbor::from_slice(&bytes).expect("Failed to parse transaction CBOR");

        insta::assert_yaml_snapshot!(
            &cbor,
            {
                "[0][3]" => "valid_until",
                "[0][8]" => "valid_after",
            }
        );

        Ok(())
    }};
}

#[test]
fn test_build_simplest_transaction() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_transaction_with_multiple_inputs() -> Result<(), ValidationError> {
    let input_a = Input::build([0; 32], 0);
    let resolved_a = Output::lovelaces(vec![], 1000000).build();

    let input_b = Input::build([0; 32], 1);
    let resolved_b = Output::lovelaces(vec![], 1000001).build();

    let output = Output::lovelaces(vec![], 1000000).build();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input_a, resolved_a)
        .input(input_b, resolved_b)
        .output(output)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_transaction_with_multiple_outputs() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();

    let output_a = Output::lovelaces(vec![], 499999).build();
    let output_b = Output::lovelaces(vec![], 500001).build();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output_a)
        .output(output_b)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_transaction_with_ttl() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let slot = 101938047;

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .valid_until_slot(slot)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_transaction_with_timestamp_ttl() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let valid_until = Instant::now();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .valid_until(valid_until)?
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_transaction_with_valid_after() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let slot = 101938047;

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .valid_from_slot(slot)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_transaction_with_timestamp_valid_after() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let valid_after = Instant::now();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .valid_from(valid_after)?
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_multiasset_transaction() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);

    let assets = MultiAsset::new().add([0; 28].into(), "MyAsset", 1000000)?;

    let resolved = Output::multiasset(vec![], 1000000, assets.clone()).build();
    let output = Output::multiasset(vec![], 1000000, assets).build();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_mint() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let assets = MultiAsset::new().add([0; 28].into(), "MyAsset 2", 1000000)?;

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .mint(assets)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_with_reference_inputs() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input.clone(), resolved.clone())
        .output(output)
        .reference_input(input, resolved)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_with_collateral_inputs() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 999998).build();

    let collateral = Input::build([0; 32], 1);
    let collateral_return = Output::lovelaces(vec![], 2).build();

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input.clone(), resolved.clone())
        .output(output)
        .collateral(collateral, resolved)
        .collateral_return(collateral_return)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_with_plutus_data() -> Result<(), ValidationError> {
    use plutus::*;

    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let data = map().item(int(1), int(2));

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .plutus_data(data)
        .build_hex()?;

    assert_transaction!(tx)
}

#[test]
fn test_build_with_native_script() -> Result<(), ValidationError> {
    let input = Input::build([0; 32], 0);
    let resolved = Output::lovelaces(vec![], 1000000).build();
    let output = Output::lovelaces(vec![], 1000000).build();

    let script = NativeScript::all()
        .add(NativeScript::pubkey([0; 28]))
        .add(NativeScript::pubkey([1; 28]));

    let tx = TransactionBuilder::new(NetworkParams::mainnet())
        .input(input, resolved)
        .output(output)
        .native_script(script)
        .build_hex()?;

    assert_transaction!(tx)
}
