import { Address as BrowserAddress } from "../../pkg-browser/ergo_lib_wasm";
import { Address as NodeAddress } from "../../pkg-nodejs/ergo_lib_wasm";

declare const browserAddress: BrowserAddress;
declare const nodeAddress: NodeAddress;

browserAddress.free();
nodeAddress.free();
