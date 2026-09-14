package org.vocaltract.pixel;

import java.util.Arrays;
import java.util.List;
import kotlin.Metadata;
import kotlin.io.ConstantsKt;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: VocalAcousticsState.kt */
@Metadata(d1 = {"\u0000>\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000b\n\u0000\n\u0002\u0010\u0007\n\u0002\b\u0003\n\u0002\u0010 \n\u0002\b\u0002\n\u0002\u0018\u0002\n\u0002\b\u0003\n\u0002\u0010\u000e\n\u0002\b\b\n\u0002\u0010\u0014\n\u0002\b5\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001BÓ\u0001\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\b\u0010\u0004\u001a\u0004\u0018\u00010\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\b\u0010\u0007\u001a\u0004\u0018\u00010\u0005\u0012\f\u0010\b\u001a\b\u0012\u0004\u0012\u00020\u00050\t\u0012\f\u0010\n\u001a\b\u0012\u0004\u0012\u00020\u00050\t\u0012\f\u0010\u000b\u001a\b\u0012\u0004\u0012\u00020\f0\t\u0012\u0006\u0010\r\u001a\u00020\u0005\u0012\n\b\u0002\u0010\u000e\u001a\u0004\u0018\u00010\u0005\u0012\b\b\u0002\u0010\u000f\u001a\u00020\u0010\u0012\b\b\u0002\u0010\u0011\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0012\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0013\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0014\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0015\u001a\u00020\u0010\u0012\b\b\u0002\u0010\u0016\u001a\u00020\u0005\u0012\b\b\u0002\u0010\u0017\u001a\u00020\u0003\u0012\b\b\u0002\u0010\u0018\u001a\u00020\u0019\u0012\u000e\b\u0002\u0010\u001a\u001a\b\u0012\u0004\u0012\u00020\u00050\t¢\u0006\u0004\b\u001b\u0010\u001cJ\t\u00107\u001a\u00020\u0003HÆ\u0003J\u0010\u00108\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010 J\t\u00109\u001a\u00020\u0005HÆ\u0003J\u0010\u0010:\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010 J\u000f\u0010;\u001a\b\u0012\u0004\u0012\u00020\u00050\tHÆ\u0003J\u000f\u0010<\u001a\b\u0012\u0004\u0012\u00020\u00050\tHÆ\u0003J\u000f\u0010=\u001a\b\u0012\u0004\u0012\u00020\f0\tHÆ\u0003J\t\u0010>\u001a\u00020\u0005HÆ\u0003J\u0010\u0010?\u001a\u0004\u0018\u00010\u0005HÆ\u0003¢\u0006\u0002\u0010 J\t\u0010@\u001a\u00020\u0010HÆ\u0003J\t\u0010A\u001a\u00020\u0003HÆ\u0003J\t\u0010B\u001a\u00020\u0005HÆ\u0003J\t\u0010C\u001a\u00020\u0005HÆ\u0003J\t\u0010D\u001a\u00020\u0005HÆ\u0003J\t\u0010E\u001a\u00020\u0010HÆ\u0003J\t\u0010F\u001a\u00020\u0005HÆ\u0003J\t\u0010G\u001a\u00020\u0003HÆ\u0003J\t\u0010H\u001a\u00020\u0019HÆ\u0003J\u000f\u0010I\u001a\b\u0012\u0004\u0012\u00020\u00050\tHÆ\u0003Jê\u0001\u0010J\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\n\b\u0002\u0010\u0004\u001a\u0004\u0018\u00010\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00052\n\b\u0002\u0010\u0007\u001a\u0004\u0018\u00010\u00052\u000e\b\u0002\u0010\b\u001a\b\u0012\u0004\u0012\u00020\u00050\t2\u000e\b\u0002\u0010\n\u001a\b\u0012\u0004\u0012\u00020\u00050\t2\u000e\b\u0002\u0010\u000b\u001a\b\u0012\u0004\u0012\u00020\f0\t2\b\b\u0002\u0010\r\u001a\u00020\u00052\n\b\u0002\u0010\u000e\u001a\u0004\u0018\u00010\u00052\b\b\u0002\u0010\u000f\u001a\u00020\u00102\b\b\u0002\u0010\u0011\u001a\u00020\u00032\b\b\u0002\u0010\u0012\u001a\u00020\u00052\b\b\u0002\u0010\u0013\u001a\u00020\u00052\b\b\u0002\u0010\u0014\u001a\u00020\u00052\b\b\u0002\u0010\u0015\u001a\u00020\u00102\b\b\u0002\u0010\u0016\u001a\u00020\u00052\b\b\u0002\u0010\u0017\u001a\u00020\u00032\b\b\u0002\u0010\u0018\u001a\u00020\u00192\u000e\b\u0002\u0010\u001a\u001a\b\u0012\u0004\u0012\u00020\u00050\tHÆ\u0001¢\u0006\u0002\u0010KJ\u0013\u0010L\u001a\u00020\u00032\b\u0010M\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010N\u001a\u00020OHÖ\u0001J\t\u0010P\u001a\u00020\u0010HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u001d\u0010\u001eR\u0015\u0010\u0004\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010!\u001a\u0004\b\u001f\u0010 R\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\"\u0010#R\u0015\u0010\u0007\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010!\u001a\u0004\b$\u0010 R\u0017\u0010\b\u001a\b\u0012\u0004\u0012\u00020\u00050\t¢\u0006\b\n\u0000\u001a\u0004\b%\u0010&R\u0017\u0010\n\u001a\b\u0012\u0004\u0012\u00020\u00050\t¢\u0006\b\n\u0000\u001a\u0004\b'\u0010&R\u0017\u0010\u000b\u001a\b\u0012\u0004\u0012\u00020\f0\t¢\u0006\b\n\u0000\u001a\u0004\b(\u0010&R\u0011\u0010\r\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b)\u0010#R\u0015\u0010\u000e\u001a\u0004\u0018\u00010\u0005¢\u0006\n\n\u0002\u0010!\u001a\u0004\b*\u0010 R\u0011\u0010\u000f\u001a\u00020\u0010¢\u0006\b\n\u0000\u001a\u0004\b+\u0010,R\u0011\u0010\u0011\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b-\u0010\u001eR\u0011\u0010\u0012\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b.\u0010#R\u0011\u0010\u0013\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b/\u0010#R\u0011\u0010\u0014\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b0\u0010#R\u0011\u0010\u0015\u001a\u00020\u0010¢\u0006\b\n\u0000\u001a\u0004\b1\u0010,R\u0011\u0010\u0016\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b2\u0010#R\u0011\u0010\u0017\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b3\u0010\u001eR\u0011\u0010\u0018\u001a\u00020\u0019¢\u0006\b\n\u0000\u001a\u0004\b4\u00105R\u0017\u0010\u001a\u001a\b\u0012\u0004\u0012\u00020\u00050\t¢\u0006\b\n\u0000\u001a\u0004\b6\u0010&"}, d2 = {"Lorg/vocaltract/pixel/AcousticEstimate;", "", "voiced", "", "f0Hz", "", "f0Confidence", "f0StdHz", "formantsHz", "", "formantStdHz", "harmonics", "Lorg/vocaltract/pixel/HarmonicEstimate;", "rms", "rawF0Hz", "pitchDecision", "", "pitchRejected", "harmonicity", "snrDb", "noiseFloorDb", "noiseState", "noiseConfidence", "backgroundChanged", "noiseBandsDb", "", "formantCandidatesHz", "<init>", "(ZLjava/lang/Float;FLjava/lang/Float;Ljava/util/List;Ljava/util/List;Ljava/util/List;FLjava/lang/Float;Ljava/lang/String;ZFFFLjava/lang/String;FZ[FLjava/util/List;)V", "getVoiced", "()Z", "getF0Hz", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getF0Confidence", "()F", "getF0StdHz", "getFormantsHz", "()Ljava/util/List;", "getFormantStdHz", "getHarmonics", "getRms", "getRawF0Hz", "getPitchDecision", "()Ljava/lang/String;", "getPitchRejected", "getHarmonicity", "getSnrDb", "getNoiseFloorDb", "getNoiseState", "getNoiseConfidence", "getBackgroundChanged", "getNoiseBandsDb", "()[F", "getFormantCandidatesHz", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "component9", "component10", "component11", "component12", "component13", "component14", "component15", "component16", "component17", "component18", "component19", "copy", "(ZLjava/lang/Float;FLjava/lang/Float;Ljava/util/List;Ljava/util/List;Ljava/util/List;FLjava/lang/Float;Ljava/lang/String;ZFFFLjava/lang/String;FZ[FLjava/util/List;)Lorg/vocaltract/pixel/AcousticEstimate;", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class AcousticEstimate {
    private final boolean backgroundChanged;
    private final float f0Confidence;
    private final Float f0Hz;
    private final Float f0StdHz;
    private final List<Float> formantCandidatesHz;
    private final List<Float> formantStdHz;
    private final List<Float> formantsHz;
    private final float harmonicity;
    private final List<HarmonicEstimate> harmonics;
    private final float[] noiseBandsDb;
    private final float noiseConfidence;
    private final float noiseFloorDb;
    private final String noiseState;
    private final String pitchDecision;
    private final boolean pitchRejected;
    private final Float rawF0Hz;
    private final float rms;
    private final float snrDb;
    private final boolean voiced;

    /* renamed from: component1, reason: from getter */
    public final boolean getVoiced() {
        return this.voiced;
    }

    /* renamed from: component10, reason: from getter */
    public final String getPitchDecision() {
        return this.pitchDecision;
    }

    /* renamed from: component11, reason: from getter */
    public final boolean getPitchRejected() {
        return this.pitchRejected;
    }

    /* renamed from: component12, reason: from getter */
    public final float getHarmonicity() {
        return this.harmonicity;
    }

    /* renamed from: component13, reason: from getter */
    public final float getSnrDb() {
        return this.snrDb;
    }

    /* renamed from: component14, reason: from getter */
    public final float getNoiseFloorDb() {
        return this.noiseFloorDb;
    }

    /* renamed from: component15, reason: from getter */
    public final String getNoiseState() {
        return this.noiseState;
    }

    /* renamed from: component16, reason: from getter */
    public final float getNoiseConfidence() {
        return this.noiseConfidence;
    }

    /* renamed from: component17, reason: from getter */
    public final boolean getBackgroundChanged() {
        return this.backgroundChanged;
    }

    /* renamed from: component18, reason: from getter */
    public final float[] getNoiseBandsDb() {
        return this.noiseBandsDb;
    }

    public final List<Float> component19() {
        return this.formantCandidatesHz;
    }

    /* renamed from: component2, reason: from getter */
    public final Float getF0Hz() {
        return this.f0Hz;
    }

    /* renamed from: component3, reason: from getter */
    public final float getF0Confidence() {
        return this.f0Confidence;
    }

    /* renamed from: component4, reason: from getter */
    public final Float getF0StdHz() {
        return this.f0StdHz;
    }

    public final List<Float> component5() {
        return this.formantsHz;
    }

    public final List<Float> component6() {
        return this.formantStdHz;
    }

    public final List<HarmonicEstimate> component7() {
        return this.harmonics;
    }

    /* renamed from: component8, reason: from getter */
    public final float getRms() {
        return this.rms;
    }

    /* renamed from: component9, reason: from getter */
    public final Float getRawF0Hz() {
        return this.rawF0Hz;
    }

    public final AcousticEstimate copy(boolean voiced, Float f0Hz, float f0Confidence, Float f0StdHz, List<Float> formantsHz, List<Float> formantStdHz, List<HarmonicEstimate> harmonics, float rms, Float rawF0Hz, String pitchDecision, boolean pitchRejected, float harmonicity, float snrDb, float noiseFloorDb, String noiseState, float noiseConfidence, boolean backgroundChanged, float[] noiseBandsDb, List<Float> formantCandidatesHz) {
        Intrinsics.checkNotNullParameter(formantsHz, "formantsHz");
        Intrinsics.checkNotNullParameter(formantStdHz, "formantStdHz");
        Intrinsics.checkNotNullParameter(harmonics, "harmonics");
        Intrinsics.checkNotNullParameter(pitchDecision, "pitchDecision");
        Intrinsics.checkNotNullParameter(noiseState, "noiseState");
        Intrinsics.checkNotNullParameter(noiseBandsDb, "noiseBandsDb");
        Intrinsics.checkNotNullParameter(formantCandidatesHz, "formantCandidatesHz");
        return new AcousticEstimate(voiced, f0Hz, f0Confidence, f0StdHz, formantsHz, formantStdHz, harmonics, rms, rawF0Hz, pitchDecision, pitchRejected, harmonicity, snrDb, noiseFloorDb, noiseState, noiseConfidence, backgroundChanged, noiseBandsDb, formantCandidatesHz);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof AcousticEstimate)) {
            return false;
        }
        AcousticEstimate acousticEstimate = (AcousticEstimate) other;
        return this.voiced == acousticEstimate.voiced && Intrinsics.areEqual((Object) this.f0Hz, (Object) acousticEstimate.f0Hz) && Float.compare(this.f0Confidence, acousticEstimate.f0Confidence) == 0 && Intrinsics.areEqual((Object) this.f0StdHz, (Object) acousticEstimate.f0StdHz) && Intrinsics.areEqual(this.formantsHz, acousticEstimate.formantsHz) && Intrinsics.areEqual(this.formantStdHz, acousticEstimate.formantStdHz) && Intrinsics.areEqual(this.harmonics, acousticEstimate.harmonics) && Float.compare(this.rms, acousticEstimate.rms) == 0 && Intrinsics.areEqual((Object) this.rawF0Hz, (Object) acousticEstimate.rawF0Hz) && Intrinsics.areEqual(this.pitchDecision, acousticEstimate.pitchDecision) && this.pitchRejected == acousticEstimate.pitchRejected && Float.compare(this.harmonicity, acousticEstimate.harmonicity) == 0 && Float.compare(this.snrDb, acousticEstimate.snrDb) == 0 && Float.compare(this.noiseFloorDb, acousticEstimate.noiseFloorDb) == 0 && Intrinsics.areEqual(this.noiseState, acousticEstimate.noiseState) && Float.compare(this.noiseConfidence, acousticEstimate.noiseConfidence) == 0 && this.backgroundChanged == acousticEstimate.backgroundChanged && Intrinsics.areEqual(this.noiseBandsDb, acousticEstimate.noiseBandsDb) && Intrinsics.areEqual(this.formantCandidatesHz, acousticEstimate.formantCandidatesHz);
    }

    public int hashCode() {
        int hashCode = Boolean.hashCode(this.voiced) * 31;
        Float f = this.f0Hz;
        int hashCode2 = (((hashCode + (f == null ? 0 : f.hashCode())) * 31) + Float.hashCode(this.f0Confidence)) * 31;
        Float f2 = this.f0StdHz;
        int hashCode3 = (((((((((hashCode2 + (f2 == null ? 0 : f2.hashCode())) * 31) + this.formantsHz.hashCode()) * 31) + this.formantStdHz.hashCode()) * 31) + this.harmonics.hashCode()) * 31) + Float.hashCode(this.rms)) * 31;
        Float f3 = this.rawF0Hz;
        return ((((((((((((((((((((hashCode3 + (f3 != null ? f3.hashCode() : 0)) * 31) + this.pitchDecision.hashCode()) * 31) + Boolean.hashCode(this.pitchRejected)) * 31) + Float.hashCode(this.harmonicity)) * 31) + Float.hashCode(this.snrDb)) * 31) + Float.hashCode(this.noiseFloorDb)) * 31) + this.noiseState.hashCode()) * 31) + Float.hashCode(this.noiseConfidence)) * 31) + Boolean.hashCode(this.backgroundChanged)) * 31) + Arrays.hashCode(this.noiseBandsDb)) * 31) + this.formantCandidatesHz.hashCode();
    }

    public String toString() {
        return "AcousticEstimate(voiced=" + this.voiced + ", f0Hz=" + this.f0Hz + ", f0Confidence=" + this.f0Confidence + ", f0StdHz=" + this.f0StdHz + ", formantsHz=" + this.formantsHz + ", formantStdHz=" + this.formantStdHz + ", harmonics=" + this.harmonics + ", rms=" + this.rms + ", rawF0Hz=" + this.rawF0Hz + ", pitchDecision=" + this.pitchDecision + ", pitchRejected=" + this.pitchRejected + ", harmonicity=" + this.harmonicity + ", snrDb=" + this.snrDb + ", noiseFloorDb=" + this.noiseFloorDb + ", noiseState=" + this.noiseState + ", noiseConfidence=" + this.noiseConfidence + ", backgroundChanged=" + this.backgroundChanged + ", noiseBandsDb=" + Arrays.toString(this.noiseBandsDb) + ", formantCandidatesHz=" + this.formantCandidatesHz + ")";
    }

    public AcousticEstimate(boolean z, Float f, float f2, Float f3, List<Float> formantsHz, List<Float> formantStdHz, List<HarmonicEstimate> harmonics, float f4, Float f5, String pitchDecision, boolean z2, float f6, float f7, float f8, String noiseState, float f9, boolean z3, float[] noiseBandsDb, List<Float> formantCandidatesHz) {
        Intrinsics.checkNotNullParameter(formantsHz, "formantsHz");
        Intrinsics.checkNotNullParameter(formantStdHz, "formantStdHz");
        Intrinsics.checkNotNullParameter(harmonics, "harmonics");
        Intrinsics.checkNotNullParameter(pitchDecision, "pitchDecision");
        Intrinsics.checkNotNullParameter(noiseState, "noiseState");
        Intrinsics.checkNotNullParameter(noiseBandsDb, "noiseBandsDb");
        Intrinsics.checkNotNullParameter(formantCandidatesHz, "formantCandidatesHz");
        this.voiced = z;
        this.f0Hz = f;
        this.f0Confidence = f2;
        this.f0StdHz = f3;
        this.formantsHz = formantsHz;
        this.formantStdHz = formantStdHz;
        this.harmonics = harmonics;
        this.rms = f4;
        this.rawF0Hz = f5;
        this.pitchDecision = pitchDecision;
        this.pitchRejected = z2;
        this.harmonicity = f6;
        this.snrDb = f7;
        this.noiseFloorDb = f8;
        this.noiseState = noiseState;
        this.noiseConfidence = f9;
        this.backgroundChanged = z3;
        this.noiseBandsDb = noiseBandsDb;
        this.formantCandidatesHz = formantCandidatesHz;
    }

    public final boolean getVoiced() {
        return this.voiced;
    }

    public final Float getF0Hz() {
        return this.f0Hz;
    }

    public final float getF0Confidence() {
        return this.f0Confidence;
    }

    public final Float getF0StdHz() {
        return this.f0StdHz;
    }

    public final List<Float> getFormantsHz() {
        return this.formantsHz;
    }

    public final List<Float> getFormantStdHz() {
        return this.formantStdHz;
    }

    public final List<HarmonicEstimate> getHarmonics() {
        return this.harmonics;
    }

    public final float getRms() {
        return this.rms;
    }

    public final Float getRawF0Hz() {
        return this.rawF0Hz;
    }

    public /* synthetic */ AcousticEstimate(boolean z, Float f, float f2, Float f3, List list, List list2, List list3, float f4, Float f5, String str, boolean z2, float f6, float f7, float f8, String str2, float f9, boolean z3, float[] fArr, List list4, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(z, f, f2, f3, list, list2, list3, f4, (i & 256) != 0 ? f : f5, (i & ConstantsKt.MINIMUM_BLOCK_SIZE) != 0 ? "unprocessed" : str, (i & 1024) != 0 ? false : z2, (i & 2048) != 0 ? 0.0f : f6, (i & ConstantsKt.DEFAULT_BLOCK_SIZE) != 0 ? 0.0f : f7, (i & ConstantsKt.DEFAULT_BUFFER_SIZE) != 0 ? -120.0f : f8, (i & 16384) != 0 ? "uninitialized" : str2, (32768 & i) != 0 ? 0.0f : f9, (65536 & i) != 0 ? false : z3, (131072 & i) != 0 ? new float[0] : fArr, (i & 262144) != 0 ? list : list4);
    }

    public final String getPitchDecision() {
        return this.pitchDecision;
    }

    public final boolean getPitchRejected() {
        return this.pitchRejected;
    }

    public final float getHarmonicity() {
        return this.harmonicity;
    }

    public final float getSnrDb() {
        return this.snrDb;
    }

    public final float getNoiseFloorDb() {
        return this.noiseFloorDb;
    }

    public final String getNoiseState() {
        return this.noiseState;
    }

    public final float getNoiseConfidence() {
        return this.noiseConfidence;
    }

    public final boolean getBackgroundChanged() {
        return this.backgroundChanged;
    }

    public final float[] getNoiseBandsDb() {
        return this.noiseBandsDb;
    }

    public final List<Float> getFormantCandidatesHz() {
        return this.formantCandidatesHz;
    }
}
