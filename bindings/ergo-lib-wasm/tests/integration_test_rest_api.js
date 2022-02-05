import { assert } from 'chai';

import {
  get_info, NodeConf,
} from '../pkg/ergo_lib_wasm';

import fetch from 'node-fetch';

// @ts-ignore
global.fetch = fetch;
// @ts-ignore
global.Headers = fetch.Headers;
// @ts-ignore
global.Request = fetch.Request;
// @ts-ignore
global.Response = fetch.Response;

it('node REST API get_info endpoint', async () => {
  let node_conf = new NodeConf("127.0.0.1:9053");
  assert(node_conf != null);
  let res = await get_info(node_conf);
  assert(res != null);
  assert(node_conf != null);
});
