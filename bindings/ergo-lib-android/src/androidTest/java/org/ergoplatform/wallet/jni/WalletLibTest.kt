package org.ergoplatform.wallet.jni


import android.os.Parcel
import android.text.TextUtils.writeToParcel
import androidx.test.filters.SmallTest
import org.junit.Test
import org.junit.runner.RunWith
import org.junit.Assert.assertNotEquals


@SmallTest
class WalletLibTest {

    @Test
    fun addressParsing() {
        val addressPtr = WalletLib.addressFromTestNet("3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN")
        assertNotEquals(0, addressPtr)
        WalletLib.addressDelete(addressPtr)
    }
}
