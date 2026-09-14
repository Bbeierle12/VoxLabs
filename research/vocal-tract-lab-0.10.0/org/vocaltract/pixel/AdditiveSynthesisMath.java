package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;
import kotlin.math.MathKt;
import kotlin.ranges.RangesKt;

/* compiled from: AdditiveSynthesis.kt */
@Metadata(d1 = {"\u0000<\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\u0014\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0002\n\u0002\b\u0002\n\u0002\u0010\u0017\n\u0002\b\u0003\n\u0002\u0010\u0006\n\u0000\bÆ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J \u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u00072\u0006\u0010\b\u001a\u00020\u00052\b\b\u0002\u0010\t\u001a\u00020\u0005J\u0016\u0010\n\u001a\u00020\u000b2\u0006\u0010\f\u001a\u00020\r2\u0006\u0010\b\u001a\u00020\u0005J\u0016\u0010\u000e\u001a\u00020\u000f2\u0006\u0010\u0010\u001a\u00020\u000b2\u0006\u0010\u0011\u001a\u00020\u0012J(\u0010\u0013\u001a\u00020\u000b2\u0006\u0010\f\u001a\u00020\r2\u0006\u0010\b\u001a\u00020\u00052\u0006\u0010\u0014\u001a\u00020\u00052\b\b\u0002\u0010\u0015\u001a\u00020\u0016J(\u0010\u0017\u001a\u00020\u000b2\u0006\u0010\f\u001a\u00020\r2\u0006\u0010\b\u001a\u00020\u00052\u0006\u0010\u0014\u001a\u00020\u00052\b\b\u0002\u0010\u0015\u001a\u00020\u0016"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSynthesisMath;", "", "<init>", "()V", "activePartialCount", "", "fundamentalHz", "", "sampleRateHz", "partialCount", "targetAmplitudes", "", "state", "Lorg/vocaltract/pixel/AdditiveSynthState;", "floatToPcm16", "", "input", "output", "", "summedCycle", "pointCount", "phaseRadians", "", "standingField"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AdditiveSynthesisMath {
    public static final AdditiveSynthesisMath INSTANCE = new AdditiveSynthesisMath();

    private AdditiveSynthesisMath() {
    }

    public static /* synthetic */ int activePartialCount$default(AdditiveSynthesisMath additiveSynthesisMath, float f, int i, int i2, int i3, Object obj) {
        if ((i3 & 4) != 0) {
            i2 = 16;
        }
        return additiveSynthesisMath.activePartialCount(f, i, i2);
    }

    public final int activePartialCount(float fundamentalHz, int sampleRateHz, int partialCount) {
        int i = 0;
        if (!Float.isInfinite(fundamentalHz) && !Float.isNaN(fundamentalHz) && fundamentalHz > 0.0f && sampleRateHz > 0) {
            float f = sampleRateHz / 2.0f;
            if (1 <= partialCount) {
                for (int i2 = 1; i2 * fundamentalHz < f; i2++) {
                    i++;
                    if (i2 == partialCount) {
                        break;
                    }
                }
            }
        }
        return i;
    }

    public final float[] targetAmplitudes(AdditiveSynthState state, int sampleRateHz) {
        Intrinsics.checkNotNullParameter(state, "state");
        float[] fArr = new float[16];
        if (state.getPlaying() && sampleRateHz > 0) {
            float f = 0.0f;
            if (state.getMasterGain() > 0.0f) {
                int activePartialCount$default = activePartialCount$default(this, state.getFundamentalHz(), sampleRateHz, 0, 4, null);
                for (int i = 0; i < activePartialCount$default; i++) {
                    f += state.partialGain(i);
                }
                float masterGain = state.getMasterGain() / Math.max(1.0f, f);
                for (int i2 = 0; i2 < activePartialCount$default; i2++) {
                    fArr[i2] = state.partialGain(i2) * masterGain;
                }
            }
        }
        return fArr;
    }

    public final void floatToPcm16(float[] input, short[] output) {
        Intrinsics.checkNotNullParameter(input, "input");
        Intrinsics.checkNotNullParameter(output, "output");
        if (output.length < input.length) {
            throw new IllegalArgumentException("PCM output must be at least as large as the input".toString());
        }
        int length = input.length;
        for (int i = 0; i < length; i++) {
            output[i] = (short) MathKt.roundToInt(RangesKt.coerceIn(input[i], -1.0f, 1.0f) * 32767);
        }
    }

    public static /* synthetic */ float[] summedCycle$default(AdditiveSynthesisMath additiveSynthesisMath, AdditiveSynthState additiveSynthState, int i, int i2, double d, int i3, Object obj) {
        if ((i3 & 8) != 0) {
            d = 0.0d;
        }
        return additiveSynthesisMath.summedCycle(additiveSynthState, i, i2, d);
    }

    public final float[] summedCycle(AdditiveSynthState state, int sampleRateHz, int pointCount, double phaseRadians) {
        Intrinsics.checkNotNullParameter(state, "state");
        if (pointCount <= 0) {
            return new float[0];
        }
        float[] targetAmplitudes = targetAmplitudes(state, sampleRateHz);
        float[] fArr = new float[pointCount];
        for (int i = 0; i < pointCount; i++) {
            double d = 0.0d;
            double d2 = pointCount == 1 ? 0.0d : i / (pointCount - 1);
            int length = targetAmplitudes.length;
            int i2 = 0;
            while (i2 < length) {
                int i3 = i2 + 1;
                double d3 = i3;
                d += targetAmplitudes[i2] * Math.sin((6.283185307179586d * d3 * d2) + (d3 * phaseRadians));
                i2 = i3;
            }
            fArr[i] = (float) d;
        }
        return fArr;
    }

    public static /* synthetic */ float[] standingField$default(AdditiveSynthesisMath additiveSynthesisMath, AdditiveSynthState additiveSynthState, int i, int i2, double d, int i3, Object obj) {
        if ((i3 & 8) != 0) {
            d = 0.0d;
        }
        return additiveSynthesisMath.standingField(additiveSynthState, i, i2, d);
    }

    public final float[] standingField(AdditiveSynthState state, int sampleRateHz, int pointCount, double phaseRadians) {
        Intrinsics.checkNotNullParameter(state, "state");
        if (pointCount <= 0) {
            return new float[0];
        }
        float[] targetAmplitudes = targetAmplitudes(state, sampleRateHz);
        float[] fArr = new float[pointCount];
        for (int i = 0; i < pointCount; i++) {
            double d = 0.0d;
            double d2 = pointCount == 1 ? 0.0d : i / (pointCount - 1);
            int length = targetAmplitudes.length;
            int i2 = 0;
            while (i2 < length) {
                int i3 = i2 + 1;
                double d3 = i3;
                d += targetAmplitudes[i2] * Math.sin(3.141592653589793d * d3 * d2) * Math.cos(d3 * phaseRadians);
                i2 = i3;
            }
            fArr[i] = (float) d;
        }
        return fArr;
    }
}
