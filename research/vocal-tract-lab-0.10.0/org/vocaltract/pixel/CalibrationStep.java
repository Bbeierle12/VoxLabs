package org.vocaltract.pixel;

import kotlin.Metadata;
import kotlin.jvm.internal.DefaultConstructorMarker;
import kotlin.jvm.internal.Intrinsics;

/* compiled from: SingerCalibration.kt */
@Metadata(d1 = {"\u0000&\n\u0002\u0018\u0002\n\u0002\u0010\u0000\n\u0000\n\u0002\u0010\u000e\n\u0002\b\u0002\n\u0002\u0010\t\n\u0000\n\u0002\u0010\u000b\n\u0002\b\u0011\n\u0002\u0010\b\n\u0000\b\u0086\b\u0018\u00002\u00020\u0001B)\u0012\u0006\u0010\u0002\u001a\u00020\u0003\u0012\u0006\u0010\u0004\u001a\u00020\u0003\u0012\u0006\u0010\u0005\u001a\u00020\u0006\u0012\b\b\u0002\u0010\u0007\u001a\u00020\b¢\u0006\u0004\b\t\u0010\nJ\t\u0010\u0012\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0013\u001a\u00020\u0003HÆ\u0003J\t\u0010\u0014\u001a\u00020\u0006HÆ\u0003J\t\u0010\u0015\u001a\u00020\bHÆ\u0003J1\u0010\u0016\u001a\u00020\u00002\b\b\u0002\u0010\u0002\u001a\u00020\u00032\b\b\u0002\u0010\u0004\u001a\u00020\u00032\b\b\u0002\u0010\u0005\u001a\u00020\u00062\b\b\u0002\u0010\u0007\u001a\u00020\bHÆ\u0001J\u0013\u0010\u0017\u001a\u00020\b2\b\u0010\u0018\u001a\u0004\u0018\u00010\u0001HÖ\u0003J\t\u0010\u0019\u001a\u00020\u001aHÖ\u0001J\t\u0010\u001b\u001a\u00020\u0003HÖ\u0001R\u0011\u0010\u0002\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\u000b\u0010\fR\u0011\u0010\u0004\u001a\u00020\u0003¢\u0006\b\n\u0000\u001a\u0004\b\r\u0010\fR\u0011\u0010\u0005\u001a\u00020\u0006¢\u0006\b\n\u0000\u001a\u0004\b\u000e\u0010\u000fR\u0011\u0010\u0007\u001a\u00020\b¢\u0006\b\n\u0000\u001a\u0004\b\u0010\u0010\u0011"}, d2 = {"Lorg/vocaltract/pixel/CalibrationStep;", "", "id", "", "prompt", "durationMillis", "", "forceNoiseLearning", "", "<init>", "(Ljava/lang/String;Ljava/lang/String;JZ)V", "getId", "()Ljava/lang/String;", "getPrompt", "getDurationMillis", "()J", "getForceNoiseLearning", "()Z", "component1", "component2", "component3", "component4", "copy", "equals", "other", "hashCode", "", "toString"}, k = 1, mv = {2, 0, 0}, xi = 48)
/* loaded from: /tmp/claude-0/-home-user-VoxLabs/790d6529-3be5-5735-9f45-ee1ed7631e39/scratchpad/vtl/x/classes.dex */
public final /* data */ class CalibrationStep {
    private final long durationMillis;
    private final boolean forceNoiseLearning;
    private final String id;
    private final String prompt;

    public static /* synthetic */ CalibrationStep copy$default(CalibrationStep calibrationStep, String str, String str2, long j, boolean z, int i, Object obj) {
        if ((i & 1) != 0) {
            str = calibrationStep.id;
        }
        if ((i & 2) != 0) {
            str2 = calibrationStep.prompt;
        }
        String str3 = str2;
        if ((i & 4) != 0) {
            j = calibrationStep.durationMillis;
        }
        long j2 = j;
        if ((i & 8) != 0) {
            z = calibrationStep.forceNoiseLearning;
        }
        return calibrationStep.copy(str, str3, j2, z);
    }

    /* renamed from: component1, reason: from getter */
    public final String getId() {
        return this.id;
    }

    /* renamed from: component2, reason: from getter */
    public final String getPrompt() {
        return this.prompt;
    }

    /* renamed from: component3, reason: from getter */
    public final long getDurationMillis() {
        return this.durationMillis;
    }

    /* renamed from: component4, reason: from getter */
    public final boolean getForceNoiseLearning() {
        return this.forceNoiseLearning;
    }

    public final CalibrationStep copy(String id, String prompt, long durationMillis, boolean forceNoiseLearning) {
        Intrinsics.checkNotNullParameter(id, "id");
        Intrinsics.checkNotNullParameter(prompt, "prompt");
        return new CalibrationStep(id, prompt, durationMillis, forceNoiseLearning);
    }

    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof CalibrationStep)) {
            return false;
        }
        CalibrationStep calibrationStep = (CalibrationStep) other;
        return Intrinsics.areEqual(this.id, calibrationStep.id) && Intrinsics.areEqual(this.prompt, calibrationStep.prompt) && this.durationMillis == calibrationStep.durationMillis && this.forceNoiseLearning == calibrationStep.forceNoiseLearning;
    }

    public int hashCode() {
        return (((((this.id.hashCode() * 31) + this.prompt.hashCode()) * 31) + Long.hashCode(this.durationMillis)) * 31) + Boolean.hashCode(this.forceNoiseLearning);
    }

    public String toString() {
        return "CalibrationStep(id=" + this.id + ", prompt=" + this.prompt + ", durationMillis=" + this.durationMillis + ", forceNoiseLearning=" + this.forceNoiseLearning + ")";
    }

    public CalibrationStep(String id, String prompt, long j, boolean z) {
        Intrinsics.checkNotNullParameter(id, "id");
        Intrinsics.checkNotNullParameter(prompt, "prompt");
        this.id = id;
        this.prompt = prompt;
        this.durationMillis = j;
        this.forceNoiseLearning = z;
    }

    public /* synthetic */ CalibrationStep(String str, String str2, long j, boolean z, int i, DefaultConstructorMarker defaultConstructorMarker) {
        this(str, str2, j, (i & 8) != 0 ? false : z);
    }

    public final String getId() {
        return this.id;
    }

    public final String getPrompt() {
        return this.prompt;
    }

    public final long getDurationMillis() {
        return this.durationMillis;
    }

    public final boolean getForceNoiseLearning() {
        return this.forceNoiseLearning;
    }
}
