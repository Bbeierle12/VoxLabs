package org.vocaltract.pixel;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.collections.CollectionsKt;
import kotlin.collections.IntIterator;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: SharedTractModel.kt */
@Metadata(d1 = {"\u0000$\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0013\n\u0002\b\u0006\n\u0002\u0010\u0011\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\u0018\u00002\u00020\u0001B\u0017\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003¢\u0006\u0004\b\u0005\u0010\u0006J\u000e\u0010\f\u001a\u00020\r2\u0006\u0010\u000e\u001a\u00020\u000fR\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0007\u0010\bR\u0016\u0010\t\u001a\b\u0012\u0004\u0012\u00020\u00030\nX\u0082\u0004¢\u0006\u0004\n\u0002\u0010\u000b"}, d2 = {"Lorg/vocaltract/pixel/TubeGrid;", "", "lengths", "", "frequencies", "<init>", "([D[D)V", "getFrequencies", "()[D", "cs", "", "[[D", "response", "Lorg/vocaltract/pixel/TractResponse;", "areas", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class TubeGrid {
    private final double[][] cs;
    private final double[] frequencies;
    private final double[] lengths;

    public TubeGrid(double[] lengths, double[] frequencies) {
        int i;
        int i2;
        Intrinsics.checkNotNullParameter(lengths, "lengths");
        Intrinsics.checkNotNullParameter(frequencies, "frequencies");
        double[] copyOf = Arrays.copyOf(lengths, lengths.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.lengths = copyOf;
        double[] copyOf2 = Arrays.copyOf(frequencies, frequencies.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
        this.frequencies = copyOf2;
        int length = frequencies.length;
        double[][] dArr = new double[length][];
        for (int i3 = 0; i3 < length; i3++) {
            dArr[i3] = new double[lengths.length * 4];
        }
        this.cs = dArr;
        if (!(lengths.length == 0)) {
            int length2 = lengths.length;
            while (i < length2) {
                double d = lengths[i];
                i = (Double.isInfinite(d) || Double.isNaN(d) || d <= 0.0d) ? 0 : i + 1;
            }
            if (!(frequencies.length == 0)) {
                int length3 = frequencies.length;
                while (i2 < length3) {
                    double d2 = frequencies[i2];
                    i2 = (Double.isInfinite(d2) || Double.isNaN(d2) || d2 <= 0.0d) ? 0 : i2 + 1;
                }
                Iterable until = RangesKt.until(1, frequencies.length);
                if (!(until instanceof Collection) || !((Collection) until).isEmpty()) {
                    Iterator it = until.iterator();
                    while (it.hasNext()) {
                        int nextInt = ((IntIterator) it).nextInt();
                        if (frequencies[nextInt] <= frequencies[nextInt - 1]) {
                            throw new IllegalArgumentException("Failed requirement.".toString());
                        }
                    }
                }
                int length4 = frequencies.length;
                for (int i4 = 0; i4 < length4; i4++) {
                    int i5 = 0;
                    for (int length5 = lengths.length; i5 < length5; length5 = length5) {
                        double d3 = frequencies[i4];
                        double d4 = lengths[i5] / 1000.0d;
                        double d5 = ((6.283185307179586d * d3) * d4) / 350.0d;
                        double sqrt = Math.sqrt(d3 / 1000.0d) * (-0.3d) * d4;
                        int i6 = i5 * 4;
                        this.cs[i4][i6] = Math.cos(d5) * Math.cosh(sqrt);
                        this.cs[i4][i6 + 1] = (-Math.sin(d5)) * Math.sinh(sqrt);
                        this.cs[i4][i6 + 2] = Math.sin(d5) * Math.cosh(sqrt);
                        this.cs[i4][i6 + 3] = Math.cos(d5) * Math.sinh(sqrt);
                        i5++;
                    }
                }
                return;
            }
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }

    public final double[] getFrequencies() {
        return this.frequencies;
    }

    public final TractResponse response(float[] areas) {
        int i;
        double d;
        Intrinsics.checkNotNullParameter(areas, "areas");
        int i2 = 1;
        if (areas.length == this.lengths.length + 1) {
            int length = areas.length;
            while (i < length) {
                float f = areas[i];
                i = (Float.isInfinite(f) || Float.isNaN(f) || f <= 0.0f) ? 0 : i + 1;
            }
            int length2 = this.lengths.length;
            double[] dArr = new double[length2];
            int i3 = 0;
            while (true) {
                d = 400.75d;
                if (i3 >= length2) {
                    break;
                }
                dArr[i3] = 400.75d / ((areas[i3] + areas[r11]) * 5.0E-5d);
                i3++;
            }
            double[] dArr2 = this.frequencies;
            int length3 = dArr2.length;
            double[] dArr3 = new double[length3];
            int length4 = dArr2.length;
            int i4 = 0;
            while (i4 < length4) {
                int length5 = this.lengths.length;
                double d2 = 1.0d;
                int i5 = 0;
                double d3 = 0.0d;
                double d4 = 0.0d;
                double d5 = 0.0d;
                while (i5 < length5) {
                    double[] dArr4 = this.cs[i4];
                    int i6 = i5 * 4;
                    double d6 = dArr4[i6];
                    double d7 = dArr4[i6 + 1];
                    double d8 = dArr4[i6 + 2];
                    double d9 = dArr4[i6 + 3];
                    double d10 = dArr[i5];
                    double d11 = -d9;
                    double d12 = d11 / d10;
                    double d13 = d8 / d10;
                    double d14 = d11 * d10;
                    double d15 = d8 * d10;
                    double d16 = (((d3 * d6) - (d4 * d7)) + (d2 * d12)) - (d5 * d13);
                    double d17 = (d3 * d7) + (d4 * d6) + (d13 * d2) + (d12 * d5);
                    double d18 = (((d3 * d14) - (d4 * d15)) + (d2 * d6)) - (d5 * d7);
                    d5 = (d3 * d15) + (d4 * d14) + (d2 * d7) + (d5 * d6);
                    i5++;
                    d3 = d16;
                    d4 = d17;
                    d2 = d18;
                }
                double last = ArraysKt.last(areas) * 1.0E-4d;
                double sqrt = ((this.frequencies[i4] * 6.283185307179586d) * Math.sqrt(last / 3.141592653589793d)) / 350;
                double d19 = d / last;
                double d20 = (((d19 * 0.25d) * sqrt) * sqrt) / (1 + ((0.25d * sqrt) * sqrt));
                double d21 = d19 * 0.61d * sqrt;
                dArr3[i4] = (-20) * Math.log10(Math.max(1.0E-12d, Math.hypot(((d3 * d20) - (d4 * d21)) + d2, (d3 * d21) + (d4 * d20) + d5)));
                i4++;
                i2 = 1;
                dArr = dArr;
                d = 400.75d;
            }
            int i7 = i2;
            Double maxOrNull = ArraysKt.maxOrNull(dArr3);
            Intrinsics.checkNotNull(maxOrNull);
            double doubleValue = maxOrNull.doubleValue();
            double[] dArr5 = new double[length3];
            for (int i8 = 0; i8 < length3; i8++) {
                dArr5[i8] = dArr3[i8] - doubleValue;
            }
            ArrayList arrayList = new ArrayList();
            int lastIndex = ArraysKt.getLastIndex(dArr5);
            for (int i9 = i7; i9 < lastIndex; i9++) {
                double d22 = dArr5[i9];
                int i10 = i9 - 1;
                double d23 = dArr5[i10];
                if (d22 > d23) {
                    int i11 = i9 + 1;
                    double d24 = dArr5[i11];
                    if (d22 >= d24) {
                        double d25 = (d23 - (2 * d22)) + d24;
                        double coerceIn = Math.abs(d25) > 1.0E-12d ? RangesKt.coerceIn(((dArr5[i10] - dArr5[i11]) * 0.5d) / d25, -0.5d, 0.5d) : 0.0d;
                        double[] dArr6 = this.frequencies;
                        arrayList.add(Double.valueOf(dArr6[i9] + (coerceIn * (dArr6[i11] - dArr6[i10]) * 0.5d)));
                    }
                }
            }
            double[] dArr7 = this.frequencies;
            double[] copyOf = Arrays.copyOf(dArr7, dArr7.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            return new TractResponse(copyOf, dArr5, CollectionsKt.toDoubleArray(CollectionsKt.take(arrayList, 6)));
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }
}
