package org.vocaltract.pixel;

import java.util.Arrays;
import java.util.List;
import kotlin.Metadata;
import kotlin.collections.CollectionsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;
import kotlin.uuid.Uuid;

/* compiled from: VocalAcousticsState.kt */
@Metadata(d1 = {"\u00006\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u0014\n\u0002\b\u0003\n\u0002\u0010\u0007\n\u0002\b\u0002\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0002\n\u0002\u0010 \n\u0002\b\u001d\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B]\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0003\u0012\u0006\u0010\u0006\u001a\u00020\u0007\u0012\u0006\u0010\b\u001a\u00020\u0007\u0012\b\b\u0002\u0010\t\u001a\u00020\n\u0012\b\b\u0002\u0010\u000b\u001a\u00020\f\u0012\b\b\u0002\u0010\r\u001a\u00020\n\u0012\u000e\b\u0002\u0010\u000e\u001a\b\u0012\u0004\u0012\u00020\n0\u000f¢\u0006\u0004\b\u0010\u0010\u0011J\t\u0010 \u001a\u00020\u0003HÆ\u0003J\t\u0010!\u001a\u00020\u0003HÆ\u0003J\t\u0010\"\u001a\u00020\u0003HÆ\u0003J\t\u0010#\u001a\u00020\u0007HÆ\u0003J\t\u0010$\u001a\u00020\u0007HÆ\u0003J\t\u0010%\u001a\u00020\nHÆ\u0003J\t\u0010&\u001a\u00020\fHÆ\u0003J\t\u0010'\u001a\u00020\nHÆ\u0003J\u000f\u0010(\u001a\b\u0012\u0004\u0012\u00020\n0\u000fHÆ\u0003Ji\u0010)\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00032\b\b\u0002\u0010\u0006\u001a\u00020\u00072\b\b\u0002\u0010\b\u001a\u00020\u00072\b\b\u0002\u0010\t\u001a\u00020\n2\b\b\u0002\u0010\u000b\u001a\u00020\f2\b\b\u0002\u0010\r\u001a\u00020\n2\u000e\b\u0002\u0010\u000e\u001a\b\u0012\u0004\u0012\u00020\n0\u000fHÆ\u0001J\u0013\u0010*\u001a\u00020\f2\b\u0010+\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010,\u001a\u00020-HÖ\u0001J\t\u0010.\u001a\u00020\nHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013R\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0013R\u0011\u0010\u0005\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0015\u0010\u0013R\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u0016\u0010\u0017R\u0011\u0010\b\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\u0018\u0010\u0017R\u0011\u0010\t\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b\u0019\u0010\u001aR\u0011\u0010\u000b\u001a\u00020\f¢\u0006\b\n\u0000\u001a\u0004\b\u001b\u0010\u001cR\u0011\u0010\r\u001a\u00020\n¢\u0006\b\n\u0000\u001a\u0004\b\u001d\u0010\u001aR\u0017\u0010\u000e\u001a\b\u0012\u0004\u0012\u00020\n0\u000f¢\u0006\b\n\u0000\u001a\u0004\b\u001e\u0010\u001f"}, d2 = {"Lorg/vocaltract/pixel/TractPosterior;", "", "sectionPosition", "", "areaCm2", "coefficients", "confidence", "", "relativeAreaStd", "interpretation", "", "abstained", "", "abstentionReason", "modeLabels", "", "<init>", "([F[F[FFFLjava/lang/String;ZLjava/lang/String;Ljava/util/List;)V", "getSectionPosition", "()[F", "getAreaCm2", "getCoefficients", "getConfidence", "()F", "getRelativeAreaStd", "getInterpretation", "()Ljava/lang/String;", "getAbstained", "()Z", "getAbstentionReason", "getModeLabels", "()Ljava/util/List;", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "copy", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class TractPosterior {
    private final boolean abstained;
    private final String abstentionReason;
    private final float[] areaCm2;
    private final float[] coefficients;
    private final float confidence;
    private final String interpretation;
    private final List<String> modeLabels;
    private final float relativeAreaStd;
    private final float[] sectionPosition;

    /* renamed from: component1, reason: from getter */
    public final float[] getSectionPosition() {
        return this.sectionPosition;
    }

    /* renamed from: component2, reason: from getter */
    public final float[] getAreaCm2() {
        return this.areaCm2;
    }

    /* renamed from: component3, reason: from getter */
    public final float[] getCoefficients() {
        return this.coefficients;
    }

    /* renamed from: component4, reason: from getter */
    public final float getConfidence() {
        return this.confidence;
    }

    /* renamed from: component5, reason: from getter */
    public final float getRelativeAreaStd() {
        return this.relativeAreaStd;
    }

    /* renamed from: component6, reason: from getter */
    public final String getInterpretation() {
        return this.interpretation;
    }

    /* renamed from: component7, reason: from getter */
    public final boolean getAbstained() {
        return this.abstained;
    }

    /* renamed from: component8, reason: from getter */
    public final String getAbstentionReason() {
        return this.abstentionReason;
    }

    public final List<String> component9() {
        return this.modeLabels;
    }

    public final TractPosterior copy(float[] sectionPosition, float[] areaCm2, float[] coefficients, float confidence, float relativeAreaStd, String interpretation, boolean abstained, String abstentionReason, List<String> modeLabels) {
        Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
        Intrinsics.checkNotNullParameter(areaCm2, "areaCm2");
        Intrinsics.checkNotNullParameter(coefficients, "coefficients");
        Intrinsics.checkNotNullParameter(interpretation, "interpretation");
        Intrinsics.checkNotNullParameter(abstentionReason, "abstentionReason");
        Intrinsics.checkNotNullParameter(modeLabels, "modeLabels");
        return new TractPosterior(sectionPosition, areaCm2, coefficients, confidence, relativeAreaStd, interpretation, abstained, abstentionReason, modeLabels);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof TractPosterior)) {
            return false;
        }
        TractPosterior tractPosterior = (TractPosterior) other;
        return Intrinsics.areEqual(this.sectionPosition, tractPosterior.sectionPosition) && Intrinsics.areEqual(this.areaCm2, tractPosterior.areaCm2) && Intrinsics.areEqual(this.coefficients, tractPosterior.coefficients) && Float.compare(this.confidence, tractPosterior.confidence) == 0 && Float.compare(this.relativeAreaStd, tractPosterior.relativeAreaStd) == 0 && Intrinsics.areEqual(this.interpretation, tractPosterior.interpretation) && this.abstained == tractPosterior.abstained && Intrinsics.areEqual(this.abstentionReason, tractPosterior.abstentionReason) && Intrinsics.areEqual(this.modeLabels, tractPosterior.modeLabels);
    }

    public int hashCode() {
        return (((((((((((((((Arrays.hashCode(this.sectionPosition) * 31) + Arrays.hashCode(this.areaCm2)) * 31) + Arrays.hashCode(this.coefficients)) * 31) + Float.hashCode(this.confidence)) * 31) + Float.hashCode(this.relativeAreaStd)) * 31) + this.interpretation.hashCode()) * 31) + Boolean.hashCode(this.abstained)) * 31) + this.abstentionReason.hashCode()) * 31) + this.modeLabels.hashCode();
    }

    public String toString() {
        return "TractPosterior(sectionPosition=" + Arrays.toString(this.sectionPosition) + ", areaCm2=" + Arrays.toString(this.areaCm2) + ", coefficients=" + Arrays.toString(this.coefficients) + ", confidence=" + this.confidence + ", relativeAreaStd=" + this.relativeAreaStd + ", interpretation=" + this.interpretation + ", abstained=" + this.abstained + ", abstentionReason=" + this.abstentionReason + ", modeLabels=" + this.modeLabels + ")";
    }

    public TractPosterior(float[] sectionPosition, float[] areaCm2, float[] coefficients, float f, float f2, String interpretation, boolean z, String abstentionReason, List<String> modeLabels) {
        Intrinsics.checkNotNullParameter(sectionPosition, "sectionPosition");
        Intrinsics.checkNotNullParameter(areaCm2, "areaCm2");
        Intrinsics.checkNotNullParameter(coefficients, "coefficients");
        Intrinsics.checkNotNullParameter(interpretation, "interpretation");
        Intrinsics.checkNotNullParameter(abstentionReason, "abstentionReason");
        Intrinsics.checkNotNullParameter(modeLabels, "modeLabels");
        this.sectionPosition = sectionPosition;
        this.areaCm2 = areaCm2;
        this.coefficients = coefficients;
        this.confidence = f;
        this.relativeAreaStd = f2;
        this.interpretation = interpretation;
        this.abstained = z;
        this.abstentionReason = abstentionReason;
        this.modeLabels = modeLabels;
    }

    public final float[] getSectionPosition() {
        return this.sectionPosition;
    }

    public final float[] getAreaCm2() {
        return this.areaCm2;
    }

    public final float[] getCoefficients() {
        return this.coefficients;
    }

    public final float getConfidence() {
        return this.confidence;
    }

    public final float getRelativeAreaStd() {
        return this.relativeAreaStd;
    }

    public /* synthetic */ TractPosterior(float[] fArr, float[] fArr2, float[] fArr3, float f, float f2, String str, boolean z, String str2, List list, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(fArr, fArr2, fArr3, f, f2, (i & 32) != 0 ? "Audio-conditioned estimate; anatomy is not observed" : str, (i & 64) != 0 ? false : z, (i & Uuid.SIZE_BITS) != 0 ? "none" : str2, (i & 256) != 0 ? CollectionsKt.emptyList() : list);
    }

    public final String getInterpretation() {
        return this.interpretation;
    }

    public final boolean getAbstained() {
        return this.abstained;
    }

    public final String getAbstentionReason() {
        return this.abstentionReason;
    }

    public final List<String> getModeLabels() {
        return this.modeLabels;
    }
}
