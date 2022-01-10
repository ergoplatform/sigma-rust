import {
    Address,
    BlockHeaders,
    BoxId,
    BoxValue,
    Contract,
    DataInputs,
    DerivationPath,
    ErgoBoxCandidate,
    ErgoBoxCandidateBuilder,
    ErgoBoxCandidates,
    ErgoBoxes,
    ErgoStateContext,
    ExtSecretKey,
    I64,
    Mnemonic,
    NetworkAddress,
    NetworkPrefix,
    PreHeader,
    SecretKey,
    SecretKeys,
    TxBuilder,
    UnsignedInput, 
    UnsignedInputs, 
    UnsignedTransaction,
    Wallet
} from "../../pkg-nodejs/ergo_lib_wasm";
import { blockContext } from "./blocks";
import { inputBoxesJson } from "./boxes";

// Roughly equivilent to Utils.paymentTransaction from `ergo-wallet`
function paymentTransaction(
    recipientAddress: Address,
    changeAddress: Address,
    transferAmt: string,
    feeAmt: BoxValue,
    changeAmt: BoxValue,
    inputIds: string[],
    currentHeight: number,
) {
    const payTo = new ErgoBoxCandidateBuilder(
        BoxValue.from_i64(I64.from_str(transferAmt)),
        Contract.pay_to_address(recipientAddress),
        currentHeight
    ).build();
    const change = new ErgoBoxCandidateBuilder(
        changeAmt,
        Contract.pay_to_address(changeAddress),
        currentHeight
    ).build();
    const fee = ErgoBoxCandidate.new_miner_fee_box(feeAmt, currentHeight);

    const unsignedInputArray = inputIds.map(BoxId.from_str).map(UnsignedInput.from_box_id)
    const unsignedInputs = new UnsignedInputs();
    unsignedInputArray.forEach((i) => unsignedInputs.add(i));

    const outputs = new ErgoBoxCandidates(payTo);

    if (change.value().as_i64().as_num() > 0) {
        outputs.add(change);    
    }

    outputs.add(fee);

    return new UnsignedTransaction(unsignedInputs, new DataInputs(), outputs);
}

async function createTransaction() {
    const receiverAddress = Address.from_testnet_str("3WyCLbJTPeUj73dNhCv9A3F4ax1ZFV3WqA8wDLMWnTYWpTq16qpt");

    // DONT USE THE BELOW PHRASE FOR YOUR WALLET, THIS IS ONLY FOR THE EXAMPLE
    const seed = Mnemonic.to_seed(
        "enrich power host vessel trim quarter genius debate arrive acoustic galaxy zoo february devote clarify",
        ""
    );
    // DONT USE THE ABOVE PHRASE FOR YOUR WALLET, THIS IS ONLY FOR THE EXAMPLE

    // derive the root extended key/secret
    const extendedSecretKey = ExtSecretKey.derive_master(seed);
    // derive the initial secret key, this is the change key and is also the owner of the boxes used as inputs
    const changePath = DerivationPath.from_string("m/44'/429'/0'/0/0");
    const changeSk = extendedSecretKey.derive(changePath);

    console.log(changeSk.public_key().to_address().to_base58(NetworkPrefix.Testnet));

    const baseAddress = extendedSecretKey.public_key().to_address();
    const myAddress = NetworkAddress.new(NetworkPrefix.Testnet, baseAddress);

    const transferAmount = "25000000";
    const feeAmt = TxBuilder.SUGGESTED_TX_FEE();
    const changeAmt = BoxValue.SAFE_USER_MIN();
    const myInputs = [
        "d91b208429510e7a613200df89d5e63a77bdacf3cdf308005d66c0919df583b7",
        "db59e14dc7a2c953a877e2b45cd9dae72421bea792e848873d47d426e8328d9b",
    ];

    const unsignedTx = paymentTransaction(
        receiverAddress,
        myAddress.address(),
        transferAmount,
        feeAmt,
        changeAmt,
        myInputs,
        141514
    );

    const blockHeaders = BlockHeaders.from_json(blockContext);
    const preHeader = PreHeader.from_block_header(blockHeaders.get(0));
    const stateCtx = new ErgoStateContext(preHeader, blockHeaders);

    const dlogSecret = SecretKey.dlog_from_bytes(changeSk.secret_key_bytes());
    const secretKeys = new SecretKeys();
    secretKeys.add(dlogSecret);

    const wallet = Wallet.from_secrets(secretKeys);

    const inputBoxes = ErgoBoxes.from_boxes_json(inputBoxesJson);
    const dataInputs = ErgoBoxes.empty();

    const signedTx = wallet.sign_transaction(stateCtx, unsignedTx, inputBoxes, dataInputs);

    console.log(signedTx.to_json());
}

createTransaction();
