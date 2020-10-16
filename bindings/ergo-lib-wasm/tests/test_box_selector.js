import { expect, assert } from 'chai';

import {
  ErgoBoxes, SimpleBoxSelector, Tokens, BoxValue
} from '../pkg/ergo_lib_wasm';

it('SimpleBoxSelector test', async () => {
  const unspent_boxes = ErgoBoxes.from_boxes_json([
    {
      "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
      "value": 67500000000,
      "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
      "assets": [],
      "creationHeight": 284761,
      "additionalRegisters": {},
      "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
      "index": 1
    }
  ]);
  const box_selector = new SimpleBoxSelector();
  const selection = box_selector.select(unspent_boxes, BoxValue.from_u32(1000000), new Tokens());
  assert(selection != null);
  assert(selection.boxes().get(0).value().as_i64().to_str() == unspent_boxes.get(0).value().as_i64().to_str());
});

