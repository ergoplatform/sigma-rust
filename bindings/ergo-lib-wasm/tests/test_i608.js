import { expect, assert } from 'chai';

import { generate_block_headers } from './utils';

import * as ergo from "..";
let wasm;
beforeEach(async () => {
  wasm = await ergo;
});

const publicKeys = [
  "0302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189",
  "0399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f85",
  "024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b66",
  "027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd",
];

const secrets = [
  "00eda6c0e9fc808d4cf050fc4e98705372b9f0786a6b63aa4013d1a20539b104",
  "cc2e48e5e53059e0d68866eff97a6037cb39945ea9f09f40fcec82d12cd8cb8b",
  "c97250f41cfa8d545c2f8d75b2ee24002b5feec32340c2bb81fa4e2d4c7527d3",
  "53ceef0ece83401cf5cd853fd0c1a9bbfab750d76f278b3187f1a14768d6e9c4",
];

it('i608', async () => {

  const firstSK = wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from("00eda6c0e9fc808d4cf050fc4e98705372b9f0786a6b63aa4013d1a20539b104", "hex")));
  const firstSKS = new wasm.SecretKeys();
  firstSKS.add(firstSK);
  const firstWallet = wasm.Wallet.from_secrets(firstSKS);

  const secondSK = wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from("cc2e48e5e53059e0d68866eff97a6037cb39945ea9f09f40fcec82d12cd8cb8b", "hex")));
  const secondSKS = new wasm.SecretKeys();
  secondSKS.add(secondSK);
  const secondWallet = wasm.Wallet.from_secrets(secondSKS);

  const thirdSK = wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from("c97250f41cfa8d545c2f8d75b2ee24002b5feec32340c2bb81fa4e2d4c7527d3", "hex")));
  const thirdSKS = new wasm.SecretKeys();
  thirdSKS.add(thirdSK);
  const thirdWallet = wasm.Wallet.from_secrets(thirdSKS);


  const forthSK = wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from("53ceef0ece83401cf5cd853fd0c1a9bbfab750d76f278b3187f1a14768d6e9c4", "hex")));
  const forthSKS = new wasm.SecretKeys();
  forthSKS.add(forthSK);
  const forthWallet = wasm.Wallet.from_secrets(forthSKS);


  const box1 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from("80ade204100504000400040004000402d804d601b2a5730000d602e4c6a7041ad603e4c6a70510d604ad7202d901040ecdee7204ea02d19683020193c27201c2a7938cb2db63087201730100018cb2db6308a773020001eb02ea02d19683020193e4c67201041a720293e4c672010510720398b27203730300720498b272037304007204d18b0f01a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac85301021a04210302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189210399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f8521024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b6621027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd1002060865c7e0d4a77ccd605b3e4812d38140f7e68fdf740cb6cdc1d8957b75138d1e4c00", "hex")))
  const box2 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from("80c8afa02510010e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac853d1aea4d9010163aedb63087201d901034d0e938c7203017300d18b0f00002eca6f88ea99fb4fe313d94cb05c576b1b7a94ec7166aec958b36bcea4b8ff1a01", "hex")))
  // console.log(box1.to_json())
  // console.log(box2.to_json())
  const boxes = wasm.ErgoBoxes.empty()
  boxes.add(box1)

  boxes.add(box2)

  const tempBoxes = wasm.ErgoBoxes.empty()
  tempBoxes.add(box1)

  const reduced = wasm.ReducedTransaction.sigma_parse_bytes(Uint8Array.from(Buffer.from("ce04022f4cd0df4db787875b3a071e098b72ba4923bd2460e08184b34359563febe04700005e8269c8e2b975a43dc6e74a9c5b10b273313c6d32c1dd40c171fc0a8852ca0100000001a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530480ade204100504000400040004000402d804d601b2a5730000d602e4c6a7041ad603e4c6a70510d604ad7202d901040ecdee7204ea02d19683020193c27201c2a7938cb2db63087201730100018cb2db6308a773020001eb02ea02d19683020193e4c67201041a720293e4c672010510720398b27203730300720498b272037304007204d18b0f010001021a04210302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189210399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f8521024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b6621027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd100206088094ebdc030008cd0314368e16c9c99c5a6e20dda917aeb826b3a908becff543b3a36b38e6b3355ff5d18b0f0000c0843d1005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304d18b0f0000c0af87c3210008cd0314368e16c9c99c5a6e20dda917aeb826b3a908becff543b3a36b38e6b3355ff5d18b0f00009702980304cd0302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189cd0399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f85cd024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b66cd027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd9604cd0302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189cd0399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f85cd024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b66cd027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bdf39b03d3cb9e02d073", "hex")))

  let firstHints = firstWallet.generate_commitments_for_reduced_transaction(reduced).all_hints_for_input(0);
  const firstKnown = [firstHints.get(0)];
  const firstOwn = [firstHints.get(1)];


  let secondHints = secondWallet.generate_commitments_for_reduced_transaction(reduced).all_hints_for_input(0);
  const secondKnown = [secondHints.get(0)];
  const secondOwn = [secondHints.get(1)];


  let thirdHints = thirdWallet.generate_commitments_for_reduced_transaction(reduced).all_hints_for_input(0);
  const thirdKnown = [thirdHints.get(0)];
  const thirdOwn = [thirdHints.get(1)];



  let forthHints = forthWallet.generate_commitments_for_reduced_transaction(reduced).all_hints_for_input(0);
  const forthKnown = [forthHints.get(0)];
  const forthOwn = [forthHints.get(1)];


  const firstTransactionHintsBagKnown = wasm.TransactionHintsBag.empty();
  const firstHintsBag = wasm.HintsBag.empty();
  firstHintsBag.add_commitment(firstKnown[0]);
  firstTransactionHintsBagKnown.add_hints_for_input(0, firstHintsBag);


  const secondTransactionHintsBagKnown = wasm.TransactionHintsBag.empty();
  const secondHintsBag = wasm.HintsBag.empty();
  secondHintsBag.add_commitment(secondKnown[0]);
  secondTransactionHintsBagKnown.add_hints_for_input(0, secondHintsBag);


  const thirdTransactionHintsBagKnown = wasm.TransactionHintsBag.empty();
  const thirdHintsBag = wasm.HintsBag.empty();
  thirdHintsBag.add_commitment(thirdKnown[0]);
  thirdTransactionHintsBagKnown.add_hints_for_input(0, thirdHintsBag);


  const forthTransactionHintsBagKnown = wasm.TransactionHintsBag.empty();
  const forthHintsBag = wasm.HintsBag.empty();
  forthHintsBag.add_commitment(forthKnown[0]);
  forthTransactionHintsBagKnown.add_hints_for_input(0, forthHintsBag);


  // console.log(firstTransactionHintsBagKnown.to_json().publicHints['0'])
  // console.log(secondTransactionHintsBagKnown.to_json().publicHints['0'])
  // console.log(thirdTransactionHintsBagKnown.to_json().publicHints['0'])
  // console.log(forthTransactionHintsBagKnown.to_json().publicHints['0'])

  const firstCommitment = JSON.stringify(firstTransactionHintsBagKnown.to_json())
  const secondCommitment = JSON.stringify(secondTransactionHintsBagKnown.to_json())
  const thirdCommitment = JSON.stringify(thirdTransactionHintsBagKnown.to_json())
  const forthCommitment = JSON.stringify(forthTransactionHintsBagKnown.to_json())

  const firstHintsBagFromJson = wasm.TransactionHintsBag.from_json(firstCommitment);
  const secondHintsBagFromJson = wasm.TransactionHintsBag.from_json(secondCommitment);
  const thirdHintsBagFromJson = wasm.TransactionHintsBag.from_json(thirdCommitment);
  const forthHintsBagFromJson = wasm.TransactionHintsBag.from_json(forthCommitment);

  const firstTxHintsBag = wasm.TransactionHintsBag.empty();


  firstTxHintsBag.add_hints_for_input(0, secondHintsBagFromJson.all_hints_for_input(0));
  firstTxHintsBag.add_hints_for_input(0, thirdHintsBagFromJson.all_hints_for_input(0));
  firstTxHintsBag.add_hints_for_input(0, forthHintsBagFromJson.all_hints_for_input(0));


  const secondTxHintsBag = wasm.TransactionHintsBag.empty();

  secondTxHintsBag.add_hints_for_input(0, firstHintsBagFromJson.all_hints_for_input(0));
  secondTxHintsBag.add_hints_for_input(0, thirdHintsBagFromJson.all_hints_for_input(0));
  secondTxHintsBag.add_hints_for_input(0, forthHintsBagFromJson.all_hints_for_input(0));


  const forthTxHintsBag = wasm.TransactionHintsBag.empty();
  forthTxHintsBag.add_hints_for_input(0, firstHintsBagFromJson.all_hints_for_input(0));
  forthTxHintsBag.add_hints_for_input(0, secondHintsBagFromJson.all_hints_for_input(0));
  forthTxHintsBag.add_hints_for_input(0, thirdHintsBagFromJson.all_hints_for_input(0));



  const forthSign = forthWallet.sign_reduced_transaction_multi(reduced, forthTxHintsBag);

  const firstRealPropositions = new wasm.Propositions;
  const firstSimulatedPropositions = new wasm.Propositions;

  const firstPK = Uint8Array.from(Buffer.from("cd0302e57ca7ebf8cfa1802d4bc79a455008307a936b4f50f0629d9bef484fdd5189", "hex"));
  const secondPK = Uint8Array.from(Buffer.from("cd0399f5724bbc4d08c6e146d61449c05a3e0546868b1d4f83411f325187d5ca4f85", "hex"));
  const thirdPK = Uint8Array.from(Buffer.from("cd024e06e6c6073e13a03fa4629882a69108cd60e0a9fbb2e0fcc898ce68a7051b66", "hex"));
  const forthPK = Uint8Array.from(Buffer.from("cd027a069cc972fc7816539a316ba1cfc0164656d63dd1873ee407670b0e8195f3bd", "hex"));
  firstRealPropositions.add_proposition_from_byte(forthPK);

  const block_headers = generate_block_headers();
  const pre_header = wasm.PreHeader.from_block_header(block_headers.get(0));
  const ctx = new wasm.ErgoStateContext(pre_header, block_headers, wasm.Parameters.default_parameters());

  const secondHintsBagExtract = wasm.extract_hints(forthSign, ctx, boxes, wasm.ErgoBoxes.from_boxes_json([]), firstRealPropositions, firstSimulatedPropositions);
  assert(secondHintsBagExtract !== null);
});
