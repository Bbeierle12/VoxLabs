package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: AdditiveSynthesis.kt */
@Metadata(d1 = {"\u00008\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\b\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0006\n\u0002\u0010\u0014\n\u0002\b\u0003\n\u0002\u0010\u0006\n\u0002\b\u0003\n\u0002\u0010\u0002\n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0002\u0018\u0000 \u00192\u00020\u0001:\u0001\u0019B#\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0004\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0006\u001a\u00020\u0005¢\u0006\u0004\b\u0007\u0010\bJ\u0006\u0010\u0013\u001a\u00020\u0014J\u0016\u0010\u0015\u001a\u00020\u00142\u0006\u0010\u0016\u001a\u00020\u00172\u0006\u0010\u0018\u001a\u00020\fR\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\t\u0010\nR\u000e\u0010\u000b\u001a\u00020\fX\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\r\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000e\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u000f\u001a\u00020\u0010X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\u0011\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000R\u000e\u0010\u0012\u001a\u00020\u0010X\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSignalGenerator;", "", "sampleRateHz", "", "gainSmoothingMs", "", "frequencySmoothingMs", "<init>", "(IFF)V", "getSampleRateHz", "()I", "currentAmplitudes", "", "gainAlpha", "frequencyAlpha", "nyquistHz", "", "basePhase", "currentFundamentalHz", "reset", "", "render", "state", "Lorg/vocaltract/pixel/AdditiveSynthState;", "output", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class AdditiveSignalGenerator {
    private static final Companion Companion = new Companion(null);

    @Deprecated
    public static final float SILENCE_EPSILON = 1.0E-7f;
    private double basePhase;
    private final float[] currentAmplitudes;
    private double currentFundamentalHz;
    private final float frequencyAlpha;
    private final float gainAlpha;
    private final double nyquistHz;
    private final int sampleRateHz;

    public AdditiveSignalGenerator(int i, float f, float f2) {
        this.sampleRateHz = i;
        this.currentAmplitudes = new float[16];
        Companion companion = Companion;
        this.gainAlpha = companion.smoothingAlpha(f, i);
        this.frequencyAlpha = companion.smoothingAlpha(f2, i);
        this.nyquistHz = i / 2.0d;
        this.currentFundamentalHz = Double.NaN;
        if (8000 > i || i >= 192001) {
            throw new IllegalArgumentException(("Unsupported synthesis sample rate: " + i).toString());
        }
    }

    public /* synthetic */ AdditiveSignalGenerator(int i, float f, float f2, int i2, DefaultConstructorMarker defaultConstructorMarker) {
        this(i, (i2 & 2) != 0 ? 12.0f : f, (i2 & 4) != 0 ? 8.0f : f2);
    }

    public final int getSampleRateHz() {
        return this.sampleRateHz;
    }

    public final void reset() {
        ArraysKt.fill$default(this.currentAmplitudes, 0.0f, 0, 0, 6, (Object) null);
        this.currentFundamentalHz = Double.NaN;
        this.basePhase = 0.0d;
    }

    public final void render(AdditiveSynthState state, float[] output) {
        Intrinsics.checkNotNullParameter(state, "state");
        Intrinsics.checkNotNullParameter(output, "output");
        float[] targetAmplitudes = AdditiveSynthesisMath.INSTANCE.targetAmplitudes(state, this.sampleRateHz);
        double fundamentalHz = state.getFundamentalHz();
        double d = this.currentFundamentalHz;
        if (Double.isInfinite(d) || Double.isNaN(d)) {
            this.currentFundamentalHz = fundamentalHz;
        }
        int length = output.length;
        int i = 0;
        while (i < length) {
            double d2 = this.currentFundamentalHz;
            this.currentFundamentalHz = d2 + ((fundamentalHz - d2) * this.frequencyAlpha);
            int length2 = this.currentAmplitudes.length;
            double d3 = 0.0d;
            int i2 = 0;
            while (i2 < length2) {
                int i3 = i2 + 1;
                int i4 = i;
                double d4 = i3;
                double d5 = fundamentalHz;
                boolean z = this.currentFundamentalHz * d4 < this.nyquistHz;
                float f = z ? targetAmplitudes[i2] : 0.0f;
                float[] fArr = this.currentAmplitudes;
                float f2 = fArr[i2];
                int i5 = length;
                float f3 = f2 + ((f - f2) * this.gainAlpha);
                fArr[i2] = f3;
                if (z && Math.abs(f3) > 1.0E-7f) {
                    d3 += this.currentAmplitudes[i2] * Math.sin(this.basePhase * d4);
                }
                i2 = i3;
                i = i4;
                fundamentalHz = d5;
                length = i5;
            }
            int i6 = length;
            double d6 = fundamentalHz;
            int i7 = i;
            output[i7] = RangesKt.coerceIn((float) d3, -1.0f, 1.0f);
            double d7 = this.basePhase + ((this.currentFundamentalHz * 6.283185307179586d) / this.sampleRateHz);
            this.basePhase = d7;
            if (d7 >= 6.283185307179586d) {
                this.basePhase = d7 % 6.283185307179586d;
            }
            i = i7 + 1;
            fundamentalHz = d6;
            length = i6;
        }
    }

    /* compiled from: AdditiveSynthesis.kt */
    @Metadata(d1 = {"\u0000\u0018\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010\b\b\u0082\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003J\u0016\u0010\u0006\u001a\u00020\u00052\u0006\u0010\u0007\u001a\u00020\u00052\u0006\u0010\b\u001a\u00020\tR\u000e\u0010\u0004\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/AdditiveSignalGenerator$Companion;", "", "<init>", "()V", "SILENCE_EPSILON", "", "smoothingAlpha", "milliseconds", "sampleRateHz", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
    private static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public final float smoothingAlpha(float milliseconds, int sampleRateHz) {
            return RangesKt.coerceIn((float) (1.0d - Math.exp((-1.0d) / ((((Float.isInfinite(milliseconds) || Float.isNaN(milliseconds)) ? 12.0f : RangesKt.coerceAtLeast(milliseconds, 0.1f)) * sampleRateHz) / 1000.0f))), 1.0E-6f, 1.0f);
        }
    }
}
