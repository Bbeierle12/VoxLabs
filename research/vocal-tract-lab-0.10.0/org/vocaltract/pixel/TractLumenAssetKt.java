package org.vocaltract.pixel;

import java.nio.ByteBuffer;
import java.util.Arrays;
import kotlin.Metadata;
import kotlin.UByte;
import kotlin.UShort;
import kotlin.collections.ArraysKt;
import kotlin.jvm.functions.Function1;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: TractLumenAsset.kt */
@Metadata(d1 = {"\u0000\u001c\n\u0000\n\u0002\u0010\b\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0002\u0010\u0012\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0002H\u0002\u001a\u0014\u0010\u0003\u001a\u00020\u0004*\u00020\u00022\u0006\u0010\u0005\u001a\u00020\u0001H\u0002\u001a\f\u0010\u0006\u001a\u00020\u0007*\u00020\bH\u0002"}, d2 = {"unsignedShort", "", "Ljava/nio/ByteBuffer;", "readFloatArray", "", "count", "hex", "", ""}, k = 2, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class TractLumenAssetKt {
    /* JADX INFO: Access modifiers changed from: private */
    public static final int unsignedShort(ByteBuffer byteBuffer) {
        return byteBuffer.getShort() & UShort.MAX_VALUE;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final float[] readFloatArray(ByteBuffer byteBuffer, int i) {
        float[] fArr = new float[i];
        for (int i2 = 0; i2 < i; i2++) {
            fArr[i2] = byteBuffer.getFloat();
        }
        return fArr;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final String hex(byte[] bArr) {
        return ArraysKt.joinToString$default(bArr, (CharSequence) "", (CharSequence) null, (CharSequence) null, 0, (CharSequence) null, new Function1() { // from class: org.vocaltract.pixel.TractLumenAssetKt$$ExternalSyntheticLambda0
            /* JADX DEBUG: Don't trust debug lines info. Lines numbers was adjusted: min line is 0 */
            @Override // kotlin.jvm.functions.Function1
            public final Object invoke(Object obj) {
                CharSequence hex$lambda$0;
                hex$lambda$0 = TractLumenAssetKt.hex$lambda$0(((Byte) obj).byteValue());
                return hex$lambda$0;
            }
        }, 30, (Object) null);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final CharSequence hex$lambda$0(byte b) {
        String format = String.format("%02x", Arrays.copyOf(new Object[]{Integer.valueOf(b & UByte.MAX_VALUE)}, 1));
        Intrinsics.checkNotNullExpressionValue(format, "format(...)");
        return format;
    }
}
