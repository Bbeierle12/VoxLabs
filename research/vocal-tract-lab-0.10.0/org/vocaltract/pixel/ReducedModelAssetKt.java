package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.List;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;
import org.json.JSONArray;

/* compiled from: ReducedModelAsset.kt */
@Metadata(d1 = {"\u0000\u001c\n\u0000\n\u0002\u0010\u0014\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0011\n\u0002\b\u0002\n\u0002\u0010 \n\u0002\u0010\u000e\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0002H\u0002\u001a\u0017\u0010\u0003\u001a\b\u0012\u0004\u0012\u00020\u00010\u0004*\u00020\u0002H\u0002¢\u0006\u0002\u0010\u0005\u001a\u0012\u0010\u0006\u001a\b\u0012\u0004\u0012\u00020\b0\u0007*\u00020\u0002H\u0002"}, d2 = {"toFloatArray", "", "Lorg/json/JSONArray;", "toFloatArray2d", "", "(Lorg/json/JSONArray;)[[F", "toStringList", "", ""}, k = 2, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class ReducedModelAssetKt {
    /* JADX INFO: Access modifiers changed from: private */
    public static final float[] toFloatArray(JSONArray jSONArray) {
        int length = jSONArray.length();
        float[] fArr = new float[length];
        for (int i = 0; i < length; i++) {
            fArr[i] = (float) jSONArray.getDouble(i);
        }
        return fArr;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final float[][] toFloatArray2d(JSONArray jSONArray) {
        int length = jSONArray.length();
        float[][] fArr = new float[length][];
        for (int i = 0; i < length; i++) {
            JSONArray jSONArray2 = jSONArray.getJSONArray(i);
            Intrinsics.checkNotNullExpressionValue(jSONArray2, "getJSONArray(...)");
            fArr[i] = toFloatArray(jSONArray2);
        }
        return fArr;
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final List<String> toStringList(JSONArray jSONArray) {
        int length = jSONArray.length();
        ArrayList arrayList = new ArrayList(length);
        for (int i = 0; i < length; i++) {
            arrayList.add(jSONArray.getString(i));
        }
        return arrayList;
    }
}
