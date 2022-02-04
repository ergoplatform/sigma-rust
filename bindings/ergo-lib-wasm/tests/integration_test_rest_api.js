import { assert } from 'chai';

import {
  get_info, NodeConf,
} from '../pkg/ergo_lib_wasm';

it('node REST API get_info endpoint', async () => {
  let node_conf = new NodeConf("localhost:9053");
  assert(node_conf != null);
  let res = await get_info(node_conf);
  assert(res != null);
  assert(node_conf != null);
});
