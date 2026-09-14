package org.vocaltract.pixel;

import java.util.List;
import kotlin.Metadata;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: SingerCalibration.kt */
@Metadata(d1 = {"\u0000.\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0000\n\u0002\u0010\b\n\u0002\b\u0002\n\u0002\u0010\u0007\n\u0000\n\u0002\u0010 \n\u0002\b\u001e\n\u0002\u0010\u000b\n\u0002\b\u0003\b\u0086\b\u0018\u00002\u00020\u0001BO\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0005\u0012\u0006\u0010\u0006\u001a\u00020\u0005\u0012\b\u0010\u0007\u001a\u0004\u0018\u00010\b\u0012\f\u0010\t\u001a\b\u0012\u0004\u0012\u00020\b0\n\u0012\u0006\u0010\u000b\u001a\u00020\b\u0012\u0006\u0010\f\u001a\u00020\b\u0012\u0006\u0010\r\u001a\u00020\u0005¢\u0006\u0004\b\u000e\u0010\u000fJ\t\u0010\u001e\u001a\u00020\u0003HÆ\u0003J\t\u0010\u001f\u001a\u00020\u0005HÆ\u0003J\t\u0010 \u001a\u00020\u0005HÆ\u0003J\u0010\u0010!\u001a\u0004\u0018\u00010\bHÆ\u0003¢\u0006\u0002\u0010\u0016J\u000f\u0010\"\u001a\b\u0012\u0004\u0012\u00020\b0\nHÆ\u0003J\t\u0010#\u001a\u00020\bHÆ\u0003J\t\u0010$\u001a\u00020\bHÆ\u0003J\t\u0010%\u001a\u00020\u0005HÆ\u0003Jf\u0010&\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00052\b\b\u0002\u0010\u0006\u001a\u00020\u00052\n\b\u0002\u0010\u0007\u001a\u0004\u0018\u00010\b2\u000e\b\u0002\u0010\t\u001a\b\u0012\u0004\u0012\u00020\b0\n2\b\b\u0002\u0010\u000b\u001a\u00020\b2\b\b\u0002\u0010\f\u001a\u00020\b2\b\b\u0002\u0010\r\u001a\u00020\u0005HÆ\u0001¢\u0006\u0002\u0010'J\u0013\u0010(\u001a\u00020)2\b\u0010*\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010+\u001a\u00020\u0005HÖ\u0001J\t\u0010,\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011R\u0011\u0010\u0004\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0012\u0010\u0013R\u0011\u0010\u0006\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u0014\u0010\u0013R\u0015\u0010\u0007\u001a\u0004\u0018\u00010\b¢\u0006\n\n\u0002\u0010\u0017\u001a\u0004\b\u0015\u0010\u0016R\u0017\u0010\t\u001a\b\u0012\u0004\u0012\u00020\b0\n¢\u0006\b\n\u0000\u001a\u0004\b\u0018\u0010\u0019R\u0011\u0010\u000b\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u001a\u0010\u001bR\u0011\u0010\f\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u001c\u0010\u001bR\u0011\u0010\r\u001a\u00020\u0005¢\u0006\b\n\u0000\u001a\u0004\b\u001d\u0010\u0013"}, d2 = {"Lorg/vocaltract/pixel/CalibrationStepSummary;", "", "id", "", "frames", "", "voicedFrames", "meanF0Hz", "", "meanFormantsHz", "", "meanSnrDb", "meanTractConfidence", "abstainedFrames", "<init>", "(Ljava/lang/String;IILjava/lang/Float;Ljava/util/List;FFI)V", "getId", "()Ljava/lang/String;", "getFrames", "()I", "getVoicedFrames", "getMeanF0Hz", "()Ljava/lang/Float;", "Ljava/lang/Float;", "getMeanFormantsHz", "()Ljava/util/List;", "getMeanSnrDb", "()F", "getMeanTractConfidence", "getAbstainedFrames", "component1", "component2", "component3", "component4", "component5", "component6", "component7", "component8", "copy", "(Ljava/lang/String;IILjava/lang/Float;Ljava/util/List;FFI)Lorg/vocaltract/pixel/CalibrationStepSummary;", "equals", "", "other", "hashCode", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class CalibrationStepSummary {
    private final int abstainedFrames;
    private final int frames;
    private final String id;
    private final Float meanF0Hz;
    private final List<Float> meanFormantsHz;
    private final float meanSnrDb;
    private final float meanTractConfidence;
    private final int voicedFrames;

    /* renamed from: component1, reason: from getter */
    public final String getId() {
        return this.id;
    }

    /* renamed from: component2, reason: from getter */
    public final int getFrames() {
        return this.frames;
    }

    /* renamed from: component3, reason: from getter */
    public final int getVoicedFrames() {
        return this.voicedFrames;
    }

    /* renamed from: component4, reason: from getter */
    public final Float getMeanF0Hz() {
        return this.meanF0Hz;
    }

    public final List<Float> component5() {
        return this.meanFormantsHz;
    }

    /* renamed from: component6, reason: from getter */
    public final float getMeanSnrDb() {
        return this.meanSnrDb;
    }

    /* renamed from: component7, reason: from getter */
    public final float getMeanTractConfidence() {
        return this.meanTractConfidence;
    }

    /* renamed from: component8, reason: from getter */
    public final int getAbstainedFrames() {
        return this.abstainedFrames;
    }

    public final CalibrationStepSummary copy(String id, int frames, int voicedFrames, Float meanF0Hz, List<Float> meanFormantsHz, float meanSnrDb, float meanTractConfidence, int abstainedFrames) {
        Intrinsics.checkNotNullParameter(id, "id");
        Intrinsics.checkNotNullParameter(meanFormantsHz, "meanFormantsHz");
        return new CalibrationStepSummary(id, frames, voicedFrames, meanF0Hz, meanFormantsHz, meanSnrDb, meanTractConfidence, abstainedFrames);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CalibrationStepSummary)) {
            return false;
        }
        CalibrationStepSummary calibrationStepSummary = (CalibrationStepSummary) other;
        return Intrinsics.areEqual(this.id, calibrationStepSummary.id) && this.frames == calibrationStepSummary.frames && this.voicedFrames == calibrationStepSummary.voicedFrames && Intrinsics.areEqual((Object) this.meanF0Hz, (Object) calibrationStepSummary.meanF0Hz) && Intrinsics.areEqual(this.meanFormantsHz, calibrationStepSummary.meanFormantsHz) && Float.compare(this.meanSnrDb, calibrationStepSummary.meanSnrDb) == 0 && Float.compare(this.meanTractConfidence, calibrationStepSummary.meanTractConfidence) == 0 && this.abstainedFrames == calibrationStepSummary.abstainedFrames;
    }

    public int hashCode() {
        int hashCode = ((((this.id.hashCode() * 31) + Integer.hashCode(this.frames)) * 31) + Integer.hashCode(this.voicedFrames)) * 31;
        Float f = this.meanF0Hz;
        return ((((((((hashCode + (f == null ? 0 : f.hashCode())) * 31) + this.meanFormantsHz.hashCode()) * 31) + Float.hashCode(this.meanSnrDb)) * 31) + Float.hashCode(this.meanTractConfidence)) * 31) + Integer.hashCode(this.abstainedFrames);
    }

    public String toString() {
        return "CalibrationStepSummary(id=" + this.id + ", frames=" + this.frames + ", voicedFrames=" + this.voicedFrames + ", meanF0Hz=" + this.meanF0Hz + ", meanFormantsHz=" + this.meanFormantsHz + ", meanSnrDb=" + this.meanSnrDb + ", meanTractConfidence=" + this.meanTractConfidence + ", abstainedFrames=" + this.abstainedFrames + ")";
    }

    public CalibrationStepSummary(String id, int i, int i2, Float f, List<Float> meanFormantsHz, float f2, float f3, int i3) {
        Intrinsics.checkNotNullParameter(id, "id");
        Intrinsics.checkNotNullParameter(meanFormantsHz, "meanFormantsHz");
        this.id = id;
        this.frames = i;
        this.voicedFrames = i2;
        this.meanF0Hz = f;
        this.meanFormantsHz = meanFormantsHz;
        this.meanSnrDb = f2;
        this.meanTractConfidence = f3;
        this.abstainedFrames = i3;
    }

    public final String getId() {
        return this.id;
    }

    public final int getFrames() {
        return this.frames;
    }

    public final int getVoicedFrames() {
        return this.voicedFrames;
    }

    public final Float getMeanF0Hz() {
        return this.meanF0Hz;
    }

    public final List<Float> getMeanFormantsHz() {
        return this.meanFormantsHz;
    }

    public final float getMeanSnrDb() {
        return this.meanSnrDb;
    }

    public final float getMeanTractConfidence() {
        return this.meanTractConfidence;
    }

    public final int getAbstainedFrames() {
        return this.abstainedFrames;
    }
}
