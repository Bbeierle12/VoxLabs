package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: AcousticMatchOverlay.kt */
@Metadata(d1 = {"\u0000\u0016\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0018\u0002\n\u0000\n\u0002\u0018\u0002\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u000e\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u0007"}, d2 = {"Lorg/vocaltract/pixel/AcousticOverlayMath;", "", "<init>", "()V", "compute", "Lorg/vocaltract/pixel/AcousticMatchSummary;", "acoustic", "Lorg/vocaltract/pixel/AcousticEstimate;"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AcousticOverlayMath {
    public static final AcousticOverlayMath INSTANCE = new AcousticOverlayMath();

    private AcousticOverlayMath() {
    }

    public final AcousticMatchSummary compute(AcousticEstimate acoustic) {
        String str;
        int i;
        float f;
        Intrinsics.checkNotNullParameter(acoustic, "acoustic");
        if (!acoustic.getVoiced() || acoustic.getFormantsHz().isEmpty() || acoustic.getHarmonics().isEmpty()) {
            return AcousticMatchSummary.INSTANCE.getNONE();
        }
        int min = Math.min(4, acoustic.getFormantsHz().size());
        float f2 = 0.0f;
        int i2 = -1;
        HarmonicEstimate harmonicEstimate = null;
        float f3 = 0.0f;
        float f4 = 0.0f;
        int i3 = 0;
        while (i3 < min) {
            float floatValue = acoustic.getFormantsHz().get(i3).floatValue();
            if (!Float.isInfinite(floatValue) && !Float.isNaN(floatValue) && floatValue > f2) {
                Float f5 = (Float) CollectionsKt.getOrNull(acoustic.getFormantStdHz(), i3);
                float max = Math.max(Math.max(55.0f, 0.035f * floatValue), f5 != null ? f5.floatValue() : f2);
                for (HarmonicEstimate harmonicEstimate2 : acoustic.getHarmonics()) {
                    float frequencyHz = harmonicEstimate2.getFrequencyHz();
                    if (Float.isInfinite(frequencyHz) || Float.isNaN(frequencyHz) || harmonicEstimate2.getFrequencyHz() <= f2) {
                        i = min;
                        f = floatValue;
                    } else {
                        float frequencyHz2 = harmonicEstimate2.getFrequencyHz() - floatValue;
                        float f6 = frequencyHz2 / max;
                        f = floatValue;
                        i = min;
                        float exp = ((float) Math.exp(f6 * (-0.5f) * f6)) * ((RangesKt.coerceIn((float) Math.pow(10.0d, harmonicEstimate2.getRelativeGainDb() / 20.0f), 0.0f, 1.0f) * 0.65f) + 0.35f);
                        if (exp > f3) {
                            f3 = exp;
                            min = i;
                            i2 = i3;
                            harmonicEstimate = harmonicEstimate2;
                            f4 = frequencyHz2;
                            floatValue = f;
                            f2 = 0.0f;
                        }
                    }
                    min = i;
                    floatValue = f;
                    f2 = 0.0f;
                }
            }
            i3++;
            min = min;
            f2 = 0.0f;
        }
        if (harmonicEstimate == null) {
            return AcousticMatchSummary.INSTANCE.getNONE();
        }
        boolean z = f3 >= 0.18f;
        float floatValue2 = acoustic.getFormantsHz().get(i2).floatValue();
        if (z) {
            str = "H" + harmonicEstimate.getNumber() + " " + ((int) harmonicEstimate.getFrequencyHz()) + " Hz ↔ R" + (i2 + 1) + " " + ((int) floatValue2) + " Hz • " + ((int) (100 * f3)) + "% match";
        } else {
            str = "No close H↔R match • nearest detuning " + ((int) Math.abs(f4)) + " Hz";
        }
        return new AcousticMatchSummary(z, RangesKt.coerceIn(f3, 0.0f, 1.0f), Integer.valueOf(i2 + 1), Float.valueOf(floatValue2), Integer.valueOf(harmonicEstimate.getNumber()), Float.valueOf(harmonicEstimate.getFrequencyHz()), Float.valueOf(f4), str, null, 256, null);
    }
}
