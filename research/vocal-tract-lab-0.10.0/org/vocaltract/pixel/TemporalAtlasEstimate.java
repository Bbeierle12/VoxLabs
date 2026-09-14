package org.vocaltract.pixel;

import java.util.Arrays;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: InferenceStabilizers.kt */
@Metadata(d1 = {"\u0000*\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0014\n\u0000\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0012\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B'\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0007\u0012\u0006\u0010\b\u001a\u00020\t¢\u0006\u0004\b\n\u0010\u000bJ\t\u0010\u0014\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0015\u001a\u00020\u0005HÆ\u0003J\t\u0010\u0016\u001a\u00020\u0007HÆ\u0003J\t\u0010\u0017\u001a\u00020\tHÆ\u0003J1\u0010\u0018\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00072\b\b\u0002\u0010\b\u001a\u00020\tHÆ\u0001J\u0013\u0010\u0019\u001a\u00020\u00072\b\u0010\u001a\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u001b\u001a\u00020\u001cHÖ\u0001J\t\u0010\u001d\u001a\u00020\tHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\f\u0010\rR\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000fR\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011R\u0011\u0010\b\u001a\u00020\t¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013"}, d2 = {"Lorg/vocaltract/pixel/TemporalAtlasEstimate;", "", "coefficients", "", "confidence", "", "abstained", "", "reason", "", "<init>", "([FFZLjava/lang/String;)V", "getCoefficients", "()[F", "getConfidence", "()F", "getAbstained", "()Z", "getReason", "()Ljava/lang/String;", "component1", "component2", "component3", "component4", "copy", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class TemporalAtlasEstimate {
    private final boolean abstained;
    private final float[] coefficients;
    private final float confidence;
    private final String reason;

    public static /* synthetic */ TemporalAtlasEstimate copy$default(TemporalAtlasEstimate temporalAtlasEstimate, float[] fArr, float f, boolean z, String str, int i, Object obj) {
        if ((i & 1) != 0) {
            fArr = temporalAtlasEstimate.coefficients;
        }
        if ((i & 2) != 0) {
            f = temporalAtlasEstimate.confidence;
        }
        if ((i & 4) != 0) {
            z = temporalAtlasEstimate.abstained;
        }
        if ((i & 8) != 0) {
            str = temporalAtlasEstimate.reason;
        }
        return temporalAtlasEstimate.copy(fArr, f, z, str);
    }

    /* renamed from: component1, reason: from getter */
    public final float[] getCoefficients() {
        return this.coefficients;
    }

    /* renamed from: component2, reason: from getter */
    public final float getConfidence() {
        return this.confidence;
    }

    /* renamed from: component3, reason: from getter */
    public final boolean getAbstained() {
        return this.abstained;
    }

    /* renamed from: component4, reason: from getter */
    public final String getReason() {
        return this.reason;
    }

    public final TemporalAtlasEstimate copy(float[] coefficients, float confidence, boolean abstained, String reason) {
        Intrinsics.checkNotNullParameter(coefficients, "coefficients");
        Intrinsics.checkNotNullParameter(reason, "reason");
        return new TemporalAtlasEstimate(coefficients, confidence, abstained, reason);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TemporalAtlasEstimate)) {
            return false;
        }
        TemporalAtlasEstimate temporalAtlasEstimate = (TemporalAtlasEstimate) other;
        return Intrinsics.areEqual(this.coefficients, temporalAtlasEstimate.coefficients) && Float.compare(this.confidence, temporalAtlasEstimate.confidence) == 0 && this.abstained == temporalAtlasEstimate.abstained && Intrinsics.areEqual(this.reason, temporalAtlasEstimate.reason);
    }

    public int hashCode() {
        return (((((Arrays.hashCode(this.coefficients) * 31) + Float.hashCode(this.confidence)) * 31) + Boolean.hashCode(this.abstained)) * 31) + this.reason.hashCode();
    }

    public String toString() {
        return "TemporalAtlasEstimate(coefficients=" + Arrays.toString(this.coefficients) + ", confidence=" + this.confidence + ", abstained=" + this.abstained + ", reason=" + this.reason + ")";
    }

    public TemporalAtlasEstimate(float[] coefficients, float f, boolean z, String reason) {
        Intrinsics.checkNotNullParameter(coefficients, "coefficients");
        Intrinsics.checkNotNullParameter(reason, "reason");
        this.coefficients = coefficients;
        this.confidence = f;
        this.abstained = z;
        this.reason = reason;
    }

    public final float[] getCoefficients() {
        return this.coefficients;
    }

    public final float getConfidence() {
        return this.confidence;
    }

    public final boolean getAbstained() {
        return this.abstained;
    }

    public final String getReason() {
        return this.reason;
    }
}
