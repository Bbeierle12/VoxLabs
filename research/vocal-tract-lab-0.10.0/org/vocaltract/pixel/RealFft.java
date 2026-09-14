package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.IntProgression;
import kotlin.ranges.RangesKt;

/* compiled from: RealtimeDspPipeline.kt */
@Metadata(d1 = {"\u0000\u0018\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u0002\n\u0000\n\u0002\u0010\u0013\n\u0000\bÀ\u0002\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0016\u0010\u0004\u001a\u00020\u00052\u0006\u0010\u0006\u001a\u00020\u00072\u0006\u0010\b\u001a\u00020\u0007"}, d2 = {"Lorg/vocaltract/pixel/RealFft;", "", "<init>", "()V", "transform", "", "real", "", "imaginary"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class RealFft {
    public static final RealFft INSTANCE = new RealFft();

    private RealFft() {
    }

    public final void transform(double[] real, double[] imaginary) {
        Intrinsics.checkNotNullParameter(real, "real");
        Intrinsics.checkNotNullParameter(imaginary, "imaginary");
        if (real.length == imaginary.length && real.length > 0) {
            if ((real.length & (real.length - 1)) == 0) {
                int length = real.length;
                int i = 0;
                int i2 = 0;
                for (int i3 = 1; i3 < length; i3++) {
                    int i4 = length >> 1;
                    while ((i2 & i4) != 0) {
                        i2 ^= i4;
                        i4 >>= 1;
                    }
                    i2 ^= i4;
                    if (i3 < i2) {
                        double d = real[i3];
                        real[i3] = real[i2];
                        real[i2] = d;
                        double d2 = imaginary[i3];
                        imaginary[i3] = imaginary[i2];
                        imaginary[i2] = d2;
                    }
                }
                int i5 = 2;
                while (i5 <= length) {
                    double d3 = (-6.283185307179586d) / i5;
                    double cos = Math.cos(d3);
                    double sin = Math.sin(d3);
                    IntProgression step = RangesKt.step(RangesKt.until(i, length), i5);
                    int first = step.getFirst();
                    int last = step.getLast();
                    int step2 = step.getStep();
                    if ((step2 > 0 && first <= last) || (step2 < 0 && last <= first)) {
                        while (true) {
                            int i6 = i5 / 2;
                            double d4 = 1.0d;
                            double d5 = 0.0d;
                            while (i < i6) {
                                int i7 = first + i;
                                int i8 = i7 + i6;
                                double d6 = real[i8];
                                double d7 = imaginary[i8];
                                double d8 = (d6 * d4) - (d7 * d5);
                                double d9 = (d6 * d5) + (d7 * d4);
                                real[i8] = real[i7] - d8;
                                imaginary[i8] = imaginary[i7] - d9;
                                real[i7] = real[i7] + d8;
                                imaginary[i7] = imaginary[i7] + d9;
                                double d10 = (d4 * cos) - (d5 * sin);
                                d5 = (d5 * cos) + (d4 * sin);
                                i++;
                                d4 = d10;
                            }
                            if (first != last) {
                                first += step2;
                                i = 0;
                            }
                        }
                    }
                    i5 <<= 1;
                    i = 0;
                }
                return;
            }
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }
}
