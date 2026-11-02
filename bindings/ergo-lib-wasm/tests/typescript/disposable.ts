import { Address as BrowserAddress } from "../../pkg-browser/ergo_lib_wasm";
import { Address as NodeAddress } from "../../pkg-nodejs/ergo_lib_wasm";

type Assert<T extends true> = T;
type BrowserAddressIsDisposable = Assert<BrowserAddress extends Disposable ? true : false>;
type NodeAddressIsDisposable = Assert<NodeAddress extends Disposable ? true : false>;

declare const browserAddress: BrowserAddress;
declare const nodeAddress: NodeAddress;

browserAddress[Symbol.dispose]();
nodeAddress[Symbol.dispose]();
