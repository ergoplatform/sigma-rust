package org.ergoplatform.wallet.jni

object WalletLib {
    init {
        System.loadLibrary("ergowalletlibjni")
    }

    @JvmStatic external fun addressFromTestNet(addressStr: String): Long
    @JvmStatic external fun addressDelete(address: Long)
}
