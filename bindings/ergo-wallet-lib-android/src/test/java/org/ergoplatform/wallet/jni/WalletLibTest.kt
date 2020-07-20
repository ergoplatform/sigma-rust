package org.ergoplatform.wallet.jni

import org.junit.Test
import org.junit.Assert.assertNotEquals

class WalletLibTest {

    @Test
    fun addressParsing() {
        val addressPtr = WalletLib.addressFromTestNet("test")
        assertNotEquals(addressPtr, 0)
        WalletLib.deleteAddress(addressPtr)
    }
}
