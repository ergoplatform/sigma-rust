package org.ergoplatform.wallet.jni

class WalletLib {
    init {
        System.loadLibrary("ergo-wallet-lib-jni")
    }

    @JvmStatic external fun addressFromTestNet(addressStr: String): Long
    @JvmStatic external fun deleteAddress(address: Long)
}
