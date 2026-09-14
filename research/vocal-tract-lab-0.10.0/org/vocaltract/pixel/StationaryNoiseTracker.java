package org.vocaltract.pixel;

import java.util.Arrays;
import java.util.Iterator;
import java.util.NoSuchElementException;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.collections.IntIterator;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000(\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0002\b\u0004\n\u0002\u0010\u0014\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0004\n\u0002\u0018\u0002\n\u0002\b\u0005\b\u0000\u0018\u00002\u00020\u0001B\u0019\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0004\u001a\u00020\u0003¢\u0006\u0004\b\u0005\u0010\u0006J \u0010\u000e\u001a\u00020\u000f2\u0006\u0010\u0010\u001a\u00020\b2\u0006\u0010\u0011\u001a\u00020\n2\b\b\u0002\u0010\u0012\u001a\u00020\nJ\u0010\u0010\u0013\u001a\u00020\b2\u0006\u0010\u0014\u001a\u00020\bH\u0002R\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0004\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0007\u001a\u00020\bX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\t\u001a\u00020\nX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u000b\u001a\u00020\u0003X\u0082\u000e¢\u0006\u0002\n\u0000R\u0010\u0010\f\u001a\u0004\u0018\u00010\bX\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\u0003X\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/StationaryNoiseTracker;", "", "bins", "", "bandCount", "<init>", "(II)V", "noise", "", "initialized", "", "observations", "previousBands", "changedHold", "update", "Lorg/vocaltract/pixel/NoiseEstimate;", "power", "voiced", "forceLearning", "summarizeBands", "values"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class StationaryNoiseTracker {
    private final int bandCount;
    private final int bins;
    private int changedHold;
    private boolean initialized;
    private final float[] noise;
    private int observations;
    private float[] previousBands;

    public StationaryNoiseTracker(int i, int i2) {
        this.bins = i;
        this.bandCount = i2;
        this.noise = new float[i];
    }

    public /* synthetic */ StationaryNoiseTracker(int i, int i2, int i3, DefaultConstructorMarker defaultConstructorMarker) {
        this(i, (i3 & 2) != 0 ? 8 : i2);
    }

    public static /* synthetic */ NoiseEstimate update$default(StationaryNoiseTracker stationaryNoiseTracker, float[] fArr, boolean z, boolean z2, int i, Object obj) {
        if ((i & 4) != 0) {
            z2 = false;
        }
        return stationaryNoiseTracker.update(fArr, z, z2);
    }

    public final NoiseEstimate update(float[] power, boolean voiced, boolean forceLearning) {
        float abs;
        int i;
        boolean z;
        String str;
        Intrinsics.checkNotNullParameter(power, "power");
        if (power.length != this.bins) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        float[] summarizeBands = summarizeBands(power);
        float[] fArr = this.previousBands;
        if (!this.initialized) {
            int length = power.length;
            for (int i2 = 0; i2 < length; i2++) {
                this.noise[i2] = Math.max(power[i2], 1.0E-12f);
            }
            this.initialized = true;
            this.observations = 1;
        } else {
            float f = forceLearning ? 0.68f : !voiced ? 0.9f : 0.9985f;
            float f2 = forceLearning ? 0.55f : 0.78f;
            int length2 = power.length;
            for (int i3 = 0; i3 < length2; i3++) {
                float max = Math.max(power[i3], 1.0E-12f);
                float[] fArr2 = this.noise;
                float f3 = fArr2[i3];
                float f4 = max <= f3 ? f2 : f;
                fArr2[i3] = (f3 * f4) + ((1.0f - f4) * max);
            }
            this.observations++;
        }
        float[] fArr3 = new float[this.bins];
        int length3 = power.length;
        double d = 0.0d;
        double d2 = 0.0d;
        for (int i4 = 0; i4 < length3; i4++) {
            float f5 = power[i4];
            float max2 = Math.max(Math.max(f5 - (this.noise[i4] * 1.08f), f5 * 0.01f), 1.0E-12f);
            fArr3[i4] = max2;
            d += max2;
            d2 += this.noise[i4];
        }
        float coerceIn = RangesKt.coerceIn((float) (Math.log10(Math.max(1.0E-12d, d / Math.max(1.0E-12d, d2))) * 10.0d), -20.0f, 60.0f);
        float log10 = (float) (Math.log10(Math.max(1.0E-12d, d2 / this.bins)) * 10.0d);
        float[] summarizeBands2 = summarizeBands(this.noise);
        if (fArr == null) {
            abs = 0.0f;
        } else {
            Iterator<Integer> it = ArraysKt.getIndices(summarizeBands).iterator();
            if (!it.hasNext()) {
                throw new NoSuchElementException();
            }
            IntIterator intIterator = (IntIterator) it;
            int nextInt = intIterator.nextInt();
            abs = Math.abs(summarizeBands[nextInt] - fArr[nextInt]);
            while (it.hasNext()) {
                int nextInt2 = intIterator.nextInt();
                abs = Math.max(abs, Math.abs(summarizeBands[nextInt2] - fArr[nextInt2]));
            }
        }
        float coerceIn2 = RangesKt.coerceIn(this.observations / 36.0f, 0.0f, 1.0f);
        if ((!voiced || forceLearning) && coerceIn2 > 0.55f && abs > 2.0f) {
            i = 12;
            z = true;
        } else {
            int i5 = this.changedHold;
            if (i5 > 0) {
                z = true;
                i = i5 - 1;
            } else {
                z = true;
                i = 0;
            }
        }
        this.changedHold = i;
        if (!voiced || forceLearning) {
            float[] copyOf = Arrays.copyOf(summarizeBands2, summarizeBands2.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            this.previousBands = copyOf;
        }
        if (coerceIn2 < 0.65f) {
            str = "learning";
        } else {
            str = this.changedHold > 0 ? "background_changed" : "stable";
        }
        return new NoiseEstimate(fArr3, coerceIn, log10, str, coerceIn2, this.changedHold > 0 ? z : false, summarizeBands2);
    }

    private final float[] summarizeBands(float[] values) {
        int i = this.bandCount;
        float[] fArr = new float[i];
        int i2 = 0;
        while (i2 < i) {
            int length = values.length * i2;
            int i3 = this.bandCount;
            int i4 = length / i3;
            int i5 = i2 + 1;
            double d = 0.0d;
            for (int i6 = i4; i6 < Math.min(Math.max(i4 + 1, (values.length * i5) / i3), values.length); i6++) {
                d += values[i6];
            }
            fArr[i2] = (float) (Math.log10(Math.max(1.0E-12d, d / Math.max(1, r4 - i4))) * 10.0d);
            i2 = i5;
        }
        return fArr;
    }
}
