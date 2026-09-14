package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.math.MathKt;
import kotlin.ranges.RangesKt;

/* compiled from: DiagnosticsCore.kt */
@Metadata(d1 = {"\u0000\u0010\n\u0000\n\u0002\u0010\u000e\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\b\u001a\f\u0010\u0000\u001a\u00020\u0001*\u00020\u0002H\u0002\u001a\f\u0010\u0003\u001a\u00020\u0004*\u00020\u0002H\u0002"}, d2 = {"oneDecimal", "", "", "percent", ""}, k = 2, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class DiagnosticsCoreKt {
    /* JADX INFO: Access modifiers changed from: private */
    public static final String oneDecimal(float f) {
        return String.valueOf(MathKt.roundToInt(f * 10.0f) / 10.0f);
    }

    /* JADX INFO: Access modifiers changed from: private */
    public static final int percent(float f) {
        return MathKt.roundToInt(RangesKt.coerceIn(f, 0.0f, 1.0f) * 100.0f);
    }
}
