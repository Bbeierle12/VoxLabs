package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: AcousticMatchOverlay.kt */
@Metadata(d1 = {"\u0000&\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010\b\n\u0002\b\u0005\n\u0002\u0010\u000e\n\u0002\b#\b\u0086\b\u0018\u0000 02\u00020\u0001:\u00010B[\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\b\u0010\u0006\u001a\u0004\u0018\u00010\u0007\u0012\b\u0010\b\u001a\u0004\u0018\u00010\u0005\u0012\b\u0010\t\u001a\u0004\u0018\u00010\u0007\u0012\b\u0010\n\u001a\u0004\u0018\u00010\u0005\u0012\b\u0010\u000b\u001a\u0004\u0018\u00010\u0005\u0012\u0006\u0010\f\u001a\u00020\r\u0012\b\b\u0002\u0010\u000e\u001a\u00020\r¢\u0006\u0004\b\u000f\u0010\u0010J\t\u0010!\u001a\u00020\u0003HÆ\u0003J\t\u0010\"\u001a\u00020\u0005HÆ\u0003J\u0010\u0010#\u001a\u0004\u0018\u00010\u0007HÆ\u0003¢\u0006\u0002\u0010\u0016J\u0010\u0010$\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010\u0019J\u0010\u0010%\u001a\u0004\u0018\u00010\u0007HÆ\u0003¢\u0006\u0002\u0010\u0016J\u0010\u0010&\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010\u0019J\u0010\u0010'\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010\u0019J\t\u0010(\u001a\u00020\rHÆ\u0003J\t\u0010)\u001a\u00020\rHÆ\u0003Jr\u0010*\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\n\b\u0002\u0010\u0006\u001a\u0004\u0018\u00010\u00072\n\b\u0002\u0010\b\u001a\u0004\u0018\u00010\u00052\n\b\u0002\u0010\t\u001a\u0004\u0018\u00010\u00072\n\b\u0002\u0010\n\u001a\u0004\u0018\u00010\u00052\n\b\u0002\u0010\u000b\u001a\u0004\u0018\u00010\u00052\b\b\u0002\u0010\f\u001a\u00020\r2\b\b\u0002\u0010\u000e\u001a\u00020\rHÆ\u0001¢\u0006\u0002\u0010+J\u0013\u0010,\u001a\u00020\u00032\b\u0010-\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010.\u001a\u00020\u0007HÖ\u0001J\t\u0010/\u001a\u00020\rHÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0011\u0010\u0012R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0013\u0010\u0014R\u0015\u0010\u0006\u001a\u0004\u0018\u00010\u0007¢\u0006\n\n\u0002\u0010\u0017\u001a\u0004\b\u0015\u0010\u0016R\u0015\u0010\b\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010\u001a\u001a\u0004\b\u0018\u0010\u0019R\u0015\u0010\t\u001a\u0004\u0018\u00010\u0007¢\u0006\n\n\u0002\u0010\u0017\u001a\u0004\b\u001b\u0010\u0016R\u0015\u0010\n\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010\u001a\u001a\u0004\b\u001c\u0010\u0019R\u0015\u0010\u000b\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010\u001a\u001a\u0004\b\u001d\u0010\u0019R\u0011\u0010\f\u001a\u00020\r¢\u0006\b\n\u0000\u001a\u0004\b\u001e\u0010\u001fR\u0011\u0010\u000e\u001a\u00020\r¢\u0006\b\n\u0000\u001a\u0004\b \u0010\u001f"}, d2 = {"Lorg/vocaltract/pixel/AcousticMatchSummary;", "", "active", "", "score", "", "resonanceNumber", "", "resonanceHz", "harmonicNumber", "harmonicHz", "detuningHz", "label", "", "interpretation", "<init>", "(ZFLjava/lang/Integer;Ljava/lang/Float;Ljava/lang/Integer;Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/String;Ljava/lang/String;)V", "getActive", "()Z", "getScore", "()F", "getResonanceNumber", "()Ljava/lang/Integer;", "Ljava/lang/Integer;", "getResonanceHz", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getHarmonicNumber", "getHarmonicHz", "getDetuningHz", "getLabel", "()Ljava/lang/String;", "getInterpretation", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "copy", "(ZFLjava/lang/Integer;Ljava/lang/Float;Ljava/lang/Integer;Ljava/lang/Float;Ljava/lang/Float;Ljava/lang/String;Ljava/lang/String;)Lorg/vocaltract/pixel/AcousticMatchSummary;", "equals", "other", "hashCode", "toString", "Companion"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class AcousticMatchSummary {
    public static final String GLOBAL_TINT_INTERPRETATION = "Global harmonic/resonance match tint; not a spatial FEM pressure field";
    private final boolean active;
    private final Float detuningHz;
    private final Float harmonicHz;
    private final Integer harmonicNumber;
    private final String interpretation;
    private final String label;
    private final Float resonanceHz;
    private final Integer resonanceNumber;
    private final float score;

    /* renamed from: Companion, reason: from kotlin metadata */
    public static final Companion INSTANCE = new Companion(null);
    private static final AcousticMatchSummary NONE = new AcousticMatchSummary(false, 0.0f, null, null, null, null, null, "No reliable harmonic/resonance match", null, 256, null);

    /* renamed from: component1, reason: from getter */
    public final boolean getActive() {
        return this.active;
    }

    /* renamed from: component2, reason: from getter */
    public final float getScore() {
        return this.score;
    }

    /* renamed from: component3, reason: from getter */
    public final Integer getResonanceNumber() {
        return this.resonanceNumber;
    }

    /* renamed from: component4, reason: from getter */
    public final Float getResonanceHz() {
        return this.resonanceHz;
    }

    /* renamed from: component5, reason: from getter */
    public final Integer getHarmonicNumber() {
        return this.harmonicNumber;
    }

    /* renamed from: component6, reason: from getter */
    public final Float getHarmonicHz() {
        return this.harmonicHz;
    }

    /* renamed from: component7, reason: from getter */
    public final Float getDetuningHz() {
        return this.detuningHz;
    }

    /* renamed from: component8, reason: from getter */
    public final String getLabel() {
        return this.label;
    }

    /* renamed from: component9, reason: from getter */
    public final String getInterpretation() {
        return this.interpretation;
    }

    public final AcousticMatchSummary copy(boolean active, float score, Integer resonanceNumber, Float resonanceHz, Integer harmonicNumber, Float harmonicHz, Float detuningHz, String label, String interpretation) {
        Intrinsics.checkNotNullParameter(label, "label");
        Intrinsics.checkNotNullParameter(interpretation, "interpretation");
        return new AcousticMatchSummary(active, score, resonanceNumber, resonanceHz, harmonicNumber, harmonicHz, detuningHz, label, interpretation);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof AcousticMatchSummary)) {
            return false;
        }
        AcousticMatchSummary acousticMatchSummary = (AcousticMatchSummary) other;
        return this.active == acousticMatchSummary.active && Float.compare(this.score, acousticMatchSummary.score) == 0 && Intrinsics.areEqual(this.resonanceNumber, acousticMatchSummary.resonanceNumber) && Intrinsics.areEqual((Object) this.resonanceHz, (Object) acousticMatchSummary.resonanceHz) && Intrinsics.areEqual(this.harmonicNumber, acousticMatchSummary.harmonicNumber) && Intrinsics.areEqual((Object) this.harmonicHz, (Object) acousticMatchSummary.harmonicHz) && Intrinsics.areEqual((Object) this.detuningHz, (Object) acousticMatchSummary.detuningHz) && Intrinsics.areEqual(this.label, acousticMatchSummary.label) && Intrinsics.areEqual(this.interpretation, acousticMatchSummary.interpretation);
    }

    public int hashCode() {
        int hashCode = ((Boolean.hashCode(this.active) * 31) + Float.hashCode(this.score)) * 31;
        Integer num = this.resonanceNumber;
        int hashCode2 = (hashCode + (num == null ? 0 : num.hashCode())) * 31;
        Float f = this.resonanceHz;
        int hashCode3 = (hashCode2 + (f == null ? 0 : f.hashCode())) * 31;
        Integer num2 = this.harmonicNumber;
        int hashCode4 = (hashCode3 + (num2 == null ? 0 : num2.hashCode())) * 31;
        Float f2 = this.harmonicHz;
        int hashCode5 = (hashCode4 + (f2 == null ? 0 : f2.hashCode())) * 31;
        Float f3 = this.detuningHz;
        return ((((hashCode5 + (f3 != null ? f3.hashCode() : 0)) * 31) + this.label.hashCode()) * 31) + this.interpretation.hashCode();
    }

    public String toString() {
        return "AcousticMatchSummary(active=" + this.active + ", score=" + this.score + ", resonanceNumber=" + this.resonanceNumber + ", resonanceHz=" + this.resonanceHz + ", harmonicNumber=" + this.harmonicNumber + ", harmonicHz=" + this.harmonicHz + ", detuningHz=" + this.detuningHz + ", label=" + this.label + ", interpretation=" + this.interpretation + ")";
    }

    public AcousticMatchSummary(boolean z, float f, Integer num, Float f2, Integer num2, Float f3, Float f4, String label, String interpretation) {
        Intrinsics.checkNotNullParameter(label, "label");
        Intrinsics.checkNotNullParameter(interpretation, "interpretation");
        this.active = z;
        this.score = f;
        this.resonanceNumber = num;
        this.resonanceHz = f2;
        this.harmonicNumber = num2;
        this.harmonicHz = f3;
        this.detuningHz = f4;
        this.label = label;
        this.interpretation = interpretation;
    }

    public final boolean getActive() {
        return this.active;
    }

    public final float getScore() {
        return this.score;
    }

    public final Integer getResonanceNumber() {
        return this.resonanceNumber;
    }

    public final Float getResonanceHz() {
        return this.resonanceHz;
    }

    public final Integer getHarmonicNumber() {
        return this.harmonicNumber;
    }

    public final Float getHarmonicHz() {
        return this.harmonicHz;
    }

    public final Float getDetuningHz() {
        return this.detuningHz;
    }

    public final String getLabel() {
        return this.label;
    }

    public /* synthetic */ AcousticMatchSummary(boolean z, float f, Integer num, Float f2, Integer num2, Float f3, Float f4, String str, String str2, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(z, f, num, f2, num2, f3, f4, str, (i & 256) != 0 ? GLOBAL_TINT_INTERPRETATION : str2);
    }

    public final String getInterpretation() {
        return this.interpretation;
    }

    /* compiled from: AcousticMatchOverlay.kt */
    @Metadata(d1 = {"\u0000\u001a\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0000\n\u0002\u0018\u0002\n\u0002\b\u0002\b\u0086\u0003\u0018\u00002\u00020\u0001B\t\b\u0002¢\u0006\u0004\b\u0002\u0010\u0003R\u000e\u0010\u0004\u001a\u00020\u0005X\u0086T¢\u0006\u0002\n\u0000R\u0011\u0010\u0006\u001a\u00020\u0007¢\u0006\b\n\u0000\u001a\u0004\b\b\u0010\t"}, d2 = {"Lorg/vocaltract/pixel/AcousticMatchSummary$Companion;", "", "<init>", "()V", "GLOBAL_TINT_INTERPRETATION", "", "NONE", "Lorg/vocaltract/pixel/AcousticMatchSummary;", "getNONE", "()Lorg/vocaltract/pixel/AcousticMatchSummary;"}, k = 1, mv = {2, 0, 0}, xi = 48)
    public static final class Companion {
        public /* synthetic */ Companion(DefaultConstructorMarker defaultConstructorMarker) {
            this();
        }

        private Companion() {
        }

        public final AcousticMatchSummary getNONE() {
            return AcousticMatchSummary.NONE;
        }
    }
}
