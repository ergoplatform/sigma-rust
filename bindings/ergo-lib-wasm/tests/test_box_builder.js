import { expect, assert } from 'chai';

import {
  Address, Wallet, ErgoBox, ErgoBoxCandidateBuilder, Contract,
  ErgoBoxes, ErgoBoxCandidates,
  ErgoStateContext, TxBuilder, BoxValue, BoxSelector, SecretKey, TxId, DataInputs, NonMandatoryRegisterId, Constant
} from '../pkg/ergo_lib_wasm';

const recipient = Address.from_testnet_str('3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN');
const contract = Contract.pay_to_address(recipient);

it('ErgoBoxCandidateBuilder test', async () => {
  const b = new ErgoBoxCandidateBuilder(BoxValue.from_u32(10000000), contract, 0).build();
  assert(b != null);
});

it('ErgoBoxCandidateBuilder set register value test', async () => {
  let builder = new ErgoBoxCandidateBuilder(BoxValue.from_u32(10000000), contract, 0);
  assert(builder.register_value(NonMandatoryRegisterId.R4) == null);
  const c = Constant.from_i32(1);
  builder.set_register_value(NonMandatoryRegisterId.R4, c);
  assert(builder.register_value(NonMandatoryRegisterId.R4).to_i32() == c.to_i32());
  const b = builder.build();
  assert(b.register_value(NonMandatoryRegisterId.R4).to_i32 = c.to_i32);
});

it('ErgoBoxCandidateBuilder delete register value test', async () => {
  let builder = new ErgoBoxCandidateBuilder(BoxValue.from_u32(10000000), contract, 0);
  const c = Constant.from_i32(1);
  builder.set_register_value(NonMandatoryRegisterId.R4, c);
  assert(builder.register_value(NonMandatoryRegisterId.R4).to_i32() == c.to_i32());
  builder.delete_register_value(NonMandatoryRegisterId.R4);
  assert(builder.register_value(NonMandatoryRegisterId.R4) == null);
  const b = builder.build();
  assert(b.register_value(NonMandatoryRegisterId.R4) == null);
});
