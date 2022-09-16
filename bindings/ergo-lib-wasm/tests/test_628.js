import { expect, assert } from 'chai';

import { generate_block_headers } from './utils';

import * as ergo from "..";
let wasm;
beforeEach(async () => {
  wasm = await ergo;
});


const sk = [
    "54da8675d508838de626514760324acfab112157a16109d10b89fb10604d21bb",
    "4ec6209a4fd3b5ad0e1987dc4ab7f59e7a22b165b1b02add97f845e2c5a123fe",
    "4f4595e243d29b0b2b6e2c2f234e04e776600ea458703920a671d3c43f8519e5",
]

const pk = [
    "02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69",
    "026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7aca",
    "02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c2",
]
const box1Hex = 'e091431007040004000e20c9a6742a45e261b7d3bee4124074ab011140af6ea4582f5989925756b18d4429040204000e205b81c74b04b7eed17bd1e3d37c990003183601951f22a3360a8bc5c848a320060e20ca82698f4294060b5702217169ee01c5afea4e76f89c63f4ac29296c3e4349fdd805d601e4c6a7041ad602b17201d603b4a573007202d604c2b2a5730100d605cb7204d196830301937202b17203afdc0c1d7201017203d901063c0e63d801d6088c720602ed9383010e8c720601e4c67208041a93c27208720495937205730296830201938cb2db6308b2a473030073040001730592a38cc7a70196830201aea4d901066393cbc272067306937205e4c6a7060ef0941101497287b9a1eff643791277744a74b7d598b834dc613f2ebc972e33767c61ac2b01031a0120a337e33042eaa1da67bcc7dfa5fcc444f63b8a695c9786494d7d22293eba542e1a0b40343763643761373239613164333530653839623330366336653434363536373930653933376462656435613466373038633639316466393664393736346665610763617264616e6f046572676f3f616464725f7465737431767a6730376432717033786a6530773737663938327a6b687165793530676a787273647168383979783872376e6173753937687230333968506f594e51775644627441797435756859794b74747965375a507a5a376550636336643272674b723966695a6d3644684408000000000000006408000000000000001e08000000000000000a2c61737365743132793065776d78676765676c796d6a706d70396d6a6635717a68346b67776a396368746b7076403030333463343466306337613338663833333139306434343132356666396233613064643964626238393133383136303138326139333062633532316462393540376566383564366235356530653365376638373638646539343937393666336239396561633032323830623062656361373235393734336339333333633430380e2012a8eb76bd00653da26d9af3a3660d587ddbe90f54b71ec9a505222eaa0095346898f5714bd54e3c704de321cd5b65af7605acf15741bf77b1ac8c311ccb921f00'
const box2Hex = 'e09143100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202d0ad110103689941746717cddd05c52f454e34eb6e203a84f931fdc47c52f44589f83496e807011a040763617264616e6f3f616464725f7465737431767a6730376432717033786a6530773737663938327a6b687165793530676a787273647168383979783872376e6173753937687230023130023830f9ef8c3b2616092abf58db0a8b13fd252b5f5fde10bc1c3e7c95d25d21716b8f00'
const box3Hex = '8087a70e100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202cea91100011a040763617264616e6f3f616464725f7465737431767a6730376432717033786a6530773737663938327a6b687165793530676a787273647168383979783872376e617375393768723008313030303030303009313030303030303030a37f9376f2d77f77edc36cb38b695df55ddf8a11702bcd755be9a3716a27114900'
const box4Hex = 'e09143100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202c9a9110158e8ee406cbd6d8b192dd64adff64eebe9336e2e673993916ccbbd6455f6911f8087a70e011a040763617264616e6f3f616464725f7465737431767a6730376432717033786a6530773737663938327a6b687165793530676a787273647168383979783872376e617375393768723007323030303030300738303030303030d78a5ae3565d2f2d5cb5c3db50133a087fb5f52daf4b5f7ed88eb6fe35b6af0c00'
const box5Hex = '80ade204100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202dfa4110103689941746717cddd05c52f454e34eb6e203a84f931fdc47c52f44589f83496a08d060099f9ff02bc08f67727f7168a9a37b773c67b3f5eab2df65d9fe8c6cf58c21ab200'
const box6Hex = '80ade204100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202dfa4110158e8ee406cbd6d8b192dd64adff64eebe9336e2e673993916ccbbd6455f6911fa08d06007c481e76bd7fdc8ccc9a0206b5b4148a7373fdb0b9fd5cedb89a7f4f7b0b432700'
const box7Hex = 'e09143100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee72028ca011010034c44f0c7a38f833190d44125ff9b3a0dd9dbb89138160182a930bc521db951e011a040763617264616e6f6c616464725f74657374317171357165757367796d71386c65647639676c747039667568356a636865746a656166686137356e3664676875723467747a63677872303773377530766a79643767717065657863616b783838396c6d70666777676d746676746471616e75333470013201353e16aa7442814278db36d4175604f1f845be5ac3f0d8add0acd5ad558d84d83e00'
const box8Hex = 'a09693b907100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202a29c11020034c44f0c7a38f833190d44125ff9b3a0dd9dbb89138160182a930bc521db95830ba2a6c892c38d508a659caf857dbe29da4343371e597efd42e40f9bc99099a516a20700206a5960d2061212509300ac83d880b2968dd8061bad0c25f636b2411ed48d5b04'
const dataBoxHex = 'e09143100504000400040004000402d802d601c2a7d602b2a5730000ea02d196830301937201c2b2a473010093c272027201938cb2db63087202730200018cb2db6308a77303000198b2e4c6a70510730400ade4c6a7041ad901030ecdee7203cdfb1001a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac85301021a032102299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a6921026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7aca2102b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c21002040494d94d8e5cd0dcf8f71d0240243aab4205243d2b592fb9424b634af5beda533b00'
const reducedTxHex = '8b0a0813f06e16e40faae349a0d5a05c9db99217bba401d9f2a148dbe509b35c9dd06b0000dee6449bf4291c70b4a92c0fa72b48eec75977014b6cf36b94601384d0fb6ab40000b1fe17ec922eb7e3a8f140ee2b80f101731cca363080937b348ededa855a125b0000b1e20438bad543c575806406fe1fdbe15657535cdedb601efb831a653cabe9ad0000e27ab8b43a031405649600d9a3ae273df8cab45ec060ea95b84774b176b01fac0000f9590bbf1567ce2e4831eb2c2626f5c86bfbc682577480a9f68d895c39e6a18200005780dfc87d433d878129375a23fa005d18741c0a36421dedff6002fbb6adce3100000b04d3e638aa31232c0afdc8fda71f6760422036f0b8a8e6dffa9d4efbb0ddc2000001c095d370d034aca2f62403d1a50e904dfb4aa9b2cf129c836c7a9333d129f76405497287b9a1eff643791277744a74b7d598b834dc613f2ebc972e33767c61ac2b0034c44f0c7a38f833190d44125ff9b3a0dd9dbb89138160182a930bc521db95a2a6c892c38d508a659caf857dbe29da4343371e597efd42e40f9bc99099a51603689941746717cddd05c52f454e34eb6e203a84f931fdc47c52f44589f8349658e8ee406cbd6d8b192dd64adff64eebe9336e2e673993916ccbbd6455f6911f06e0a71210130400040004040400040204000e20a29d9bb0d622eb8b4f83a34c4ab1b7d3f18aaaabc3aa6876912a3ebaf0da10180404040004000400010104020400040004000e2064cc72f329f5db7b69667a10af3e1726161b9b7ce918a794ea80b9c32c4ce38805020101d807d601b2a5730000d6028cb2db6308a773010001d603aeb5b4a57302b1a5d901036391b1db630872037303d9010363aedb63087203d901054d0e938c7205017202d604e4c6a7041ad605b2a5730400d606db63087205d607ae7206d901074d0e938c720701720295938cb2db63087201730500017306d196830301ef7203938cb2db6308b2a473070073080001b2720473090095720796830201938cb27206730a0001720293c27205c2a7730bd801d608c2a7d196830501ef720393c27201720893e4c67201041a7204938cb2db6308b2a4730c00730d0001b27204730e00957207d801d609b27206730f0096830701938c720901720293cbc272057310e6c67205051ae6c67205060e93e4c67205070ecb720893e4c67205041a7204938c72090273117312bbae11030001010f021e011a0120a337e33042eaa1da67bcc7dfa5fcc444f63b8a695c9786494d7d22293eba542ee0a7120008cd033e819e995909938ea0070ee8af357693938220540f9ca5443fd8253a21d101e3bbae1101021e00e0a7120008cd02e7c7c9ff46e5a3390e9f5546013de6700484c59086de40a6f62eabaf18c13483bbae1101010a00e0a7120008cd037a9b9504ff7701431700917741864cf4702c59bf86ee3015168d2269c9e18b22bbae1101013c00c08dffd107100304000e20a6ac381e6fa99929fd1477b3ba9499790a775e91d4c14c5aa86e9a118dfac8530400d801d601b2db6501fe730000ea02d1aedb63087201d901024d0e938c720201730198b2e4c672010510730200ade4c67201041ad901020ecdee7202bbae11040388950604a094ad0e01cc0a02e60600e091431005040004000e36100204a00b08cd0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798ea02d192a39a8cc7a701730073011001020402d19683030193a38cc7b2a57300000193c2b2a57301007473027303830108cdeeac93b1a57304bbae110000d300980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c200980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c200980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c200980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c200980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c200980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c200980203cd02299bbf9b7481dce78c0b6559194a385811af1ffdf6d905671c1c82882b114a69cd026e558b0a51dd7f7c3d896ecc3ed0f515d8108dd6afa2e2c6b2d6bb7585ab7acacd02b72f60c0554b710a79721b63d3d2cc4840934c92a74601a3461f8ac2185c59c20000'

it('i628', async () => {
  // https://github.com/ergoplatform/sigma-rust/issues/628
  const box1 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box1Hex, "hex")))
  const box2 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box2Hex, "hex")))
  const box3 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box3Hex, "hex")))
  const box4 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box4Hex, "hex")))
  const box5 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box5Hex, "hex")))
  const box6 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box6Hex, "hex")))
  const box7 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box7Hex, "hex")))
  const box8 = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(box8Hex, "hex")))
  const dataBox = wasm.ErgoBox.sigma_parse_bytes(Uint8Array.from(Buffer.from(dataBoxHex, "hex")))
  const reduced = wasm.ReducedTransaction.sigma_parse_bytes(Uint8Array.from(Buffer.from(reducedTxHex, "hex")))
  const sks = new wasm.SecretKeys()
  sks.add(wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from(sk[0], "hex"))))
  sks.add(wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from(sk[2], "hex"))))
  const prover = wasm.Wallet.from_secrets(sks)
  const signed = prover.sign_reduced_transaction(reduced)
  console.log(signed.to_json())

  const firstSignerSecretKeys = new wasm.SecretKeys()
  firstSignerSecretKeys.add(wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from(sk[0], "hex"))))
  const firstProver = wasm.Wallet.from_secrets(firstSignerSecretKeys);

  const temp = wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from(sk[0], "hex")))

  const firstTransactionHintsBag = wasm.TransactionHintsBag.empty()

  const secondSignerSecretKeys = new wasm.SecretKeys()
  secondSignerSecretKeys.add(wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from(sk[1], "hex"))))
  const secondProver = wasm.Wallet.from_secrets(secondSignerSecretKeys);
  const secondCommitments = secondProver.generate_commitments_for_reduced_transaction(reduced)
  let secondCommitment = secondCommitments.all_hints_for_input(1)
  const secondKnown = [secondCommitment.get(0)]
  const secondOwn = [secondCommitment.get(1)]
  let secondHintBag = wasm.HintsBag.empty()
  secondHintBag.add_commitment(secondCommitment.get(0))
  firstTransactionHintsBag.add_hints_for_input(1, secondHintBag)

  for (let i = 2; i < 8; i++) {
      secondCommitment = secondCommitments.all_hints_for_input(i)
      if (secondCommitment.len() > 0) {
          secondHintBag = wasm.HintsBag.empty()
          secondHintBag.add_commitment(secondCommitment.get(0))
          firstTransactionHintsBag.add_hints_for_input(i, secondHintBag)
          secondKnown.push(secondCommitment.get(0))
          secondOwn.push(secondCommitment.get(1))
      }
  }

  const thirdSignerSecretKeys = new wasm.SecretKeys()
  thirdSignerSecretKeys.add(wasm.SecretKey.dlog_from_bytes(Uint8Array.from(Buffer.from(sk[2], "hex"))))
  const thirdProver = wasm.Wallet.from_secrets(thirdSignerSecretKeys);

  const thirdCommitments = thirdProver.generate_commitments_for_reduced_transaction(reduced)
  let thirdCommitment = thirdCommitments.all_hints_for_input(1)
  const thirdKnown = [thirdCommitment.get(0)]
  const thirdOwn = [thirdCommitment.get(1)]
  let thirdHintBag = wasm.HintsBag.empty()

  thirdHintBag.add_commitment(thirdCommitment.get(0))
  firstTransactionHintsBag.add_hints_for_input(1, thirdHintBag)

  for (let i = 2; i < 8; i++) {
      thirdCommitment = thirdCommitments.all_hints_for_input(i)
      if (thirdCommitment.len() > 0) {
          thirdHintBag = wasm.HintsBag.empty()
          thirdHintBag.add_commitment(thirdCommitment.get(0))
          firstTransactionHintsBag.add_hints_for_input(i, thirdHintBag)
          thirdKnown.push(thirdCommitment.get(0))
          thirdOwn.push(thirdCommitment.get(1))
      }
  }
  // TODO: why removing one of the hints for index 1 make prover to pass?
  const firstSign = firstProver.sign_reduced_transaction_multi(reduced, firstTransactionHintsBag)
});
