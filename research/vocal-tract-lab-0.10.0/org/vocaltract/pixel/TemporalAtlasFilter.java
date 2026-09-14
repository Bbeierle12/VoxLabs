package org.vocaltract.pixel;

import java.util.Arrays;
import java.util.Collection;
import java.util.Iterator;
import kotlin.Metadata;
import kotlin.collections.ArraysKt;
import kotlin.collections.IntIterator;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.ranges.RangesKt;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000*\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0018\u0002\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0004\n\u0002\u0010\u0007\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\u000b\b\u0000\u0018\u00002\u00020\u0001B\u0019\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0004\u001a\u00020\u0005¢\u0006\u0004\b\u0006\u0010\u0007J\u001e\u0010\u000b\u001a\u00020\f2\u0006\u0010\r\u001a\u00020\u00052\u0006\u0010\u000e\u001a\u00020\n2\u0006\u0010\u000f\u001a\u00020\u0010R\u000e\u0010\u0002\u001a\u00020\u0003X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\b\u001a\u00020\u0005X\u0082\u0004¢\u0006\u0002\n\u0000R\u000e\u0010\t\u001a\u00020\nX\u0082\u000e¢\u0006\u0002\n\u0000"}, d2 = {"Lorg/vocaltract/pixel/TemporalAtlasFilter;", "", "asset", "Lorg/vocaltract/pixel/ReducedModelAsset;", "seed", "", "<init>", "(Lorg/vocaltract/pixel/ReducedModelAsset;[F)V", "coefficients", "confidence", "", "update", "Lorg/vocaltract/pixel/TemporalAtlasEstimate;", "raw", "evidenceConfidence", "voiced", ""}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final class TemporalAtlasFilter {
    private final ReducedModelAsset asset;
    private final float[] coefficients;
    private float confidence;

    public TemporalAtlasFilter(ReducedModelAsset asset, float[] seed) {
        Intrinsics.checkNotNullParameter(asset, "asset");
        Intrinsics.checkNotNullParameter(seed, "seed");
        this.asset = asset;
        float[] copyOf = Arrays.copyOf(seed, seed.length);
        Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
        this.coefficients = copyOf;
        if (seed.length == asset.getAreaModes().length) {
            Iterable indices = ArraysKt.getIndices(seed);
            if ((indices instanceof Collection) && ((Collection) indices).isEmpty()) {
                return;
            }
            Iterator it = indices.iterator();
            while (it.hasNext()) {
                int nextInt = ((IntIterator) it).nextInt();
                float f = seed[nextInt];
                if (!Float.isInfinite(f) && !Float.isNaN(f)) {
                    float f2 = this.asset.getCoefficientLimitsSd()[nextInt][0];
                    float f3 = this.asset.getCoefficientLimitsSd()[nextInt][1];
                    float f4 = seed[nextInt];
                    if (f2 <= f4 && f4 <= f3) {
                    }
                }
            }
            return;
        }
        throw new IllegalArgumentException("Failed requirement.".toString());
    }

    public /* synthetic */ TemporalAtlasFilter(ReducedModelAsset reducedModelAsset, float[] fArr, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(reducedModelAsset, (i & 2) != 0 ? new float[reducedModelAsset.getAreaModes().length] : fArr);
    }

    public final TemporalAtlasEstimate update(float[] raw, float evidenceConfidence, boolean voiced) {
        Intrinsics.checkNotNullParameter(raw, "raw");
        if (raw.length != this.coefficients.length) {
            throw new IllegalArgumentException("Failed requirement.".toString());
        }
        if (voiced && evidenceConfidence >= this.asset.getAbstentionConfidenceThreshold()) {
            float coerceIn = RangesKt.coerceIn((0.38f * evidenceConfidence) + 0.12f, 0.12f, 0.46f);
            int length = this.coefficients.length;
            for (int i = 0; i < length; i++) {
                float[] fArr = this.asset.getCoefficientLimitsSd()[i];
                float[] fArr2 = this.coefficients;
                float f = fArr2[i];
                fArr2[i] = RangesKt.coerceIn(f + ((raw[i] - f) * coerceIn), fArr[0], fArr[1]);
            }
            float f2 = this.confidence;
            this.confidence = f2 + ((evidenceConfidence - f2) * 0.35f);
            float[] fArr3 = this.coefficients;
            float[] copyOf = Arrays.copyOf(fArr3, fArr3.length);
            Intrinsics.checkNotNullExpressionValue(copyOf, "copyOf(...)");
            return new TemporalAtlasEstimate(copyOf, this.confidence, false, "audio_evidence");
        }
        int length2 = this.coefficients.length;
        for (int i2 = 0; i2 < length2; i2++) {
            float[] fArr4 = this.coefficients;
            fArr4[i2] = fArr4[i2] * 0.9f;
        }
        this.confidence *= 0.82f;
        String str = !voiced ? "unvoiced" : "insufficient_evidence";
        float[] fArr5 = this.coefficients;
        float[] copyOf2 = Arrays.copyOf(fArr5, fArr5.length);
        Intrinsics.checkNotNullExpressionValue(copyOf2, "copyOf(...)");
        return new TemporalAtlasEstimate(copyOf2, this.confidence, true, str);
    }
}
