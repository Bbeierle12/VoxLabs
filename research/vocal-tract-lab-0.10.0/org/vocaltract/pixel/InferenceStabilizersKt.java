package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.math.MathKt;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000\n\n\u0000\n\u0002\u0010\u000e\n\u0002\u0010\u0007\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0002H\u0000"}, d2 = {"compactDb", "", ""}, k = 2, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class InferenceStabilizersKt {
    public static final String compactDb(float f) {
        return String.valueOf(MathKt.roundToInt(f * 10.0f) / 10.0f);
    }
}
